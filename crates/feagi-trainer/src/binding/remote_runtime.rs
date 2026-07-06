//! Concrete remote [`FeagiRuntime`] driving a live FEAGI over ZMQ via the `feagi-agent` client
//! (plan Phase 1b).
//!
//! # Why this is best-effort, not benchmark-grade
//!
//! FEAGI is a free-running asynchronous spiking substrate: its burst engine advances on its own
//! clock and cannot be deterministically single-stepped or paused from a client (empirically
//! confirmed against a live instance — REST pause/hold does not halt the burst loop). The runtime
//! contract's [`FeagiRuntime::step`] therefore uses a **wall-clock** model (Option A): it waits
//! `ticks × burst_period` to give the brain a consistent integration window, then
//! [`FeagiRuntime::collect_motor`] reads the *freshest* motor frame. The exact number of bursts
//! processed during the wait is not guaranteed. True burst-level reproducibility is deferred to a
//! future embedded runtime (ADR-006).
//!
//! # Transport
//!
//! Pure ZMQ client, no Composer/REST coupling:
//! - Registers over the command/control requester ([`CommandControlAgent`]) to obtain the session
//!   [`AgentID`] and the per-capability data endpoints.
//! - Publishes pre-encoded sensory neurons (and affect-channel reward stimulation) on the sensory
//!   PUSH endpoint as a [`FeagiByteContainer`] holding one `NeuronCategoricalXYZP` structure.
//! - Subscribes to the motor PUB endpoint and decodes the latest `NeuronCategoricalXYZP` frame.
//!
//! All hosts/ports/timeouts come from [`RemoteRuntimeConfig`]; nothing is hardcoded.

use std::fmt::Display;
use std::thread::sleep;
use std::time::{Duration, Instant};

use feagi_agent::clients::{AgentRegistrationStatus, CommandControlAgent};
use feagi_agent::{AgentCapabilities, AgentDescriptor, AuthToken};
use feagi_io::protocol_implementations::zmq::FeagiZmqClientRequesterProperties;
use feagi_io::traits_and_enums::client::{FeagiClientPusher, FeagiClientSubscriber};
use feagi_io::traits_and_enums::shared::FeagiEndpointState;
use feagi_io::AgentID;
use feagi_serialization::FeagiByteContainer;
use feagi_structures::genomic::cortical_area::{CoreCorticalType, CorticalID};
use feagi_structures::neuron_voxels::xyzp::{
    CorticalMappedXYZPNeuronVoxels, NeuronVoxelXYZPArrays,
};

use crate::binding::reward::{AffectChannel, RewardSignal};
use crate::binding::runtime::FeagiRuntime;
use crate::error::TrainerError;

/// Connection and timing parameters for [`RemoteFeagiRuntime`].
///
/// Every field is caller-supplied (sourced from the run configuration); the runtime hardcodes no
/// hosts, ports, or timeouts.
#[derive(Debug, Clone)]
pub struct RemoteRuntimeConfig {
    /// ZMQ endpoint of FEAGI's agent registration (command/control) socket, e.g.
    /// `tcp://<host>:<port>`.
    pub registration_endpoint: String,
    /// Agent descriptor manufacturer field.
    pub manufacturer: String,
    /// Agent descriptor name field.
    pub agent_name: String,
    /// Agent descriptor version field.
    pub agent_version: u32,
    /// Authentication token presented at registration.
    pub auth_token: [u8; 32],
    /// Wall-clock duration that one burst tick represents (i.e. `1 / burst_frequency`). Used by
    /// [`FeagiRuntime::step`] to size the integration wait.
    pub burst_period: Duration,
    /// Poll cadence while waiting for the registration response.
    pub registration_poll_interval: Duration,
    /// Hard deadline for completing registration.
    pub registration_timeout: Duration,
    /// Poll cadence while waiting for a motor frame in [`FeagiRuntime::collect_motor`].
    pub motor_poll_interval: Duration,
    /// Hard deadline for receiving at least one motor frame in [`FeagiRuntime::collect_motor`].
    pub motor_collect_timeout: Duration,
}

