// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

// Registration Handler - processes agent registration requests

use std::collections::HashMap;
use std::sync::Arc;

use ahash::AHashSet;
use parking_lot::RwLock;
use tracing::{debug, error, info, warn};

#[allow(unused_imports)]
use feagi_services::traits::registration_handler::RegistrationHandlerTrait;
pub use feagi_services::types::registration::{
    AreaStatus, CorticalAreaAvailability, CorticalAreaStatus, RegistrationRequest,
    RegistrationResponse, TransportConfig,
};
use feagi_structures::genomic::cortical_area::descriptors::{
    CorticalSubUnitIndex, CorticalUnitIndex,
};
use feagi_structures::genomic::cortical_area::io_cortical_area_configuration_flag::FrameChangeHandling;
use feagi_structures::genomic::cortical_area::CorticalID;
use feagi_structures::genomic::{MotorCorticalUnit, SensoryCorticalUnit};
use feagi_structures::FeagiDataError;

use super::agent_registry::{
    AgentCapabilities, AgentInfo, AgentRegistry, AgentTransport, AgentType, MotorUnit, SensoryUnit,
};

/// Type alias for registration callbacks
pub type RegistrationCallback =
    Arc<parking_lot::Mutex<Option<Box<dyn Fn(String, String, String) + Send + Sync>>>>;
pub type DeregistrationCallback =
    Arc<parking_lot::Mutex<Option<Box<dyn Fn(String) + Send + Sync>>>>;
/// Type alias for dynamic gating callbacks
pub type DynamicGatingCallback = Arc<parking_lot::Mutex<Option<Box<dyn Fn(String) + Send + Sync>>>>;

/// Registration Handler
pub struct RegistrationHandler {
    agent_registry: Arc<RwLock<AgentRegistry>>,
    shm_base_path: String,
    /// Optional reference to burst engine's sensory agent manager for SHM I/O
    sensory_agent_manager: Arc<
        parking_lot::Mutex<Option<Arc<std::sync::Mutex<feagi_npu_burst_engine::AgentManager>>>>,
    >,
    /// Optional reference to burst loop runner for motor subscription tracking
    burst_runner: Arc<
        parking_lot::Mutex<
            Option<Arc<parking_lot::RwLock<feagi_npu_burst_engine::BurstLoopRunner>>>,
        >,
    >,
    /// Optional reference to GenomeService for creating cortical areas
    genome_service: Arc<
        parking_lot::Mutex<Option<Arc<dyn feagi_services::traits::GenomeService + Send + Sync>>>,
    >,
    /// Optional reference to ConnectomeService for checking cortical area existence
    connectome_service: Arc<
        parking_lot::Mutex<
            Option<Arc<dyn feagi_services::traits::ConnectomeService + Send + Sync>>,
        >,
    >,
    /// Configuration for auto-creating missing cortical areas
    auto_create_missing_areas: bool,
    /// Actual ZMQ port numbers (from config, NOT hardcoded)
    registration_port: u16,
    sensory_port: u16,
    motor_port: u16,
    viz_port: u16,
    /// WebSocket port numbers (from config, NOT hardcoded)
    ws_enabled: bool,
    ws_host: String,
    ws_sensory_port: u16,
    ws_motor_port: u16,
    ws_viz_port: u16,
    ws_registration_port: u16,
    /// Callbacks for Python integration
    on_agent_registered: RegistrationCallback,
    on_agent_deregistered: DeregistrationCallback,
    /// Callbacks for dynamic stream gating
    on_agent_registered_dynamic: DynamicGatingCallback,
    on_agent_deregistered_dynamic: DynamicGatingCallback,
}

impl RegistrationHandler {
    pub fn new(
        agent_registry: Arc<RwLock<AgentRegistry>>,
        registration_port: u16,
        sensory_port: u16,
        motor_port: u16,
        viz_port: u16,
    ) -> Self {
        Self {
            agent_registry,
            shm_base_path: "/tmp".to_string(),
            sensory_agent_manager: Arc::new(parking_lot::Mutex::new(None)),
            burst_runner: Arc::new(parking_lot::Mutex::new(None)),
            genome_service: Arc::new(parking_lot::Mutex::new(None)),
            connectome_service: Arc::new(parking_lot::Mutex::new(None)),
            auto_create_missing_areas: true, // Default enabled
            registration_port,
            sensory_port,
            motor_port,
            viz_port,
            ws_enabled: false,
            ws_host: "0.0.0.0".to_string(),
            ws_sensory_port: 9051,
            ws_motor_port: 9052,
            ws_viz_port: 9050,
            ws_registration_port: 9053,
            on_agent_registered: Arc::new(parking_lot::Mutex::new(None)),
            on_agent_deregistered: Arc::new(parking_lot::Mutex::new(None)),
            on_agent_registered_dynamic: Arc::new(parking_lot::Mutex::new(None)),
            on_agent_deregistered_dynamic: Arc::new(parking_lot::Mutex::new(None)),
        }
    }

    /// Set GenomeService for creating cortical areas
    pub fn set_genome_service(
        &self,
        service: Arc<dyn feagi_services::traits::GenomeService + Send + Sync>,
    ) {
        *self.genome_service.lock() = Some(service);
        info!("🦀 [REGISTRATION] GenomeService connected for cortical area creation");
    }

    /// Set ConnectomeService for checking cortical area existence
    pub fn set_connectome_service(
        &self,
        service: Arc<dyn feagi_services::traits::ConnectomeService + Send + Sync>,
    ) {
        *self.connectome_service.lock() = Some(service);
        info!("🦀 [REGISTRATION] ConnectomeService connected for cortical area checking");
    }

    /// Set auto-create missing cortical areas configuration
    pub fn set_auto_create_missing_areas(&mut self, enabled: bool) {
        self.auto_create_missing_areas = enabled;
        info!(
            "🦀 [REGISTRATION] Auto-create missing cortical areas: {}",
            enabled
        );
    }

    /// Set WebSocket transport configuration
    pub fn set_websocket_config(
        &mut self,
        enabled: bool,
        host: String,
        sensory_port: u16,
        motor_port: u16,
        viz_port: u16,
        registration_port: u16,
    ) {
        self.ws_enabled = enabled;
        self.ws_host = host;
        self.ws_sensory_port = sensory_port;
        self.ws_motor_port = motor_port;
        self.ws_viz_port = viz_port;
        self.ws_registration_port = registration_port;
        info!(
            "🦀 [REGISTRATION] WebSocket transport configured: enabled={}, ports={}:{}:{}:{}",
            enabled, sensory_port, motor_port, viz_port, registration_port
        );
    }

    /// Set burst runner reference (for motor subscription tracking)
    pub fn set_burst_runner(
        &self,
        runner: Arc<parking_lot::RwLock<feagi_npu_burst_engine::BurstLoopRunner>>,
    ) {
        *self.burst_runner.lock() = Some(runner);
        info!("🦀 [REGISTRATION] Burst runner connected for motor subscriptions");
    }

    /// Set the sensory agent manager (for SHM I/O coordination)
    pub fn set_sensory_agent_manager(
        &self,
        manager: Arc<std::sync::Mutex<feagi_npu_burst_engine::AgentManager>>,
    ) {
        *self.sensory_agent_manager.lock() = Some(manager);
        info!("🦀 [REGISTRATION] Sensory agent manager connected");
    }

    /// Set callback for agent registration events (for Python integration)
    pub fn set_on_agent_registered<F>(&self, callback: F)
    where
        F: Fn(String, String, String) + Send + Sync + 'static,
    {
        *self.on_agent_registered.lock() = Some(Box::new(callback));
        info!("🦀 [REGISTRATION] Agent registration callback set");
    }

    /// Set callback for agent deregistration events (for Python integration)
    pub fn set_on_agent_deregistered<F>(&self, callback: F)
    where
        F: Fn(String) + Send + Sync + 'static,
    {
        *self.on_agent_deregistered.lock() = Some(Box::new(callback));
        info!("🦀 [REGISTRATION] Agent deregistration callback set");
    }

    /// Set callback for dynamic stream gating on agent registration
    pub fn set_on_agent_registered_dynamic<F>(&self, callback: F)
    where
        F: Fn(String) + Send + Sync + 'static,
    {
        *self.on_agent_registered_dynamic.lock() = Some(Box::new(callback));
        info!("🦀 [REGISTRATION] Dynamic gating registration callback set");
    }

    /// Set callback for dynamic stream gating on agent deregistration
    pub fn set_on_agent_deregistered_dynamic<F>(&self, callback: F)
    where
        F: Fn(String) + Send + Sync + 'static,
    {
        *self.on_agent_deregistered_dynamic.lock() = Some(Box::new(callback));
        info!("🦀 [REGISTRATION] Dynamic gating deregistration callback set");
    }