/// Remote runtime bound to a live FEAGI instance over ZMQ.
///
/// Construct it with [`RemoteFeagiRuntime::connect_and_register`]. `SensoryFrame` and `MotorFrame`
/// are [`CorticalMappedXYZPNeuronVoxels`] — the same type produced by the population encoder and
/// consumed by the class decoder — so the runtime slots directly into the binding pipeline.
pub struct RemoteFeagiRuntime {
    command_agent: CommandControlAgent,
    agent_id: AgentID,
    sensory_pusher: Box<dyn FeagiClientPusher>,
    motor_subscriber: Box<dyn FeagiClientSubscriber>,
    send_buffer: FeagiByteContainer,
    recv_buffer: FeagiByteContainer,
    motor_buffer: CorticalMappedXYZPNeuronVoxels,
    increment: u16,
    burst_period: Duration,
    motor_poll_interval: Duration,
    motor_collect_timeout: Duration,
}

/// Maps any foreign error (agent/network/serialization) onto a [`TrainerError::Runtime`].
fn rt<E: Display>(error: E) -> TrainerError {
    TrainerError::Runtime(error.to_string())
}

/// Resolves the core affect cortical_area area that a reward channel stimulates.
fn affect_cortical_id(channel: AffectChannel) -> CorticalID {
    let core = match channel {
        AffectChannel::Pain => CoreCorticalType::Pain,
        AffectChannel::Pleasure => CoreCorticalType::Pleasure,
        AffectChannel::Fear => CoreCorticalType::Fear,
        AffectChannel::Hope => CoreCorticalType::Hope,
    };
    core.to_cortical_id()
}

impl RemoteFeagiRuntime {
    /// Registers with FEAGI and connects the sensory (PUSH) and motor (SUB) data sockets.
    ///
    /// Requests the `SendSensorData` and `ReceiveMotorData` capabilities, blocks (polling at
    /// `registration_poll_interval`) until the server returns the session id and endpoints or
    /// `registration_timeout` elapses, then opens both data sockets.
    pub fn connect_and_register(config: RemoteRuntimeConfig) -> Result<Self, TrainerError> {
        let requester_properties =
            FeagiZmqClientRequesterProperties::new(&config.registration_endpoint).map_err(rt)?;
        let mut command_agent = CommandControlAgent::new(Box::new(requester_properties));
        command_agent.request_connect().map_err(rt)?;

        let descriptor = AgentDescriptor::new(
            &config.manufacturer,
            &config.agent_name,
            config.agent_version,
        )
        .map_err(rt)?;
        command_agent
            .request_registration(
                descriptor,
                AuthToken::new(config.auth_token),
                vec![
                    AgentCapabilities::SendSensorData,
                    AgentCapabilities::ReceiveMotorData,
                ],
            )
            .map_err(rt)?;

        let registration_deadline = Instant::now() + config.registration_timeout;
        let (agent_id, sensory_endpoint, motor_endpoint) = loop {
            command_agent.poll_for_messages().map_err(rt)?;
            if let AgentRegistrationStatus::Registered(session_id, endpoints) =
                command_agent.registration_status()
            {
                let sensory = endpoints
                    .get(&AgentCapabilities::SendSensorData)
                    .ok_or_else(|| {
                        TrainerError::Runtime(
                            "FEAGI registration returned no SendSensorData endpoint".to_string(),
                        )
                    })?
                    .clone();
                let motor = endpoints
                    .get(&AgentCapabilities::ReceiveMotorData)
                    .ok_or_else(|| {
                        TrainerError::Runtime(
                            "FEAGI registration returned no ReceiveMotorData endpoint".to_string(),
                        )
                    })?
                    .clone();
                break (*session_id, sensory, motor);
            }
            if Instant::now() >= registration_deadline {
                return Err(TrainerError::Runtime(
                    "timed out waiting for FEAGI registration response".to_string(),
                ));
            }
            sleep(config.registration_poll_interval);
        };

        let mut sensory_pusher = sensory_endpoint
            .try_create_boxed_client_pusher_properties()
            .map_err(rt)?
            .as_boxed_client_pusher();
        sensory_pusher.request_connect().map_err(rt)?;

        let mut motor_subscriber = motor_endpoint
            .try_create_boxed_client_subscriber_properties()
            .map_err(rt)?
            .as_boxed_client_subscriber();
        motor_subscriber.request_connect().map_err(rt)?;

        Ok(Self {
            command_agent,
            agent_id,
            sensory_pusher,
            motor_subscriber,
            send_buffer: FeagiByteContainer::new_empty(),
            recv_buffer: FeagiByteContainer::new_empty(),
            motor_buffer: CorticalMappedXYZPNeuronVoxels::new(),
            increment: 0,
            burst_period: config.burst_period,
            motor_poll_interval: config.motor_poll_interval,
            motor_collect_timeout: config.motor_collect_timeout,
        })
    }