    /// Convert area name to CorticalID base64 string
    ///
    /// For IPU/OPU areas, the name should be a valid prefix (e.g., "isvi") or a base64-encoded CorticalID.
    /// Uses null-byte padding for short prefixes (standard approach).
    fn area_name_to_cortical_id(&self, area_name: &str) -> Result<String, FeagiDataError> {
        // First, check if it's already a base64-encoded CorticalID
        if let Ok(cortical_id) = CorticalID::try_from_base_64(area_name) {
            return Ok(cortical_id.as_base_64());
        }

        // Try with null byte padding (standard approach)
        let mut bytes = [b'\0'; 8];
        let name_bytes = area_name.as_bytes();
        let copy_len = name_bytes.len().min(8);
        bytes[..copy_len].copy_from_slice(&name_bytes[..copy_len]);

        if let Ok(cortical_id) = CorticalID::try_from_bytes(&bytes) {
            return Ok(cortical_id.as_base_64());
        }

        // Failed - return error with guidance
        Err(FeagiDataError::BadParameters(format!(
            "Failed to create CorticalID from area name '{}' (length: {}). \
            The area name must be either:\n\
            1. A valid prefix (1-8 characters) that can be padded to 8 bytes, or\n\
            2. A base64-encoded CorticalID.\n\
            For IPU/OPU areas, use valid prefixes like 'isvi', 'imot', 'oseg', etc., \
            or provide a base64-encoded CorticalID.",
            area_name,
            area_name.len()
        )))
    }

    /// Get all cortical IDs for a given SensoryCorticalUnit using generic method dispatch
    fn get_all_cortical_ids_for_unit(
        &self,
        unit: SensoryCorticalUnit,
        frame_change_handling: FrameChangeHandling,
        percentage_neuron_positioning: feagi_structures::genomic::cortical_area::io_cortical_area_configuration_flag::PercentageNeuronPositioning,
        group: CorticalUnitIndex,
    ) -> Result<Vec<CorticalID>, FeagiDataError> {
        // Dispatch to the appropriate get_cortical_ids_array_for method based on unit type
        // This is systematic (covers all types) not hardcoded for one specific type
        // Note: Method signatures vary based on cortical_type_parameters in the template:
        // - Units with Percentage type need: (frame_change_handling, percentage_neuron_positioning, group)
        // - Units with ImageFrame/SegmentedImageFrame/MiscData need: (frame_change_handling, group)
        // - Units with Boolean type need: (group) only
        use SensoryCorticalUnit::*;
        let cortical_ids_array: Vec<CorticalID> = match unit {
            Infrared => SensoryCorticalUnit::get_cortical_ids_array_for_infrared_with_parameters(
                frame_change_handling,
                percentage_neuron_positioning,
                group,
            )
            .to_vec(),
            Proximity => SensoryCorticalUnit::get_cortical_ids_array_for_proximity_with_parameters(
                frame_change_handling,
                percentage_neuron_positioning,
                group,
            )
            .to_vec(),
            Shock => SensoryCorticalUnit::get_cortical_ids_array_for_shock_with_parameters(
                frame_change_handling,
                percentage_neuron_positioning,
                group,
            )
            .to_vec(),
            Battery => SensoryCorticalUnit::get_cortical_ids_array_for_battery_with_parameters(
                frame_change_handling,
                percentage_neuron_positioning,
                group,
            )
            .to_vec(),
            Servo => SensoryCorticalUnit::get_cortical_ids_array_for_servo_with_parameters(
                frame_change_handling,
                percentage_neuron_positioning,
                group,
            )
            .to_vec(),
            AnalogGPIO => {
                SensoryCorticalUnit::get_cortical_ids_array_for_analog_g_p_i_o_with_parameters(
                    frame_change_handling,
                    percentage_neuron_positioning,
                    group,
                )
                .to_vec()
            }
            DigitalGPIO => {
                SensoryCorticalUnit::get_cortical_ids_array_for_digital_g_p_i_o_with_parameters(
                    group,
                )
                .to_vec()
            }
            MiscData => SensoryCorticalUnit::get_cortical_ids_array_for_misc_data_with_parameters(
                frame_change_handling,
                group,
            )
            .to_vec(),
            TextEnglishInput => {
                SensoryCorticalUnit::get_cortical_ids_array_for_text_english_input_with_parameters(
                    frame_change_handling,
                    group,
                )
                .to_vec()
            }
            Vision => {
                SensoryCorticalUnit::get_cortical_ids_array_for_segmented_vision_with_parameters(
                    frame_change_handling,
                    group,
                )
                .to_vec()
            }
            SegmentedVision => {
                SensoryCorticalUnit::get_cortical_ids_array_for_segmented_vision_with_parameters(
                    frame_change_handling,
                    group,
                )
                .to_vec()
            }
            Accelerometer => {
                SensoryCorticalUnit::get_cortical_ids_array_for_accelerometer_with_parameters(
                    frame_change_handling,
                    percentage_neuron_positioning,
                    group,
                )
                .to_vec()
            }
            Gyroscope => SensoryCorticalUnit::get_cortical_ids_array_for_gyroscope_with_parameters(
                frame_change_handling,
                percentage_neuron_positioning,
                group,
            )
            .to_vec(),
        };

        Ok(cortical_ids_array)
    }