    /// The session id assigned by FEAGI at registration.
    pub fn agent_id(&self) -> AgentID {
        self.agent_id
    }

    /// Serializes a neuron-voxel frame into the reusable send buffer (tagged with the session id)
    /// and publishes it on the sensory PUSH socket.
    fn publish_neurons(
        &mut self,
        neurons: &CorticalMappedXYZPNeuronVoxels,
    ) -> Result<(), TrainerError> {
        self.send_buffer
            .overwrite_byte_data_with_single_struct_data(neurons, self.increment)
            .map_err(rt)?;
        self.send_buffer
            .set_agent_identifier(self.agent_id)
            .map_err(rt)?;
        self.increment = self.increment.wrapping_add(1);

        match self.sensory_pusher.poll() {
            FeagiEndpointState::ActiveWaiting | FeagiEndpointState::ActiveHasData => {}
            other => {
                return Err(TrainerError::Runtime(format!(
                    "sensory pusher is not ready to send (state: {other:?})"
                )));
            }
        }
        self.sensory_pusher
            .publish_data(self.send_buffer.get_byte_ref())
            .map_err(rt)
    }

    /// Deregisters and tears down the command/control socket. Best-effort; errors are returned so
    /// callers may log them.
    pub fn shutdown(&mut self) -> Result<(), TrainerError> {
        self.command_agent.request_deregistration(None).map_err(rt)
    }
}

impl FeagiRuntime for RemoteFeagiRuntime {
    type SensoryFrame = CorticalMappedXYZPNeuronVoxels;
    type MotorFrame = CorticalMappedXYZPNeuronVoxels;

    fn submit_sensory(&mut self, frame: Self::SensoryFrame) -> Result<(), TrainerError> {
        self.publish_neurons(&frame)
    }

    fn submit_reward(&mut self, signals: &[RewardSignal]) -> Result<(), TrainerError> {
        if signals.is_empty() {
            return Ok(());
        }
        // Reward is delivered as a single-spike stimulation in the corresponding core affect area;
        // multiple signals for the same channel accumulate as separate voxels.
        let mut affect = CorticalMappedXYZPNeuronVoxels::new();
        for signal in signals {
            let cortical_id = affect_cortical_id(signal.channel);
            if !affect.contains_cortical_id(&cortical_id) {
                affect.insert(cortical_id, NeuronVoxelXYZPArrays::new());
            }
            affect
                .get_neurons_of_mut(&cortical_id)
                .expect("array was just inserted")
                .push_raw(0, 0, 0, signal.magnitude as f32);
        }
        self.publish_neurons(&affect)
    }

    fn step(&mut self, ticks: u32) -> Result<(), TrainerError> {
        // Keep the session alive, then give the free-running brain a wall-clock integration
        // window proportional to the requested ticks (Option A).
        self.command_agent.send_heartbeat().map_err(rt)?;
        sleep(self.burst_period * ticks);
        Ok(())
    }

    fn collect_motor(&mut self) -> Result<Self::MotorFrame, TrainerError> {
        let deadline = Instant::now() + self.motor_collect_timeout;
        let mut received = false;

        loop {
            let state = self.motor_subscriber.poll().clone();
            match state {
                FeagiEndpointState::ActiveHasData => {
                    let bytes = self.motor_subscriber.consume_retrieved_data().map_err(rt)?;
                    self.recv_buffer
                        .try_write_data_by_copy_and_verify(bytes)
                        .map_err(rt)?;
                    // Decode in place; draining continues so we return the freshest frame.
                    if self
                        .recv_buffer
                        .try_update_struct_from_first_found_struct_of_type(&mut self.motor_buffer)
                        .map_err(rt)?
                    {
                        received = true;
                    }
                }
                FeagiEndpointState::ActiveWaiting => {
                    if received {
                        break;
                    }
                    if Instant::now() >= deadline {
                        break;
                    }
                    sleep(self.motor_poll_interval);
                }
                FeagiEndpointState::Errored(error) => {
                    return Err(TrainerError::Runtime(format!(
                        "motor subscriber errored: {error:?}"
                    )));
                }
                FeagiEndpointState::Inactive | FeagiEndpointState::Pending => {
                    if Instant::now() >= deadline {
                        break;
                    }
                    sleep(self.motor_poll_interval);
                }
            }
        }

        if !received {
            return Err(TrainerError::Runtime(
                "no motor frame received from FEAGI within the collect timeout".to_string(),
            ));
        }
        Ok(self.motor_buffer.clone())
    }
}