    /// Helper function to safely call async code from sync context
    /// Always uses a separate thread to avoid blocking the current runtime
    fn block_on_async_service<F>(&self, future_factory: F) -> Result<bool, FeagiDataError>
    where
        F: FnOnce() -> std::pin::Pin<
                Box<
                    dyn std::future::Future<Output = feagi_services::types::ServiceResult<bool>>
                        + Send,
                >,
            > + Send
            + 'static,
    {
        // Always use a separate thread to avoid blocking the current runtime
        // This works whether we're in an async context or not
        debug!("🦀 [REGISTRATION] Starting async service call in separate thread");
        let future = future_factory();
        let (tx, rx) = std::sync::mpsc::channel();

        std::thread::spawn(move || {
            let result = (|| -> Result<bool, FeagiDataError> {
                debug!("🦀 [REGISTRATION] Creating new tokio runtime in thread");
                let rt = tokio::runtime::Runtime::new().map_err(|e| {
                    FeagiDataError::InternalError(format!("Failed to create runtime: {}", e))
                })?;
                debug!("🦀 [REGISTRATION] Blocking on async future");
                let result = rt
                    .block_on(future)
                    .map_err(|e| FeagiDataError::InternalError(format!("Service error: {}", e)))?;
                debug!("🦀 [REGISTRATION] Async future completed successfully");
                Ok(result)
            })();
            let _ = tx.send(result);
        });

        // Wait for result with a timeout to prevent hanging
        debug!("🦀 [REGISTRATION] Waiting for result (timeout: 5s)");
        match rx.recv_timeout(std::time::Duration::from_secs(5)) {
            Ok(result) => {
                debug!("🦀 [REGISTRATION] Received result from thread");
                result
            }
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                error!("🦀 [REGISTRATION] Timeout waiting for cortical area existence check (5s)");
                Err(FeagiDataError::InternalError(
                    "Timeout waiting for cortical area existence check (5s)".to_string(),
                ))
            }
            Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                error!(
                    "🦀 [REGISTRATION] Thread disconnected while checking cortical area existence"
                );
                Err(FeagiDataError::InternalError(
                    "Thread disconnected while checking cortical area existence".to_string(),
                ))
            }
        }
    }

    /// Check and ensure required cortical areas exist, creating missing ones if enabled
    /// Returns availability status for all required areas
    fn ensure_cortical_areas_exist(
        &self,
        capabilities: &AgentCapabilities,
    ) -> Result<CorticalAreaAvailability, FeagiDataError> {
        let mut ipu_statuses = Vec::new();
        let mut opu_statuses = Vec::new();

        // If auto-create is disabled and services are not available, assume all areas exist (for testing)
        if !self.auto_create_missing_areas {
            let genome_service_available = self.genome_service.lock().is_some();
            let connectome_service_available = self.connectome_service.lock().is_some();

            if !genome_service_available || !connectome_service_available {
                // Return empty availability (no checks performed)
                return Ok(CorticalAreaAvailability {
                    required_ipu_areas: Vec::new(),
                    required_opu_areas: Vec::new(),
                });
            }
        }

        // Get services (required for cortical area management)
        let genome_service = self
            .genome_service
            .lock()
            .as_ref()
            .ok_or_else(|| {
                FeagiDataError::InternalError(
                    "GenomeService not available - required for cortical area management"
                        .to_string(),
                )
            })?
            .clone();
        let connectome_service = self
            .connectome_service
            .lock()
            .as_ref()
            .ok_or_else(|| {
                FeagiDataError::InternalError(
                    "ConnectomeService not available - required for cortical area checking"
                        .to_string(),
                )
            })?
            .clone();

        // Handle IPU areas (from vision capabilities)
        if let Some(ref vision) = capabilities.vision {
            // Preferred Option B path: semantic unit + group, so agents don't need to know
            // internal 3-letter unit identifiers embedded in cortical IDs.
            if let (Some(unit), Some(group_index)) = (vision.unit, vision.group) {
                let group: CorticalUnitIndex = group_index.into();
                let frame_change_handling = FrameChangeHandling::Absolute;
                use feagi_structures::genomic::cortical_area::io_cortical_area_configuration_flag::PercentageNeuronPositioning;
                let percentage_neuron_positioning = PercentageNeuronPositioning::Linear; // Default

                let sensory_unit = match unit {
                    SensoryUnit::Infrared => SensoryCorticalUnit::Infrared,
                    SensoryUnit::Proximity => SensoryCorticalUnit::Proximity,
                    SensoryUnit::Shock => SensoryCorticalUnit::Shock,
                    SensoryUnit::Battery => SensoryCorticalUnit::Battery,
                    SensoryUnit::Servo => SensoryCorticalUnit::Servo,
                    SensoryUnit::AnalogGpio => SensoryCorticalUnit::AnalogGPIO,
                    SensoryUnit::DigitalGpio => SensoryCorticalUnit::DigitalGPIO,
                    SensoryUnit::MiscData => SensoryCorticalUnit::MiscData,
                    SensoryUnit::TextEnglishInput => SensoryCorticalUnit::TextEnglishInput,
                    SensoryUnit::Vision => SensoryCorticalUnit::Vision,
                    SensoryUnit::SegmentedVision => SensoryCorticalUnit::SegmentedVision,
                    SensoryUnit::Accelerometer => SensoryCorticalUnit::Accelerometer,
                    SensoryUnit::Gyroscope => SensoryCorticalUnit::Gyroscope,
                };

                // Generate the complete set of cortical IDs for this unit type.
                let cortical_ids = self.get_all_cortical_ids_for_unit(
                    sensory_unit,
                    frame_change_handling,
                    percentage_neuron_positioning,
                    group,
                )?;

                let topology = sensory_unit.get_unit_default_topology();

                for (i, cortical_id) in cortical_ids.iter().enumerate() {
                    let cortical_id_base64 = cortical_id.as_base_64();
                    let area_key = cortical_id_base64.clone(); // stable identifier for status reporting

                    // Determine dimensions/position from topology, else fall back to capability hints.
                    let (width, height, channels, x, y, z) = if let Some(unit_topology) =
                        topology.get(&CorticalSubUnitIndex::from(i as u8))
                    {
                        let dims = unit_topology.channel_dimensions_default;
                        let pos = unit_topology.relative_position;
                        (
                            dims[0] as usize,
                            dims[1] as usize,
                            dims[2] as usize,
                            pos[0],
                            pos[1],
                            pos[2],
                        )
                    } else {
                        (
                            vision.dimensions.0,
                            vision.dimensions.1,
                            vision.channels,
                            0,
                            0,
                            0,
                        )
                    };

                    // Check existence (blocking call via helper thread).
                    let id_for_check = cortical_id_base64.clone();
                    let connectome_service_clone = connectome_service.clone();
                    let exists = self
                        .block_on_async_service(move || {
                            let service = connectome_service_clone.clone();
                            let id = id_for_check.clone();
                            Box::pin(async move { service.cortical_area_exists(&id).await })
                        })
                        .map_err(|e| {
                            FeagiDataError::InternalError(format!(
                                "Failed to check cortical area existence for IPU area '{}': {}",
                                area_key, e
                            ))
                        })?;

                    if exists {
                        ipu_statuses.push(CorticalAreaStatus {
                            area_name: area_key.clone(),
                            cortical_id: cortical_id_base64,
                            status: AreaStatus::Existing,
                            dimensions: Some((width, height, channels)),
                            message: None,
                        });
                        continue;
                    }

                    if !self.auto_create_missing_areas {
                        ipu_statuses.push(CorticalAreaStatus {
                            area_name: area_key.clone(),
                            cortical_id: cortical_id_base64,
                            status: AreaStatus::Missing,
                            dimensions: None,
                            message: Some("Area missing and auto-create disabled".to_string()),
                        });
                        return Err(FeagiDataError::BadParameters(format!(
                            "Required IPU area '{}' is missing. Enable auto_create_missing_cortical_areas in config to auto-create.",
                            area_key
                        )));
                    }

                    let create_params = feagi_services::types::CreateCorticalAreaParams {
                        cortical_id: cortical_id_base64.clone(),
                        name: area_key.clone(),
                        dimensions: (width, height, channels),
                        position: (x, y, z),
                        area_type: "Sensory".to_string(),
                        visible: Some(true),
                        sub_group: None,
                        neurons_per_voxel: Some(1),
                        postsynaptic_current: None,
                        plasticity_constant: None,
                        degeneration: None,
                        psp_uniform_distribution: None,
                        firing_threshold_increment: None,
                        firing_threshold_limit: None,
                        consecutive_fire_count: None,
                        snooze_period: None,
                        refractory_period: None,
                        leak_coefficient: None,
                        leak_variability: None,
                        burst_engine_active: None,
                        properties: None,
                    };

                    let result = if let Ok(handle) = tokio::runtime::Handle::try_current() {
                        handle.block_on(genome_service.create_cortical_areas(vec![create_params]))
                    } else {
                        let rt = tokio::runtime::Runtime::new().map_err(|e| {
                            FeagiDataError::InternalError(format!(
                                "Failed to create tokio runtime: {}",
                                e
                            ))
                        })?;
                        rt.block_on(genome_service.create_cortical_areas(vec![create_params]))
                    };

                    match result {
                        Ok(_) => {
                            ipu_statuses.push(CorticalAreaStatus {
                                area_name: area_key.clone(),
                                cortical_id: cortical_id_base64,
                                status: AreaStatus::Created,
                                dimensions: Some((width, height, channels)),
                                message: Some("Auto-created during registration".to_string()),
                            });
                        }
                        Err(e) => {
                            ipu_statuses.push(CorticalAreaStatus {
                                area_name: area_key.clone(),
                                cortical_id: cortical_id_base64,
                                status: AreaStatus::Error,
                                dimensions: None,
                                message: Some(format!("Creation failed: {}", e)),
                            });
                            return Err(FeagiDataError::InternalError(format!(
                                "Failed to create IPU area '{}': {}",
                                area_key, e
                            )));
                        }
                    }
                }
            } else {
                // Legacy vision.target_cortical_area is no longer supported
                return Err(FeagiDataError::BadParameters(
                    "Legacy vision.target_cortical_area is no longer supported in FEAGI 2.0 Rust SDK. \
                    Please provide either:\n\
                    1. device_registrations in capabilities, or\n\
                    2. vision.unit and vision.group in the vision capability."
                        .to_string(),
                ));
            }
        }

        // Handle sensory IPU areas
        // Sensory capabilities are now used only for rate_hz and shm_path.
        // Device registrations are handled separately via device_registrations in capabilities.
        // Sensory.unit/sensory.group processing is handled above in the main IPU area creation logic
        // (see lines ~600-1000 where sensory units with unit/group are processed)

        // Handle OPU areas (from motor capabilities)
        if let Some(ref motor) = capabilities.motor {
            // Preferred Option B path: semantic unit + group, so agents don't need to know
            // internal 3-letter unit identifiers embedded in cortical IDs.
            let source_areas: Vec<(String, MotorCorticalUnit)> = if let Some(source_units) =
                motor.source_units.as_ref()
            {
                let frame_change_handling = FrameChangeHandling::Absolute;
                use feagi_structures::genomic::cortical_area::io_cortical_area_configuration_flag::PercentageNeuronPositioning;
                let percentage_neuron_positioning = PercentageNeuronPositioning::Linear; // Default

                let mut all_ids: Vec<(String, MotorCorticalUnit)> = Vec::new();
                for spec in source_units {
                    let group: CorticalUnitIndex = spec.group.into();
                    let motor_unit = match spec.unit {
                        MotorUnit::RotaryMotor => MotorCorticalUnit::RotaryMotor,
                        MotorUnit::PositionalServo => MotorCorticalUnit::PositionalServo,
                        MotorUnit::Gaze => MotorCorticalUnit::Gaze,
                        MotorUnit::MiscData => MotorCorticalUnit::MiscData,
                        MotorUnit::TextEnglishOutput => MotorCorticalUnit::TextEnglishOutput,
                        MotorUnit::ObjectSegmentation => MotorCorticalUnit::ObjectSegmentation,
                        MotorUnit::SimpleVisionOutput => MotorCorticalUnit::SimpleVisionOutput,
                    };

                    let cortical_ids: Vec<CorticalID> = match motor_unit {
                        MotorCorticalUnit::RotaryMotor => MotorCorticalUnit::get_cortical_ids_array_for_rotary_motor_with_parameters(
                            frame_change_handling,
                            percentage_neuron_positioning,
                            group,
                        )
                        .to_vec(),
                        MotorCorticalUnit::PositionalServo => MotorCorticalUnit::get_cortical_ids_array_for_positional_servo_with_parameters(
                            frame_change_handling,
                            percentage_neuron_positioning,
                            group,
                        )
                        .to_vec(),
                        MotorCorticalUnit::Gaze => MotorCorticalUnit::get_cortical_ids_array_for_gaze_with_parameters(
                            frame_change_handling,
                            percentage_neuron_positioning,
                            group,
                        )
                        .to_vec(),
                        MotorCorticalUnit::MiscData => MotorCorticalUnit::get_cortical_ids_array_for_misc_data_with_parameters(
                            frame_change_handling,
                            group,
                        )
                        .to_vec(),
                        MotorCorticalUnit::TextEnglishOutput => MotorCorticalUnit::get_cortical_ids_array_for_text_english_output_with_parameters(
                            frame_change_handling,
                            group,
                        )
                        .to_vec(),
                        MotorCorticalUnit::ObjectSegmentation => MotorCorticalUnit::get_cortical_ids_array_for_object_segmentation_with_parameters(
                            frame_change_handling,
                            group,
                        )
                        .to_vec(),
                        MotorCorticalUnit::SimpleVisionOutput => MotorCorticalUnit::get_cortical_ids_array_for_simple_vision_output_with_parameters(
                            frame_change_handling,
                            group,
                        )
                        .to_vec(),
                        MotorCorticalUnit::DynamicImageProcessing => MotorCorticalUnit::get_cortical_ids_array_for_dynamic_image_processing_with_parameters(
                            frame_change_handling,
                            percentage_neuron_positioning,
                            group,
                        )
                        .to_vec(),
                    };

                    all_ids.extend(
                        cortical_ids
                            .into_iter()
                            .map(|id| (id.as_base_64(), motor_unit)),
                    );
                }

                all_ids
            } else if let (Some(unit), Some(group_index)) = (motor.unit, motor.group) {
                let group: CorticalUnitIndex = group_index.into();
                let frame_change_handling = FrameChangeHandling::Absolute;
                use feagi_structures::genomic::cortical_area::io_cortical_area_configuration_flag::PercentageNeuronPositioning;
                let percentage_neuron_positioning = PercentageNeuronPositioning::Linear; // Default

                let motor_unit = match unit {
                    MotorUnit::RotaryMotor => MotorCorticalUnit::RotaryMotor,
                    MotorUnit::PositionalServo => MotorCorticalUnit::PositionalServo,
                    MotorUnit::Gaze => MotorCorticalUnit::Gaze,
                    MotorUnit::MiscData => MotorCorticalUnit::MiscData,
                    MotorUnit::TextEnglishOutput => MotorCorticalUnit::TextEnglishOutput,
                    MotorUnit::ObjectSegmentation => MotorCorticalUnit::ObjectSegmentation,
                    MotorUnit::SimpleVisionOutput => MotorCorticalUnit::SimpleVisionOutput,
                };

                let cortical_ids: Vec<CorticalID> = match motor_unit {
                    MotorCorticalUnit::RotaryMotor => MotorCorticalUnit::get_cortical_ids_array_for_rotary_motor_with_parameters(
                        frame_change_handling,
                        percentage_neuron_positioning,
                        group,
                    )
                    .to_vec(),
                    MotorCorticalUnit::PositionalServo => MotorCorticalUnit::get_cortical_ids_array_for_positional_servo_with_parameters(
                        frame_change_handling,
                        percentage_neuron_positioning,
                        group,
                    )
                    .to_vec(),
                    MotorCorticalUnit::Gaze => MotorCorticalUnit::get_cortical_ids_array_for_gaze_with_parameters(
                        frame_change_handling,
                        percentage_neuron_positioning,
                        group,
                    )
                    .to_vec(),
                    MotorCorticalUnit::MiscData => MotorCorticalUnit::get_cortical_ids_array_for_misc_data_with_parameters(
                        frame_change_handling,
                        group,
                    )
                    .to_vec(),
                    MotorCorticalUnit::TextEnglishOutput => MotorCorticalUnit::get_cortical_ids_array_for_text_english_output_with_parameters(
                        frame_change_handling,
                        group,
                    )
                    .to_vec(),
                    MotorCorticalUnit::ObjectSegmentation => MotorCorticalUnit::get_cortical_ids_array_for_object_segmentation_with_parameters(
                        frame_change_handling,
                        group,
                    )
                    .to_vec(),
                    MotorCorticalUnit::SimpleVisionOutput => MotorCorticalUnit::get_cortical_ids_array_for_simple_vision_output_with_parameters(
                        frame_change_handling,
                        group,
                    )
                    .to_vec(),
                    MotorCorticalUnit::DynamicImageProcessing => MotorCorticalUnit::get_cortical_ids_array_for_dynamic_image_processing_with_parameters(
                        frame_change_handling,
                        percentage_neuron_positioning,
                        group,
                    )
                    .to_vec(),
                };

                cortical_ids
                    .into_iter()
                    .map(|id| (id.as_base_64(), motor_unit))
                    .collect()
            } else {
                // Legacy motor.source_cortical_areas are no longer supported
                return Err(FeagiDataError::BadParameters(
                    "Legacy motor.source_cortical_areas are no longer supported in FEAGI 2.0 Rust SDK. \
                    Please provide either:\n\
                    1. device_registrations in capabilities, or\n\
                    2. motor.unit and motor.group in the motor capability, or\n\
                    3. motor.source_units array with unit+group specifications."
                        .to_string(),
                ));
            };

            for (area_name, motor_unit) in &source_areas {
                let cortical_id_base64 = self.area_name_to_cortical_id(area_name)?;
                let _cortical_id =
                    CorticalID::try_from_base_64(&cortical_id_base64).map_err(|e| {
                        FeagiDataError::BadParameters(format!("Failed to parse cortical ID: {}", e))
                    })?;

                // Get per-device dimensions from motor unit topology, then scale X by device_count.
                // total_x = device_count * per_device_x
                let (dimensions, position) = {
                    let topology = motor_unit.get_unit_default_topology();
                    if let Some(unit_topology) = topology.get(&CorticalSubUnitIndex::from(0u8)) {
                        let dims = unit_topology.channel_dimensions_default;
                        let pos = unit_topology.relative_position;
                        let total_x = (dims[0] as usize).saturating_mul(motor.output_count);
                        (
                            (total_x, dims[1] as usize, dims[2] as usize),
                            (pos[0], pos[1], pos[2]),
                        )
                    } else {
                        ((motor.output_count, 1, 1), (0, 0, 0))
                    }
                };

                // Check if area exists (blocking call)
                // Use helper function to safely call async code from sync context
                debug!(
                    "🦀 [REGISTRATION] Checking OPU area existence for '{}' (cortical_id: {})",
                    area_name, cortical_id_base64
                );
                let cortical_id_clone = cortical_id_base64.clone();
                let connectome_service_clone = connectome_service.clone();
                let exists = {
                    let cortical_id = cortical_id_clone.clone();
                    self.block_on_async_service(move || {
                        let service = connectome_service_clone.clone();
                        let id = cortical_id.clone();
                        Box::pin(async move {
                            debug!("🦀 [REGISTRATION] Calling cortical_area_exists for OPU area '{}'", id);
                            let result = service.cortical_area_exists(&id).await;
                            debug!("🦀 [REGISTRATION] cortical_area_exists result for OPU area '{}': {:?}", id, result);
                            result
                        })
                    })
                }
                .map_err(|e| {
                    error!("🦀 [REGISTRATION] Failed to check OPU area existence for '{}' (cortical_id: {}): {}", area_name, cortical_id_base64, e);
                    FeagiDataError::InternalError(format!("Failed to check cortical area existence for OPU area '{}': {}", area_name, e))
                })?;
                debug!(
                    "🦀 [REGISTRATION] OPU area '{}' exists: {}",
                    area_name, exists
                );

                if exists {
                    let motor_unit_name = motor_unit.get_snake_case_name();
                    info!(
                        "🦀 [REGISTRATION] OPU area '{}' already exists (motor unit: {}, dimensions: {:?})",
                        area_name, motor_unit_name, dimensions
                    );
                    opu_statuses.push(CorticalAreaStatus {
                        area_name: area_name.clone(),
                        cortical_id: cortical_id_base64,
                        status: AreaStatus::Existing,
                        dimensions: Some(dimensions),
                        message: None,
                    });
                } else if self.auto_create_missing_areas {
                    // Create missing OPU area with dimensions from motor unit template
                    let motor_unit_name = motor_unit.get_snake_case_name();
                    info!(
                        "🦀 [REGISTRATION] Auto-creating missing OPU area '{}' (motor unit: {}, dimensions: {:?})",
                        area_name, motor_unit_name, dimensions
                    );

                    let create_params = feagi_services::types::CreateCorticalAreaParams {
                        cortical_id: cortical_id_base64.clone(),
                        name: area_name.clone(),
                        dimensions,
                        position,
                        area_type: "Motor".to_string(),
                        visible: Some(true),
                        sub_group: None,
                        neurons_per_voxel: Some(1),
                        postsynaptic_current: None,
                        plasticity_constant: None,
                        degeneration: None,
                        psp_uniform_distribution: None,
                        firing_threshold_increment: None,
                        firing_threshold_limit: None,
                        consecutive_fire_count: None,
                        snooze_period: None,
                        refractory_period: None,
                        leak_coefficient: None,
                        leak_variability: None,
                        burst_engine_active: None,
                        properties: None,
                    };

                    // Try to use current runtime handle first, fallback to creating new runtime
                    let result = if let Ok(handle) = tokio::runtime::Handle::try_current() {
                        // We're in an async context - use block_on on the handle
                        handle.block_on(genome_service.create_cortical_areas(vec![create_params]))
                    } else {
                        // Not in async context - create a new runtime
                        let rt = tokio::runtime::Runtime::new().map_err(|e| {
                            FeagiDataError::InternalError(format!(
                                "Failed to create tokio runtime: {}",
                                e
                            ))
                        })?;
                        rt.block_on(genome_service.create_cortical_areas(vec![create_params]))
                    };

                    match result {
                        Ok(_) => {
                            info!(
                                "🦀 [REGISTRATION] ✅ Successfully created OPU area '{}'",
                                area_name
                            );
                            opu_statuses.push(CorticalAreaStatus {
                                area_name: area_name.clone(),
                                cortical_id: cortical_id_base64,
                                status: AreaStatus::Created,
                                dimensions: Some(dimensions),
                                message: Some("Auto-created during registration".to_string()),
                            });
                        }
                        Err(e) => {
                            error!(
                                "🦀 [REGISTRATION] ❌ Failed to create OPU area '{}': {}",
                                area_name, e
                            );
                            opu_statuses.push(CorticalAreaStatus {
                                area_name: area_name.clone(),
                                cortical_id: cortical_id_base64,
                                status: AreaStatus::Error,
                                dimensions: None,
                                message: Some(format!("Creation failed: {}", e)),
                            });
                            return Err(FeagiDataError::InternalError(format!(
                                "Failed to create OPU area '{}': {}",
                                area_name, e
                            )));
                        }
                    }
                } else {
                    warn!(
                        "🦀 [REGISTRATION] ⚠️ OPU area '{}' is missing and auto-create is disabled",
                        area_name
                    );
                    opu_statuses.push(CorticalAreaStatus {
                        area_name: area_name.clone(),
                        cortical_id: cortical_id_base64,
                        status: AreaStatus::Missing,
                        dimensions: None,
                        message: Some("Area missing and auto-create disabled".to_string()),
                    });
                    return Err(FeagiDataError::BadParameters(format!("Required OPU area '{}' is missing. Enable auto_create_missing_cortical_areas in config to auto-create.", area_name)));
                }
            }
        }

        Ok(CorticalAreaAvailability {
            required_ipu_areas: ipu_statuses,
            required_opu_areas: opu_statuses,
        })
    }

    /// Process a registration request
    pub fn process_registration(
        &self,
        request: RegistrationRequest,
    ) -> Result<RegistrationResponse, String> {
        self.process_registration_impl(request)
            .map_err(|e| e.to_string())
    }

    /// Internal implementation
    fn process_registration_impl(
        &self,
        request: RegistrationRequest,
    ) -> Result<RegistrationResponse, FeagiDataError> {
        let total_start = std::time::Instant::now();
        info!(
            "🦀 [REGISTRATION] 🔍 [LOCK-TRACE] Processing registration for agent: {} (type: {})",
            request.agent_id, request.agent_type
        );

        // Validate requested transport if provided
        if let Some(ref requested_transport) = request.chosen_transport {
            info!(
                "🦀 [REGISTRATION] Agent requests transport: {}",
                requested_transport
            );

            match requested_transport.as_str() {
                "websocket" => {
                    if !self.ws_enabled {
                        return Err(FeagiDataError::BadParameters(format!(
                            "Transport '{}' not supported: WebSocket is disabled in FEAGI configuration",
                            requested_transport
                        )));
                    }
                }
                "zmq" | "shm" | "hybrid" => {
                    // ZMQ/SHM always available (for now)
                }
                other => {
                    return Err(FeagiDataError::BadParameters(format!(
                        "Transport '{}' not supported: Available transports are: zmq, websocket (if enabled), shm",
                        other
                    )));
                }
            }
            info!(
                "🦀 [REGISTRATION] ✅ Transport '{}' is supported",
                requested_transport
            );
        }

        // Parse capabilities
        let capabilities = self.parse_capabilities(&request.capabilities)?;

        // Check and ensure required cortical areas exist (auto-create if enabled)
        let cortical_areas_availability = self.ensure_cortical_areas_exist(&capabilities)?;

        // Allocate SHM paths ONLY if agent didn't explicitly choose a non-SHM transport
        // AND the agent didn't explicitly set shm_path to None (which indicates ZMQ-only)
        let mut shm_paths = HashMap::new();
        let mut allocated_capabilities = capabilities.clone();

        // Check if agent explicitly wants SHM (either via chosen_transport or by providing shm_path)
        let agent_wants_shm = match request.chosen_transport.as_deref() {
            Some("websocket") | Some("zmq") => false, // Agent explicitly chose non-SHM transport
            Some("shm") | Some("hybrid") => true,     // Agent explicitly wants SHM
            None => {
                // If agent didn't specify transport, check if they provided shm_path
                // If shm_path is None, agent is using ZMQ-only (don't auto-allocate SHM)
                // If shm_path is Some, agent wants SHM (or already has a path)
                capabilities
                    .sensory
                    .as_ref()
                    .and_then(|s| s.shm_path.as_ref())
                    .is_some()
            }
            Some(_) => false, // Unknown transport, don't offer SHM
        };

        if agent_wants_shm {
            if let Some(ref mut sensory) = allocated_capabilities.sensory {
                // Only auto-allocate if agent didn't already provide a path
                if sensory.shm_path.is_none() {
                    let shm_path = format!(
                        "{}/feagi-shm-{}-sensory.bin",
                        self.shm_base_path, request.agent_id
                    );
                    sensory.shm_path = Some(shm_path.clone());
                    shm_paths.insert("sensory".to_string(), shm_path);
                } else {
                    // Agent provided a path, use it
                    if let Some(ref path) = sensory.shm_path {
                        shm_paths.insert("sensory".to_string(), path.clone());
                    }
                }
            }

            if allocated_capabilities.motor.is_some() {
                let shm_path = format!(
                    "{}/feagi-shm-{}-motor.bin",
                    self.shm_base_path, request.agent_id
                );
                shm_paths.insert("motor".to_string(), shm_path);
            }

            if allocated_capabilities.visualization.is_some() {
                let shm_path = format!(
                    "{}/feagi-shared-mem-visualization_stream.bin",
                    self.shm_base_path
                );
                shm_paths.insert("visualization".to_string(), shm_path);
            }
        } else {
            info!(
                "🦀 [REGISTRATION] Skipping SHM paths - agent chose transport: {:?}",
                request.chosen_transport
            );
        }

        // Determine transport
        let transport = if !shm_paths.is_empty() {
            AgentTransport::Hybrid
        } else {
            AgentTransport::Zmq
        };

        // Parse agent type string to enum
        let agent_type_enum = match request.agent_type.to_lowercase().as_str() {
            "sensory" => AgentType::Sensory,
            "motor" => AgentType::Motor,
            "both" => AgentType::Both,
            "visualization" => AgentType::Visualization,
            "infrastructure" => AgentType::Infrastructure,
            _ => {
                return Err(FeagiDataError::BadParameters(format!(
                    "Invalid agent type: {}",
                    request.agent_type
                )))
            }
        };

        // Create agent info using the new constructor
        let mut agent_info = AgentInfo::new(
            request.agent_id.clone(),
            agent_type_enum,
            allocated_capabilities,
            transport,
        );

        // Store the transport the agent chose (if provided)
        if let Some(ref chosen) = request.chosen_transport {
            agent_info.chosen_transport = Some(chosen.clone());
            info!(
                "🦀 [REGISTRATION] Agent '{}' chose transport: {}",
                request.agent_id, chosen
            );
        }

        // Register in registry
        info!(
            "🦀 [REGISTRATION] 🔍 Registering agent '{}' in AgentRegistry...",
            request.agent_id
        );
        self.agent_registry
            .write()
            .register(agent_info.clone())
            .map_err(|e| {
                FeagiDataError::InternalError(format!("Failed to register agent: {}", e))
            })?;

        // Verify registration
        let registry_count = self.agent_registry.read().get_all().len();
        let all_agents: Vec<String> = self
            .agent_registry
            .read()
            .get_all()
            .iter()
            .map(|a| a.agent_id.clone())
            .collect();
        info!(
            "🦀 [REGISTRATION] ✅ Agent '{}' registered in AgentRegistry (total agents: {})",
            request.agent_id, registry_count
        );
        info!("🦀 [REGISTRATION] Registry contents: {:?}", all_agents);
        info!(
            "🦀 [REGISTRATION] Registry pointer: {:p}",
            &*self.agent_registry as *const _
        );

        // Register with burst engine's sensory agent manager (if sensory capability exists AND using SHM)
        // NOTE: ZMQ-only agents are handled by the ZMQ sensory receiver, not the burst engine's SHM manager
        if let Some(ref sensory) = agent_info.capabilities.sensory {
            if let Some(sensory_mgr_lock) = self.sensory_agent_manager.lock().as_ref() {
                // Only register with burst engine if agent is using SHM (shm_path is Some)
                // ZMQ-only agents (shm_path is None) are handled by the ZMQ sensory receiver
                if let Some(shm_path) = &sensory.shm_path {
                    info!(
                        "🦀 [REGISTRATION] 🔍 [LOCK-TRACE] Registering {} with burst engine: {} @ {}Hz",
                        request.agent_id, shm_path, sensory.rate_hz
                    );

                    let burst_start = std::time::Instant::now();
                    let sensory_mgr = sensory_mgr_lock.lock().unwrap();
                    let burst_lock_duration = burst_start.elapsed();
                    info!(
                        "🦀 [REGISTRATION] 🔍 [LOCK-TRACE] Burst engine lock acquired in {:?}",
                        burst_lock_duration
                    );

                    // Build area_mapping for burst engine registration
                    // NOTE: cortical_mappings field removed from SensoryCapability.
                    // For new SDK agents using device_registrations, area_mapping will be empty.
                    // The burst engine should handle empty mappings gracefully or derive
                    // cortical IDs from device_registrations separately.
                    // TODO: Extract cortical IDs from device_registrations if available in capabilities
                    let area_mapping: HashMap<CorticalID, u32> = HashMap::new();

                    let config = feagi_npu_burst_engine::AgentConfig {
                        agent_id: request.agent_id.clone(),
                        shm_path: std::path::PathBuf::from(shm_path),
                        rate_hz: sensory.rate_hz,
                        area_mapping,
                    };

                    let register_start = std::time::Instant::now();
                    sensory_mgr.register_agent(config).map_err(|e| {
                        FeagiDataError::InternalError(format!(
                            "Failed to register with burst engine: {}",
                            e
                        ))
                    })?;
                    let register_duration = register_start.elapsed();

                    info!(
                        "🦀 [REGISTRATION] 🔍 [LOCK-TRACE] ✅ Agent {} registered with burst engine in {:?}",
                        request.agent_id, register_duration
                    );
                } else {
                    // This is expected for ZMQ-only agents - they're handled by the ZMQ sensory receiver, not the burst engine's SHM manager
                    debug!("🦀 [REGISTRATION] Agent {} using ZMQ-only transport (handled by ZMQ sensory receiver, not burst engine SHM manager)", request.agent_id);
                }
            } else {
                warn!("🦀 [REGISTRATION] ⚠️  Sensory agent manager not connected - skipping burst engine registration");
            }
        }

        // DEBUG: Log all capabilities received
        info!(
            "🦀 [REGISTRATION] 📋 DEBUG: Full capabilities for '{}': {:?}",
            request.agent_id, agent_info.capabilities
        );

        // Register motor subscriptions with burst engine (if motor capability exists)
        if let Some(ref motor) = agent_info.capabilities.motor {
            info!(
                "🦀 [REGISTRATION] 🎮 Motor capability DETECTED for '{}': modality='{}', output_count={}, source_cortical_areas={:?}",
                request.agent_id, motor.modality, motor.output_count, motor.source_cortical_areas
            );

            if let Some(burst_runner_lock) = self.burst_runner.lock().as_ref() {
                // Convert cortical area names to proper 8-byte CorticalID strings
                // SDK may send either plain names ("omot") or base64 encoded IDs ("b21vdAQAAAA=")
                use base64::{engine::general_purpose, Engine as _};
                use feagi_structures::genomic::cortical_area::CorticalID;

                let mut cortical_ids: AHashSet<String> = AHashSet::new();
                // Option B: if semantic motor units were provided, derive the cortical IDs now.
                // This ensures motor subscriptions are established even when the client doesn't
                // know or send internal 3-letter unit identifiers.
                let motor_source_inputs: Vec<String> = if let Some(source_units) =
                    motor.source_units.as_ref()
                {
                    let frame_change_handling = FrameChangeHandling::Absolute;
                    use feagi_structures::genomic::cortical_area::io_cortical_area_configuration_flag::PercentageNeuronPositioning;
                    let percentage_neuron_positioning = PercentageNeuronPositioning::Linear;

                    let mut out: Vec<String> = Vec::new();
                    for spec in source_units {
                        let group: CorticalUnitIndex = spec.group.into();
                        let motor_unit = match spec.unit {
                            MotorUnit::RotaryMotor => MotorCorticalUnit::RotaryMotor,
                            MotorUnit::PositionalServo => MotorCorticalUnit::PositionalServo,
                            MotorUnit::Gaze => MotorCorticalUnit::Gaze,
                            MotorUnit::MiscData => MotorCorticalUnit::MiscData,
                            MotorUnit::TextEnglishOutput => MotorCorticalUnit::TextEnglishOutput,
                            MotorUnit::ObjectSegmentation => MotorCorticalUnit::ObjectSegmentation,
                            MotorUnit::SimpleVisionOutput => MotorCorticalUnit::SimpleVisionOutput,
                        };

                        let cortical_ids_for_unit: Vec<CorticalID> = match motor_unit {
                            MotorCorticalUnit::RotaryMotor => {
                                MotorCorticalUnit::get_cortical_ids_array_for_rotary_motor_with_parameters(
                                    frame_change_handling,
                                    percentage_neuron_positioning,
                                    group,
                                )
                                .to_vec()
                            }
                            MotorCorticalUnit::PositionalServo => {
                                MotorCorticalUnit::get_cortical_ids_array_for_positional_servo_with_parameters(
                                    frame_change_handling,
                                    percentage_neuron_positioning,
                                    group,
                                )
                                .to_vec()
                            }
                            MotorCorticalUnit::Gaze => {
                                MotorCorticalUnit::get_cortical_ids_array_for_gaze_with_parameters(
                                    frame_change_handling,
                                    percentage_neuron_positioning,
                                    group,
                                )
                                .to_vec()
                            }
                            MotorCorticalUnit::MiscData => {
                                MotorCorticalUnit::get_cortical_ids_array_for_misc_data_with_parameters(
                                    frame_change_handling,
                                    group,
                                )
                                .to_vec()
                            }
                            MotorCorticalUnit::TextEnglishOutput => {
                                MotorCorticalUnit::get_cortical_ids_array_for_text_english_output_with_parameters(
                                    frame_change_handling,
                                    group,
                                )
                                .to_vec()
                            }
                            MotorCorticalUnit::ObjectSegmentation => {
                                MotorCorticalUnit::get_cortical_ids_array_for_object_segmentation_with_parameters(
                                    frame_change_handling,
                                    group,
                                )
                                .to_vec()
                            }
                            MotorCorticalUnit::SimpleVisionOutput => {
                                MotorCorticalUnit::get_cortical_ids_array_for_simple_vision_output_with_parameters(
                                    frame_change_handling,
                                    group,
                                )
                                .to_vec()
                            }
                            MotorCorticalUnit::DynamicImageProcessing => {
                                MotorCorticalUnit::get_cortical_ids_array_for_dynamic_image_processing_with_parameters(
                                    frame_change_handling,
                                    percentage_neuron_positioning,
                                    group,
                                )
                                .to_vec()
                            }
                        };

                        out.extend(cortical_ids_for_unit.into_iter().map(|id| id.as_base_64()));
                    }
                    out
                } else if let (Some(unit), Some(group_index)) = (motor.unit, motor.group) {
                    let group: CorticalUnitIndex = group_index.into();
                    let frame_change_handling = FrameChangeHandling::Absolute;
                    use feagi_structures::genomic::cortical_area::io_cortical_area_configuration_flag::PercentageNeuronPositioning;
                    let percentage_neuron_positioning = PercentageNeuronPositioning::Linear;

                    let motor_unit = match unit {
                        MotorUnit::RotaryMotor => MotorCorticalUnit::RotaryMotor,
                        MotorUnit::PositionalServo => MotorCorticalUnit::PositionalServo,
                        MotorUnit::Gaze => MotorCorticalUnit::Gaze,
                        MotorUnit::MiscData => MotorCorticalUnit::MiscData,
                        MotorUnit::TextEnglishOutput => MotorCorticalUnit::TextEnglishOutput,
                        MotorUnit::ObjectSegmentation => MotorCorticalUnit::ObjectSegmentation,
                        MotorUnit::SimpleVisionOutput => MotorCorticalUnit::SimpleVisionOutput,
                    };

                    let cortical_ids_for_unit: Vec<CorticalID> = match motor_unit {
                        MotorCorticalUnit::RotaryMotor => {
                            MotorCorticalUnit::get_cortical_ids_array_for_rotary_motor_with_parameters(
                                frame_change_handling,
                                percentage_neuron_positioning,
                                group,
                            )
                            .to_vec()
                        }
                        MotorCorticalUnit::PositionalServo => {
                            MotorCorticalUnit::get_cortical_ids_array_for_positional_servo_with_parameters(
                                frame_change_handling,
                                percentage_neuron_positioning,
                                group,
                            )
                            .to_vec()
                        }
                        MotorCorticalUnit::Gaze => {
                            MotorCorticalUnit::get_cortical_ids_array_for_gaze_with_parameters(
                                frame_change_handling,
                                percentage_neuron_positioning,
                                group,
                            )
                            .to_vec()
                        }
                        MotorCorticalUnit::MiscData => {
                            MotorCorticalUnit::get_cortical_ids_array_for_misc_data_with_parameters(
                                frame_change_handling,
                                group,
                            )
                            .to_vec()
                        }
                        MotorCorticalUnit::TextEnglishOutput => {
                            MotorCorticalUnit::get_cortical_ids_array_for_text_english_output_with_parameters(
                                frame_change_handling,
                                group,
                            )
                            .to_vec()
                        }
                        MotorCorticalUnit::ObjectSegmentation => {
                            MotorCorticalUnit::get_cortical_ids_array_for_object_segmentation_with_parameters(
                                frame_change_handling,
                                group,
                            )
                            .to_vec()
                        }
                        MotorCorticalUnit::SimpleVisionOutput => {
                            MotorCorticalUnit::get_cortical_ids_array_for_simple_vision_output_with_parameters(
                                frame_change_handling,
                                group,
                            )
                            .to_vec()
                        }
                        MotorCorticalUnit::DynamicImageProcessing => {
                            MotorCorticalUnit::get_cortical_ids_array_for_dynamic_image_processing_with_parameters(
                                frame_change_handling,
                                percentage_neuron_positioning,
                                group,
                            )
                            .to_vec()
                        }
                    };

                    cortical_ids_for_unit
                        .into_iter()
                        .map(|id| id.as_base_64())
                        .collect()
                } else {
                    motor.source_cortical_areas.clone()
                };

                for area_input in &motor_source_inputs {
                    info!(
                        "🦀 [REGISTRATION] 🎮 Processing motor cortical area: '{}'",
                        area_input
                    );

                    // Try to parse as base64 first (if coming from Python SDK)
                    let cortical_id = if let Ok(decoded) =
                        general_purpose::STANDARD.decode(area_input)
                    {
                        info!(
                            "🦀 [REGISTRATION] 🎮   → Decoded as base64: {} bytes: {:02x?}",
                            decoded.len(),
                            decoded
                        );
                        // It's base64 - decode to get the 8-byte cortical ID
                        if decoded.len() == 8 {
                            let mut bytes = [0u8; 8];
                            bytes.copy_from_slice(&decoded);
                            match CorticalID::try_from_bytes(&bytes) {
                                Ok(id) => Some(id),
                                Err(e) => {
                                    warn!("🦀 [REGISTRATION] ⚠️   → CorticalID validation failed: {:?}", e);
                                    None
                                }
                            }
                        } else {
                            warn!(
                                "🦀 [REGISTRATION] ⚠️   → Wrong length: {} (expected 8)",
                                decoded.len()
                            );
                            None
                        }
                    } else {
                        info!("🦀 [REGISTRATION] 🎮   → Not base64, treating as plain name");
                        // It's a plain name - pad to 8 bytes with null bytes
                        let mut bytes = [b'\0'; 8];
                        let name_bytes = area_input.as_bytes();
                        let copy_len = name_bytes.len().min(8);
                        bytes[..copy_len].copy_from_slice(&name_bytes[..copy_len]);
                        info!("🦀 [REGISTRATION] 🎮   → Padded to 8 bytes: {:02x?}", bytes);
                        match CorticalID::try_from_bytes(&bytes) {
                            Ok(id) => Some(id),
                            Err(e) => {
                                warn!(
                                    "🦀 [REGISTRATION] ⚠️   → CorticalID validation failed: {:?}",
                                    e
                                );
                                None
                            }
                        }
                    };

                    if let Some(id) = cortical_id {
                        // Use to_string() to get the proper 8-byte representation
                        let full_name = id.to_string();
                        info!("🦀 [REGISTRATION] ✅ Motor subscription added: '{}' → '{}' (bytes: {:02x?})",
                              area_input, full_name.escape_debug(), id.as_bytes());
                        cortical_ids.insert(full_name);
                    } else {
                        warn!(
                            "🦀 [REGISTRATION] ❌ Failed to create cortical ID from '{}'",
                            area_input
                        );
                    }
                }

                info!(
                    "🦀 [REGISTRATION] 🎮 Registering {} cortical IDs for motor subscriptions",
                    cortical_ids.len()
                );

                burst_runner_lock
                    .read()
                    .register_motor_subscriptions(request.agent_id.clone(), cortical_ids.clone());

                info!(
                    "🦀 [REGISTRATION] ✅ Motor subscriptions CONFIRMED registered for '{}'",
                    request.agent_id
                );
            } else {
                info!("🦀 [REGISTRATION] ⚠️ Agent {} has motor capability but burst runner not connected yet", request.agent_id);
            }
        } else {
            info!(
                "🦀 [REGISTRATION] 🎮 DEBUG: Agent {} has NO motor capability in registration",
                request.agent_id
            );
        }

        // Invoke Python callback if set
        if let Some(ref callback) = *self.on_agent_registered.lock() {
            // Serialize capabilities to JSON string for Python
            let caps_json =
                serde_json::to_string(&request.capabilities).unwrap_or_else(|_| "{}".to_string());

            info!(
                "🦀 [REGISTRATION] 🔍 [LOCK-TRACE] Invoking Python callback for agent: {}",
                request.agent_id
            );
            let callback_start = std::time::Instant::now();
            callback(
                request.agent_id.clone(),
                request.agent_type.clone(),
                caps_json,
            );
            let callback_duration = callback_start.elapsed();
            info!(
                "🦀 [REGISTRATION] 🔍 [LOCK-TRACE] Python callback completed in {:?}",
                callback_duration
            );
        }

        // Invoke dynamic gating callback if set
        info!("🦀 [REGISTRATION] Checking for dynamic gating callback...");
        if let Some(ref callback) = *self.on_agent_registered_dynamic.lock() {
            info!(
                "🦀 [REGISTRATION] ✅ Dynamic gating callback found, invoking for agent: {}",
                request.agent_id
            );
            callback(request.agent_id.clone());
        } else {
            warn!("⚠️  [REGISTRATION] No dynamic gating callback set - streams won't auto-start!");
        }

        // Build transport configurations
        let mut transports = Vec::new();

        // Add ZMQ transport (always available)
        transports.push(TransportConfig {
            transport_type: "zmq".to_string(),
            enabled: true,
            ports: HashMap::from([
                ("registration".to_string(), self.registration_port),
                ("sensory".to_string(), self.sensory_port),
                ("motor".to_string(), self.motor_port),
                ("visualization".to_string(), self.viz_port),
            ]),
            host: "0.0.0.0".to_string(), // Will be overridden by actual config
        });

        // Add WebSocket transport if enabled
        if self.ws_enabled {
            transports.push(TransportConfig {
                transport_type: "websocket".to_string(),
                enabled: true,
                ports: HashMap::from([
                    ("sensory".to_string(), self.ws_sensory_port),
                    ("motor".to_string(), self.ws_motor_port),
                    ("visualization".to_string(), self.ws_viz_port),
                    ("registration".to_string(), self.ws_registration_port),
                ]),
                host: self.ws_host.clone(),
            });
        }

        // Return success response
        let total_duration = total_start.elapsed();
        info!(
            "🦀 [REGISTRATION] 🔍 [LOCK-TRACE] ✅ Total registration completed in {:?} for agent: {}",
            total_duration, request.agent_id
        );
        info!(
            "🦀 [REGISTRATION] Available transports: {} (ZMQ + {})",
            transports.len(),
            if self.ws_enabled {
                "WebSocket"
            } else {
                "no WebSocket"
            }
        );

        Ok(RegistrationResponse {
            status: "success".to_string(),
            message: Some(format!(
                "Agent {} registered successfully",
                request.agent_id
            )),
            shm_paths: if shm_paths.is_empty() {
                None
            } else {
                Some(shm_paths)
            },
            zmq_ports: Some(HashMap::from([
                ("registration".to_string(), self.registration_port),
                ("sensory".to_string(), self.sensory_port),
                ("motor".to_string(), self.motor_port),
                ("visualization".to_string(), self.viz_port),
            ])),
            transports: Some(transports),
            recommended_transport: Some("zmq".to_string()), // ZMQ is default for now
            cortical_areas: cortical_areas_availability,
        })
    }

    /// Parse capabilities from JSON
    ///
    /// Only supports the new FEAGI 2.0 Rust SDK format (AgentCapabilities struct).
    /// Legacy feagi-sensorimotor format with "input"/"output" keys is no longer supported.
    fn parse_capabilities(
        &self,
        caps_json: &serde_json::Value,
    ) -> Result<AgentCapabilities, FeagiDataError> {
        // Check for legacy format and reject it
        if caps_json.get("input").is_some() || caps_json.get("output").is_some() {
            return Err(FeagiDataError::BadParameters(
                "Legacy feagi-sensorimotor format with 'input'/'output' keys is no longer supported in FEAGI 2.0 Rust SDK. \
                Please use the new AgentCapabilities format with device_registrations, sensory.unit/sensory.group, \
                or motor.unit/motor.group.".to_string()
            ));
        }

        // Unwrap if wrapped in "capabilities" key
        let caps_json = if let Some(caps_wrapper) = caps_json.get("capabilities") {
            caps_wrapper
        } else {
            caps_json
        };

        // Deserialize directly from JSON (new agent SDK format only)
        serde_json::from_value::<AgentCapabilities>(caps_json.clone()).map_err(|e| {
            FeagiDataError::DeserializationError(format!(
                "Failed to parse capabilities as AgentCapabilities format: {}. \
                Please ensure capabilities use the new FEAGI 2.0 Rust SDK format.",
                e
            ))
        })
    }

    /// Process deregistration request
    pub fn process_deregistration(&self, agent_id: &str) -> Result<String, String> {
        // Deregister from burst engine first
        if let Some(sensory_mgr_lock) = self.sensory_agent_manager.lock().as_ref() {
            let sensory_mgr = sensory_mgr_lock.lock().unwrap();
            if let Err(e) = sensory_mgr.deregister_agent(agent_id) {
                error!(
                    "🦀 [REGISTRATION] ⚠️  Failed to deregister {} from burst engine: {}",
                    agent_id, e
                );
            } else {
                info!(
                    "🦀 [REGISTRATION] ✅ Agent {} deregistered from burst engine",
                    agent_id
                );
            }
        }

        // Deregister from registry
        let result = self
            .agent_registry
            .write()
            .deregister(agent_id)
            .map(|_| format!("Agent {} deregistered", agent_id));

        // Invoke Python callback if deregistration was successful
        if result.is_ok() {
            if let Some(ref callback) = *self.on_agent_deregistered.lock() {
                info!(
                    "🦀 [REGISTRATION] Invoking Python deregistration callback for agent: {}",
                    agent_id
                );
                callback(agent_id.to_string());
            }

            // Invoke dynamic gating callback
            if let Some(ref callback) = *self.on_agent_deregistered_dynamic.lock() {
                info!(
                    "🦀 [REGISTRATION] ✅ Invoking dynamic gating callback for deregistration: {}",
                    agent_id
                );
                callback(agent_id.to_string());
            } else {
                debug!("[REGISTRATION] No dynamic gating deregistration callback set");
            }
        }

        result
    }

    /// Process heartbeat
    pub fn process_heartbeat(&self, agent_id: &str) -> Result<String, String> {
        use tracing::{debug, warn};

        debug!("💓 [REGISTRATION] Processing heartbeat for '{}'", agent_id);

        // Check if agent exists before attempting heartbeat
        let agent_exists = {
            let registry = self.agent_registry.read();
            registry.get(agent_id).is_some()
        };

        if !agent_exists {
            // Log diagnostic information when agent not found
            let all_agents: Vec<String> = {
                let registry = self.agent_registry.read();
                registry
                    .get_all()
                    .iter()
                    .map(|a| a.agent_id.clone())
                    .collect()
            };
            warn!(
                "⚠️ [REGISTRATION] Heartbeat failed for '{}': Agent not found in registry. Registered agents ({}): {:?}",
                agent_id,
                all_agents.len(),
                all_agents
            );
            return Err(FeagiDataError::BadParameters(format!(
                "Agent {} not found in registry (total registered: {})",
                agent_id,
                all_agents.len()
            ))
            .to_string());
        }

        // Agent exists - update heartbeat
        self.agent_registry
            .write()
            .heartbeat(agent_id)
            .map(|_| {
                debug!(
                    "💓 [REGISTRATION] Heartbeat successfully recorded for '{}'",
                    agent_id
                );
                format!("Heartbeat recorded for {}", agent_id)
            })
            .map_err(|e| {
                warn!(
                    "⚠️ [REGISTRATION] Heartbeat update failed for '{}': {}",
                    agent_id, e
                );
                e
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registration_handler() {
        let registry = Arc::new(RwLock::new(AgentRegistry::with_defaults()));
        let mut handler = RegistrationHandler::new(registry.clone(), 8003, 8000, 8001, 8002);

        // Disable auto-create to avoid needing GenomeService in this unit test
        handler.set_auto_create_missing_areas(false);

        let request = RegistrationRequest {
            agent_id: "test-agent".to_string(),
            // Keep this unit test focused on the registration plumbing without requiring both
            // input+output capability validation (which would also require motor cortical area setup).
            agent_type: "sensory".to_string(),
            capabilities: serde_json::json!({
                // FEAGI 2.0 Rust SDK AgentCapabilities format
                //
                // Provide a sensory capability with a shm_path so the handler allocates/returns SHM paths
                // (this unit test asserts shm_paths is present in the response).
                "sensory": {
                    "rate_hz": 60.0,
                    "shm_path": "feagi-shm-test-agent-sensory.bin"
                }
            }),
            chosen_transport: None,
        };

        let response = handler.process_registration(request).unwrap();
        assert_eq!(response.status, "success");
        assert!(response.shm_paths.is_some());

        assert_eq!(registry.read().count(), 1);
    }
}

// Implement RegistrationHandlerTrait for RegistrationHandler
#[cfg(not(test))]
impl RegistrationHandlerTrait for RegistrationHandler {
    fn process_registration(
        &self,
        request: RegistrationRequest,
    ) -> Result<RegistrationResponse, String> {
        self.process_registration_impl(request)
            .map_err(|e| e.to_string())
    }
}
