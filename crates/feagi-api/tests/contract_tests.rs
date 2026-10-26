// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

#![allow(dead_code, unused_imports, unused_variables)] // TODO: Update tests to use new API

//! Contract Tests for FEAGI Rust API
//!
//! These tests ensure that the Rust API produces correct responses
//! and handles errors properly.
//!
//! NOTE: Currently disabled - tests need to be updated to use axum test utilities
//! instead of tower::util::ServiceExt (which requires the "util" feature).
use axum::body::Body;
use axum::http::{Request, StatusCode};
use base64::{engine::general_purpose, Engine as _};
#[cfg(feature = "feagi-agent")]
use feagi_agent::clients::{AgentRegistrationStatus, CommandControlAgent};
#[cfg(feature = "feagi-agent")]
use feagi_agent::command_and_control::FeagiMessage;
#[cfg(feature = "feagi-agent")]
use feagi_agent::server::auth::DummyAuth;
#[cfg(feature = "feagi-agent")]
use feagi_agent::server::{AgentLivenessConfig, FeagiAgentHandler};
#[cfg(feature = "feagi-agent")]
use feagi_agent::AgentCapabilities;
#[cfg(feature = "feagi-agent")]
use feagi_agent::{AgentDescriptor, AuthToken};
#[cfg(feature = "feagi-agent")]
use feagi_api::common::agent_registration::{
    auto_create_cortical_areas_from_device_registrations,
    derive_motor_cortical_ids_from_device_registrations,
    derive_sensory_cortical_ids_from_device_registrations,
};
#[cfg(feature = "feagi-agent")]
use feagi_api::common::device_registration_store::build_device_registration_store_compare;
use feagi_api::common::{Json as ApiJson, State as ApiStateExtract};
use feagi_api::endpoints::agent::register_agent;
use feagi_api::endpoints::cortical_area::delete_multi_cortical_area;
#[cfg(feature = "feagi-agent")]
use feagi_api::endpoints::genome::post_upload;
use feagi_api::transports::http::server::{create_http_server, ApiState};
use feagi_api::v1::AgentRegistrationRequest;
use feagi_brain_development::ConnectomeManager;
use feagi_evolutionary::templates::create_genome_with_core_areas;
#[cfg(feature = "feagi-agent")]
use feagi_io::protocol_implementations::websocket::websocket_std::{
    FeagiWebSocketServerPublisherProperties, FeagiWebSocketServerPullerProperties,
    FeagiWebSocketServerRouterProperties,
};
#[cfg(feature = "feagi-agent")]
use feagi_io::traits_and_enums::client::{FeagiClient, FeagiClientPusher};
#[cfg(feature = "feagi-agent")]
use feagi_io::AgentID;
use feagi_npu_burst_engine::backend::CPUBackend;
use feagi_npu_burst_engine::TracingMutex;
use feagi_npu_burst_engine::{DynamicNPU, RustNPU};
use feagi_npu_runtime::StdRuntime;
#[cfg(feature = "feagi-agent")]
use feagi_serialization::FeagiByteContainer;
use feagi_services::change_ledger::{
    ChangeLedger, ChangeLedgerConfig, RecordingConnectomeService, RecordingGenomeService,
};
use feagi_services::impls::{
    AnalyticsServiceImpl, ConnectomeServiceImpl, GenomeServiceImpl, NeuronServiceImpl,
    SystemServiceImpl,
};
use feagi_services::types::{CreateBrainRegionParams, CreateCorticalAreaParams};
#[cfg(feature = "feagi-agent")]
use feagi_services::RuntimeService;
use feagi_services::{ConnectomeService, GenomeService};
#[cfg(feature = "feagi-agent")]
use feagi_structures::genomic::cortical_area::descriptors::CorticalUnitIndex;
#[cfg(feature = "feagi-agent")]
use feagi_structures::genomic::cortical_area::io_cortical_area_configuration_flag::{
    FrameChangeHandling, IOCorticalAreaConfigurationFlag, PercentageNeuronPositioning,
};
#[cfg(feature = "feagi-agent")]
use feagi_structures::genomic::cortical_area::CorticalID;
#[cfg(feature = "feagi-agent")]
use feagi_structures::genomic::SensoryCorticalUnit;
use parking_lot::RwLock;
use serde_json::{json, Value};
use std::collections::HashMap;
#[cfg(feature = "feagi-agent")]
use std::collections::HashSet;
#[cfg(feature = "feagi-agent")]
use std::net::{Ipv4Addr, SocketAddr, TcpListener};
use std::sync::{Arc, Mutex, OnceLock};
#[cfg(feature = "feagi-agent")]
use std::time::{Duration, Instant};
use tower::ServiceExt;
// tower::util::ServiceExt requires the "util" feature which may not be enabled
// Using axum's test utilities instead

/// Build ApiState with initialized components.
/// Each test gets a fresh, isolated manager (no singleton conflicts)
/// Last frequency applied through the mock runtime (f64 bits); 0 means never set.
static MOCK_RUNTIME_FREQUENCY_BITS: std::sync::atomic::AtomicU64 =
    std::sync::atomic::AtomicU64::new(0);
use std::sync::atomic::Ordering;

/// Workspace-level `feagi_configuration.toml` checked into this repository.
fn repo_config_path() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../feagi_configuration.toml")
}

/// Point `FEAGI_CONFIG_PATH` at the repo config exactly once per process (library code
/// resolves configuration through this variable). Tests that need a different config
/// go through [`set_temp_config`], which serializes on `CONFIG_ENV_LOCK`.
fn ensure_base_config_env() {
    static INIT: std::sync::Once = std::sync::Once::new();
    INIT.call_once(|| {
        if std::env::var_os("FEAGI_CONFIG_PATH").is_none() {
            std::env::set_var("FEAGI_CONFIG_PATH", repo_config_path());
        }
    });
}

fn build_test_state() -> ApiState {
    ensure_base_config_env();

    // Initialize NPU (fire_ledger_window=10)
    let runtime = StdRuntime;
    let backend = CPUBackend::new();
    let npu_result = RustNPU::new(runtime, backend, 1_000_000, 10_000_000, 10).unwrap();
    let npu = Arc::new(TracingMutex::new(
        DynamicNPU::F32(npu_result),
        "api-contract-test-npu",
    ));

    // Create isolated ConnectomeManager for testing (bypasses singleton)
    let manager = Arc::new(RwLock::new(ConnectomeManager::new_for_testing_with_npu(
        Arc::clone(&npu),
    )));

    // Create services
    let genome_service_impl = Arc::new(GenomeServiceImpl::new(Arc::clone(&manager)));
    let current_genome = genome_service_impl.get_current_genome_arc();
    let genome_load_counter = genome_service_impl.get_genome_load_counter_arc();
    let genome_load_timestamp = genome_service_impl.get_genome_load_timestamp_arc();
    {
        let mut genome_guard = current_genome.write();
        *genome_guard = Some(create_genome_with_core_areas(
            "test-genome".to_string(),
            "test".to_string(),
        ));
    }
    let genome_service = genome_service_impl;
    let mut connectome_service_impl =
        ConnectomeServiceImpl::new(Arc::clone(&manager), current_genome.clone());
    connectome_service_impl.set_genome_load_signals(genome_load_counter, genome_load_timestamp);
    let connectome_service_impl: Arc<dyn ConnectomeService + Send + Sync> =
        Arc::new(connectome_service_impl);
    let genome_config = feagi_config::GenomeConfig::default();
    let change_ledger = Arc::new(
        ChangeLedger::new(
            ChangeLedgerConfig {
                session_capacity: genome_config.change_ledger_session_capacity,
                max_persisted_entries: genome_config.change_history_max_entries,
                max_persisted_bytes: genome_config.change_history_max_bytes,
                agent_id_max_length: genome_config.change_agent_id_max_length,
            },
            current_genome.clone(),
        )
        .expect("change ledger"),
    );
    let genome_service: Arc<dyn GenomeService + Send + Sync> =
        Arc::new(RecordingGenomeService::new(
            genome_service,
            Arc::clone(&connectome_service_impl),
            Arc::clone(&change_ledger),
        ));
    let connectome_service: Arc<dyn ConnectomeService + Send + Sync> = Arc::new(
        RecordingConnectomeService::new(connectome_service_impl, Arc::clone(&change_ledger)),
    );
    // For tests, use empty version info
    let version_info = feagi_services::types::VersionInfo::default();
    let system_service = Arc::new(SystemServiceImpl::new(
        Arc::clone(&manager),
        None, // No BurstLoopRunner for basic tests
        version_info,
    ));
    let analytics_service = Arc::new(AnalyticsServiceImpl::new(
        Arc::clone(&manager),
        None, // No BurstLoopRunner for basic tests
    ));
    let neuron_service = Arc::new(NeuronServiceImpl::new(Arc::clone(&manager)));

    // Create a mock RuntimeService that always returns NotImplemented
    // This is acceptable for tests that don't exercise runtime control
    struct MockRuntimeService;
    #[async_trait::async_trait]
    impl feagi_services::RuntimeService for MockRuntimeService {
        async fn start(&self) -> feagi_services::ServiceResult<()> {
            Err(feagi_services::ServiceError::NotImplemented(
                "MockRuntimeService".to_string(),
            ))
        }
        async fn stop(&self) -> feagi_services::ServiceResult<()> {
            Err(feagi_services::ServiceError::NotImplemented(
                "MockRuntimeService".to_string(),
            ))
        }
        async fn pause(&self) -> feagi_services::ServiceResult<()> {
            Err(feagi_services::ServiceError::NotImplemented(
                "MockRuntimeService".to_string(),
            ))
        }
        async fn resume(&self) -> feagi_services::ServiceResult<()> {
            Err(feagi_services::ServiceError::NotImplemented(
                "MockRuntimeService".to_string(),
            ))
        }
        async fn step(&self) -> feagi_services::ServiceResult<()> {
            Err(feagi_services::ServiceError::NotImplemented(
                "MockRuntimeService".to_string(),
            ))
        }
        async fn get_status(&self) -> feagi_services::ServiceResult<feagi_services::RuntimeStatus> {
            Ok(feagi_services::RuntimeStatus {
                is_running: false,
                is_paused: false,
                frequency_hz: 1000.0,
                burst_count: 0,
                current_rate_hz: 0.0,
                last_burst_neuron_count: 0,
                avg_burst_time_ms: 0.0,
            })
        }
        async fn set_frequency(&self, frequency: f64) -> feagi_services::ServiceResult<()> {
            MOCK_RUNTIME_FREQUENCY_BITS.store(frequency.to_bits(), Ordering::SeqCst);
            Ok(())
        }
        async fn get_burst_count(&self) -> feagi_services::ServiceResult<u64> {
            Ok(0)
        }
        async fn reset_burst_count(&self) -> feagi_services::ServiceResult<()> {
            Err(feagi_services::ServiceError::NotImplemented(
                "MockRuntimeService".to_string(),
            ))
        }
        async fn get_fcl_snapshot(&self) -> feagi_services::ServiceResult<Vec<(u64, f32)>> {
            Ok(vec![])
        }
        async fn get_fcl_snapshot_with_cortical_idx(
            &self,
        ) -> feagi_services::ServiceResult<Vec<(u64, u32, f32)>> {
            Ok(vec![])
        }
        async fn get_fire_queue_sample(
            &self,
        ) -> feagi_services::ServiceResult<
            std::collections::HashMap<u32, (Vec<u32>, Vec<u32>, Vec<u32>, Vec<u32>, Vec<f32>)>,
        > {
            Ok(std::collections::HashMap::new())
        }
        async fn get_fire_history(
            &self,
            areas: &[u32],
            bursts: usize,
        ) -> feagi_services::ServiceResult<feagi_services::types::FireHistory> {
            use feagi_services::types::{FireHistoryArea, FireHistoryFrame, FiredNeuron};
            Ok(feagi_services::types::FireHistory {
                current_burst: 42,
                areas: areas
                    .iter()
                    .map(|idx| FireHistoryArea {
                        cortical_idx: *idx,
                        window: bursts,
                        frames: vec![
                            FireHistoryFrame {
                                burst: 41,
                                neurons: vec![],
                            },
                            FireHistoryFrame {
                                burst: 42,
                                neurons: vec![
                                    FiredNeuron {
                                        neuron_id: 7,
                                        voxel: Some((1, 2, 3)),
                                    },
                                    FiredNeuron {
                                        neuron_id: 50_000_001,
                                        voxel: None,
                                    },
                                ],
                            },
                        ],
                    })
                    .collect(),
            })
        }
        async fn unsubscribe_fire_history(
            &self,
            areas: &[u32],
        ) -> feagi_services::ServiceResult<usize> {
            Ok(areas.len())
        }
        async fn get_fire_ledger_configs(
            &self,
        ) -> feagi_services::ServiceResult<Vec<(u32, usize)>> {
            Ok(vec![])
        }
        async fn configure_fire_ledger_window(
            &self,
            _cortical_id: u32,
            _window_size: usize,
        ) -> feagi_services::ServiceResult<()> {
            Err(feagi_services::ServiceError::NotImplemented(
                "MockRuntimeService".to_string(),
            ))
        }
        async fn get_fcl_sampler_config(&self) -> feagi_services::ServiceResult<(f64, u32)> {
            Ok((0.0, 0))
        }
        async fn set_fcl_sampler_config(
            &self,
            _sample_rate: Option<f64>,
            _max_samples: Option<u32>,
        ) -> feagi_services::ServiceResult<()> {
            Err(feagi_services::ServiceError::NotImplemented(
                "MockRuntimeService".to_string(),
            ))
        }
        async fn get_area_fcl_sample_rate(
            &self,
            _cortical_id: u32,
        ) -> feagi_services::ServiceResult<f64> {
            Ok(0.0)
        }
        async fn set_area_fcl_sample_rate(
            &self,
            _cortical_id: u32,
            _sample_rate: f64,
        ) -> feagi_services::ServiceResult<()> {
            Err(feagi_services::ServiceError::NotImplemented(
                "MockRuntimeService".to_string(),
            ))
        }
        async fn inject_sensory_by_coordinates(
            &self,
            _cortical_area_name: &str,
            _coordinates: &[(u32, u32, u32, f32)],
            _mode: feagi_services::traits::runtime_service::ManualStimulationMode,
        ) -> feagi_services::ServiceResult<usize> {
            Err(feagi_services::ServiceError::NotImplemented(
                "MockRuntimeService".to_string(),
            ))
        }

        async fn register_motor_subscriptions(
            &self,
            _agent_id: &str,
            _cortical_ids: Vec<String>,
            _rate_hz: f64,
        ) -> feagi_services::ServiceResult<()> {
            Ok(())
        }

        async fn register_visualization_subscriptions(
            &self,
            _agent_id: &str,
            _rate_hz: f64,
        ) -> feagi_services::ServiceResult<()> {
            Ok(())
        }
        fn unregister_motor_subscriptions(&self, _agent_id: &str) {}
        fn unregister_visualization_subscriptions(&self, _agent_id: &str) {}
        async fn reset_cortical_area_states(
            &self,
            cortical_indices: &[u32],
        ) -> feagi_services::ServiceResult<Vec<(u32, usize)>> {
            Ok(cortical_indices.iter().map(|&i| (i, 0)).collect())
        }
        fn clear_all_motor_subscriptions(&self) {}
        fn clear_all_visualization_subscriptions(&self) {}
    }

    let runtime_service =
        Arc::new(MockRuntimeService) as Arc<dyn feagi_services::RuntimeService + Send + Sync>;

    // Create API state
    // Get FEAGI session timestamp (when this instance started)
    let feagi_session_timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0);

    let (genome_transition_lock, genome_transition_in_progress) =
        ApiState::init_genome_transition_controls();
    let filesystem_data_root = ApiState::filesystem_data_root_from_config(std::path::Path::new(""));
    ApiState {
        network_connection_info_provider: None,
        agent_service: None,
        analytics_service,
        connectome_service,
        genome_service,
        neuron_service,
        runtime_service,
        system_service,
        feagi_session_timestamp,
        filesystem_data_root,
        memory_stats_cache: None,
        amalgamation_state: ApiState::init_amalgamation_state(),
        genome_transition_lock,
        genome_transition_in_progress,
        last_failed_mutation: ApiState::init_last_failed_mutation(),
        change_ledger: Some(change_ledger),
        #[cfg(feature = "feagi-agent")]
        agent_handler: Some(ApiState::init_agent_registration_handler()),
    }
}

#[cfg(feature = "feagi-agent")]
#[derive(Default)]
struct RuntimeTransitionTracker {
    motor_subscriptions: Mutex<HashSet<String>>,
    visualization_subscriptions: Mutex<HashSet<String>>,
    /// Every `set_frequency` call, in order (registration must never add one).
    frequency_calls: Mutex<Vec<f64>>,
}

#[cfg(feature = "feagi-agent")]
struct TrackingRuntimeService {
    tracker: Arc<RuntimeTransitionTracker>,
}

#[cfg(feature = "feagi-agent")]
impl TrackingRuntimeService {
    fn new(tracker: Arc<RuntimeTransitionTracker>) -> Self {
        Self { tracker }
    }
}

#[cfg(feature = "feagi-agent")]
#[async_trait::async_trait]
impl feagi_services::RuntimeService for TrackingRuntimeService {
    async fn start(&self) -> feagi_services::ServiceResult<()> {
        Ok(())
    }
    async fn stop(&self) -> feagi_services::ServiceResult<()> {
        Ok(())
    }
    async fn pause(&self) -> feagi_services::ServiceResult<()> {
        Ok(())
    }
    async fn resume(&self) -> feagi_services::ServiceResult<()> {
        Ok(())
    }
    async fn step(&self) -> feagi_services::ServiceResult<()> {
        Ok(())
    }
    async fn get_status(&self) -> feagi_services::ServiceResult<feagi_services::RuntimeStatus> {
        // Last applied frequency, or the 40 Hz a user dialed in before registering.
        let frequency_hz = self
            .tracker
            .frequency_calls
            .lock()
            .unwrap()
            .last()
            .copied()
            .unwrap_or(40.0);
        Ok(feagi_services::RuntimeStatus {
            is_running: false,
            is_paused: false,
            frequency_hz,
            burst_count: 0,
            current_rate_hz: 0.0,
            last_burst_neuron_count: 0,
            avg_burst_time_ms: 0.0,
        })
    }
    async fn set_frequency(&self, frequency: f64) -> feagi_services::ServiceResult<()> {
        self.tracker.frequency_calls.lock().unwrap().push(frequency);
        Ok(())
    }
    async fn get_burst_count(&self) -> feagi_services::ServiceResult<u64> {
        Ok(0)
    }
    async fn reset_burst_count(&self) -> feagi_services::ServiceResult<()> {
        Ok(())
    }
    async fn get_fcl_snapshot(&self) -> feagi_services::ServiceResult<Vec<(u64, f32)>> {
        Ok(vec![])
    }
    async fn get_fcl_snapshot_with_cortical_idx(
        &self,
    ) -> feagi_services::ServiceResult<Vec<(u64, u32, f32)>> {
        Ok(vec![])
    }
    async fn get_fire_queue_sample(
        &self,
    ) -> feagi_services::ServiceResult<
        std::collections::HashMap<u32, (Vec<u32>, Vec<u32>, Vec<u32>, Vec<u32>, Vec<f32>)>,
    > {
        Ok(std::collections::HashMap::new())
    }
    async fn get_fire_ledger_configs(&self) -> feagi_services::ServiceResult<Vec<(u32, usize)>> {
        Ok(vec![])
    }
    async fn configure_fire_ledger_window(
        &self,
        _cortical_id: u32,
        _window_size: usize,
    ) -> feagi_services::ServiceResult<()> {
        Ok(())
    }
    async fn get_fcl_sampler_config(&self) -> feagi_services::ServiceResult<(f64, u32)> {
        Ok((0.0, 0))
    }
    async fn set_fcl_sampler_config(
        &self,
        _sample_rate: Option<f64>,
        _max_samples: Option<u32>,
    ) -> feagi_services::ServiceResult<()> {
        Ok(())
    }
    async fn get_area_fcl_sample_rate(
        &self,
        _cortical_id: u32,
    ) -> feagi_services::ServiceResult<f64> {
        Ok(0.0)
    }
    async fn set_area_fcl_sample_rate(
        &self,
        _cortical_id: u32,
        _sample_rate: f64,
    ) -> feagi_services::ServiceResult<()> {
        Ok(())
    }
    async fn inject_sensory_by_coordinates(
        &self,
        _cortical_area_name: &str,
        _coordinates: &[(u32, u32, u32, f32)],
        _mode: feagi_services::traits::runtime_service::ManualStimulationMode,
    ) -> feagi_services::ServiceResult<usize> {
        Ok(0)
    }
    async fn register_motor_subscriptions(
        &self,
        agent_id: &str,
        _cortical_ids: Vec<String>,
        _rate_hz: f64,
    ) -> feagi_services::ServiceResult<()> {
        self.tracker
            .motor_subscriptions
            .lock()
            .unwrap()
            .insert(agent_id.to_string());
        Ok(())
    }
    async fn register_visualization_subscriptions(
        &self,
        agent_id: &str,
        _rate_hz: f64,
    ) -> feagi_services::ServiceResult<()> {
        self.tracker
            .visualization_subscriptions
            .lock()
            .unwrap()
            .insert(agent_id.to_string());
        Ok(())
    }
    fn unregister_motor_subscriptions(&self, agent_id: &str) {
        self.tracker
            .motor_subscriptions
            .lock()
            .unwrap()
            .remove(agent_id);
    }
    fn unregister_visualization_subscriptions(&self, agent_id: &str) {
        self.tracker
            .visualization_subscriptions
            .lock()
            .unwrap()
            .remove(agent_id);
    }
    async fn reset_cortical_area_states(
        &self,
        cortical_indices: &[u32],
    ) -> feagi_services::ServiceResult<Vec<(u32, usize)>> {
        Ok(cortical_indices.iter().map(|&i| (i, 0)).collect())
    }
    fn clear_all_motor_subscriptions(&self) {
        self.tracker.motor_subscriptions.lock().unwrap().clear();
    }
    fn clear_all_visualization_subscriptions(&self) {
        self.tracker
            .visualization_subscriptions
            .lock()
            .unwrap()
            .clear();
    }
}

#[cfg(feature = "feagi-agent")]
fn wait_for_registered_agent(
    handler: &Arc<Mutex<FeagiAgentHandler>>,
    client: &mut CommandControlAgent,
    timeout: Duration,
) -> (
    String,
    feagi_io::traits_and_enums::shared::TransportProtocolEndpoint,
) {
    let deadline = Instant::now() + timeout;
    loop {
        {
            let mut guard = handler.lock().unwrap();
            guard
                .poll_command_and_control()
                .expect("command/control polling failed");
        }
        client
            .poll_for_messages()
            .expect("client poll_for_messages failed");
        if let AgentRegistrationStatus::Registered(session_id, endpoints) =
            client.registration_status()
        {
            let sensory_endpoint = endpoints
                .get(&AgentCapabilities::SendSensorData)
                .cloned()
                .expect("registration response missing sensory endpoint");
            return (session_id.to_base64(), sensory_endpoint);
        }
        assert!(
            Instant::now() < deadline,
            "timed out waiting for agent registration"
        );
        std::thread::sleep(Duration::from_millis(2));
    }
}

#[cfg(feature = "feagi-agent")]
fn wait_for_sensory_data(handler: &Arc<Mutex<FeagiAgentHandler>>, timeout: Duration) -> bool {
    let deadline = Instant::now() + timeout;
    loop {
        {
            let mut guard = handler.lock().unwrap();
            match guard.poll_agent_sensors() {
                Ok(Some(_)) => return true,
                Ok(None) => {}
                Err(_) => return false,
            }
        }
        if Instant::now() >= deadline {
            return false;
        }
        std::thread::sleep(Duration::from_millis(2));
    }
}

#[cfg(feature = "feagi-agent")]
fn reserve_free_port() -> u16 {
    let listener = TcpListener::bind(SocketAddr::from((Ipv4Addr::LOCALHOST, 0)))
        .expect("failed to bind ephemeral port");
    listener
        .local_addr()
        .expect("failed to read local addr")
        .port()
}

#[cfg(feature = "feagi-agent")]
/// Helper to create a test server with initialized components
async fn create_test_server() -> axum::Router {
    let state = build_test_state();
    create_http_server(state)
}

#[cfg(feature = "feagi-agent")]
static CONFIG_ENV_LOCK: Mutex<()> = Mutex::new(());

/// Serializes tests that load a genome (successfully or not): every load writes the
/// process-wide `StateManager` genome state that `health_check` reports.
static GENOME_STATE_LOCK: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

/// Attach an NPU to the process-wide ConnectomeManager, which `load_genome` develops
/// into. Test states use isolated managers, so the singleton has none by default.
fn ensure_singleton_connectome_has_npu() {
    let singleton = ConnectomeManager::instance();
    let mut singleton = singleton.write();
    if singleton.get_npu().is_none() {
        let npu = RustNPU::new(StdRuntime, CPUBackend::new(), 1_000_000, 10_000_000, 10)
            .expect("singleton npu");
        singleton.set_npu(Arc::new(TracingMutex::new(
            DynamicNPU::F32(npu),
            "api-contract-test-singleton-npu",
        )));
    }
}

#[tokio::test]
async fn test_health_check_reports_genome_loaded_after_service_load() {
    let _genome_state = GENOME_STATE_LOCK.lock().await;
    ensure_singleton_connectome_has_npu();
    let state = build_test_state();

    // Before any load in this test the state is whatever the process holds; load a
    // genome through the service layer (no HTTP upload wrapper involved) and the
    // health check must report it.
    let genome = create_genome_with_core_areas(
        "health-genome-state".to_string(),
        "health-genome-state".to_string(),
    );
    let json_str = feagi_evolutionary::save_genome_to_json(&genome).expect("genome json");
    state
        .genome_service
        .load_genome(feagi_services::LoadGenomeParams { json_str })
        .await
        .expect("load genome through service");

    let (status, body) = request_json(
        create_http_server(state),
        "GET",
        "/v1/system/health_check",
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["genome_state"], "loaded", "{body}");
    assert_eq!(body["genome_loading"], false, "{body}");
    assert_eq!(body["genome_validity"], true, "{body}");
}

#[cfg(feature = "feagi-agent")]
struct ConfigEnvGuard {
    previous: Option<std::ffi::OsString>,
    path: std::path::PathBuf,
    /// Held for the whole test so no other config-mutating test overlaps.
    _lock: std::sync::MutexGuard<'static, ()>,
}

#[cfg(feature = "feagi-agent")]
impl Drop for ConfigEnvGuard {
    fn drop(&mut self) {
        if let Some(value) = self.previous.take() {
            std::env::set_var("FEAGI_CONFIG_PATH", value);
        } else {
            std::env::remove_var("FEAGI_CONFIG_PATH");
        }
        let _ = std::fs::remove_file(&self.path);
    }
}

#[cfg(feature = "feagi-agent")]
fn set_temp_config(auto_create: bool) -> ConfigEnvGuard {
    ensure_base_config_env();
    let lock = CONFIG_ENV_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("Time went backwards")
        .as_nanos();
    let path = std::env::temp_dir().join(format!("feagi-config-{nanos}--temp.toml"));
    let base_path = repo_config_path();
    let base_contents =
        std::fs::read_to_string(&base_path).expect("Failed to read base FEAGI config");
    let mut contents = String::new();
    let mut in_agent = false;
    let mut injected = false;

    for line in base_contents.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("[agent]") {
            in_agent = true;
            contents.push_str(line);
            contents.push('\n');
            continue;
        }
        if in_agent && trimmed.starts_with('[') {
            if !injected {
                contents.push_str(&format!(
                    "auto_create_missing_cortical_areas = {}\n",
                    auto_create
                ));
                injected = true;
            }
            in_agent = false;
        }
        if in_agent && trimmed.starts_with("auto_create_missing_cortical_areas") {
            contents.push_str(&format!(
                "auto_create_missing_cortical_areas = {}\n",
                auto_create
            ));
            injected = true;
            continue;
        }
        contents.push_str(line);
        contents.push('\n');
    }

    if in_agent && !injected {
        contents.push_str(&format!(
            "auto_create_missing_cortical_areas = {}\n",
            auto_create
        ));
    }

    std::fs::write(&path, contents).expect("Failed to write temp config");

    let previous = std::env::var_os("FEAGI_CONFIG_PATH");
    std::env::set_var("FEAGI_CONFIG_PATH", &path);

    ConfigEnvGuard {
        previous,
        path,
        _lock: lock,
    }
}

#[cfg(feature = "feagi-agent")]
fn sample_device_registrations() -> Value {
    json!({
        "input_units_and_encoder_properties": {
            "Vision": [
                [
                    {
                        "cortical_unit_index": 0,
                        "frame_change_handling": "Absolute",
                        "device_grouping": [{"id": 0}]
                    },
                    {}
                ]
            ]
        },
        "output_units_and_decoder_properties": {
            "RotaryMotor": [
                [
                    {
                        "cortical_unit_index": 0,
                        "frame_change_handling": "Absolute",
                        "device_grouping": [{"id": 0}]
                    },
                    {}
                ]
            ]
        },
        "feedbacks": {}
    })
}

/// Multi-limb device_registrations: 4 limbs (groups 0-3), 3 PositionalServo channels per limb.
#[cfg(feature = "feagi-agent")]
fn sample_multi_limb_device_registrations() -> Value {
    json!({
        "output_units_and_decoder_properties": {
            "PositionalServo": [
                [
                    {
                        "cortical_unit_index": 0,
                        "frame_change_handling": "Absolute",
                        "device_grouping": [{"id": 0}, {"id": 1}, {"id": 2}]
                    },
                    {}
                ],
                [
                    {
                        "cortical_unit_index": 1,
                        "frame_change_handling": "Absolute",
                        "device_grouping": [{"id": 0}, {"id": 1}, {"id": 2}]
                    },
                    {}
                ],
                [
                    {
                        "cortical_unit_index": 2,
                        "frame_change_handling": "Absolute",
                        "device_grouping": [{"id": 0}, {"id": 1}, {"id": 2}]
                    },
                    {}
                ],
                [
                    {
                        "cortical_unit_index": 3,
                        "frame_change_handling": "Absolute",
                        "device_grouping": [{"id": 0}, {"id": 1}, {"id": 2}]
                    },
                    {}
                ]
            ]
        },
        "feedbacks": {}
    })
}

#[cfg(feature = "feagi-agent")]
fn sample_named_motor_device_registrations() -> Value {
    json!({
        "output_units_and_decoder_properties": {
            "PositionalServo": [
                [
                    {
                        "friendly_name": "front_left_leg",
                        "cortical_unit_index": 0,
                        "frame_change_handling": "Absolute",
                        "device_grouping": [
                            {
                                "friendly_name": "front_left_hip",
                                "device_properties": {
                                    "bundle_id": "front_left_leg",
                                    "bundle_type": "leg"
                                }
                            },
                            {
                                "friendly_name": "front_left_knee",
                                "device_properties": {
                                    "bundle_id": "front_left_leg",
                                    "bundle_type": "leg"
                                }
                            }
                        ]
                    },
                    {}
                ]
            ]
        },
        "feedbacks": {}
    })
}

#[cfg(feature = "feagi-agent")]
fn sample_motor_device_registrations_with_io_flags() -> Value {
    json!({
        "output_units_and_decoder_properties": {
            "RotaryMotor": [
                [
                    {
                        "friendly_name": "arm_motor",
                        "cortical_unit_index": 0,
                        "device_grouping": [{"id": 0}, {"id": 1}],
                        "io_configuration_flags": {
                            "frame_change_handling": "Absolute",
                            "percentage_neuron_positioning": "Linear"
                        }
                    },
                    {}
                ]
            ]
        },
        "feedbacks": {}
    })
}

/// Grouped Servo encoder (3 joints) with the io flags a MuJoCo controller sends.
#[cfg(feature = "feagi-agent")]
fn sample_servo_sensory_device_registrations() -> Value {
    json!({
        "input_units_and_encoder_properties": {
            "Servo": [
                [
                    {
                        "friendly_name": "arm_joints",
                        "cortical_unit_index": 0,
                        "device_grouping": [{"id": 0}, {"id": 1}, {"id": 2}],
                        "io_configuration_flags": {
                            "frame_change_handling": "Absolute",
                            "percentage_neuron_positioning": "Linear"
                        }
                    },
                    {}
                ]
            ]
        },
        "output_units_and_decoder_properties": {},
        "feedbacks": {}
    })
}

/// Pre-creates the Servo IPU twice as wide as its registration declares (a user
/// widened it by hand) and returns `(cortical_id, declared_dimensions, widened_dimensions)`.
#[cfg(feature = "feagi-agent")]
async fn precreate_hand_widened_servo_area(
    state: &ApiState,
    registrations: &Value,
) -> (String, (usize, usize, usize), (usize, usize, usize)) {
    use feagi_api::common::agent_registration::diff_device_registrations;

    let declared = diff_device_registrations(state, registrations).await;
    assert_eq!(declared.len(), 1, "Servo registration resolves to one IPU");
    let area = &declared[0];
    assert!(
        area.live_dimensions.is_none(),
        "area must not exist before the test pre-creates it"
    );
    let [dx, dy, dz] = area.declared_dimensions;
    let widened = (dx * 2, dy, dz);
    let per_device = (widened.0 / area.declared_device_count, dy, dz);
    state
        .genome_service
        .create_cortical_areas(vec![CreateCorticalAreaParams {
            cortical_id: area.cortical_id.clone(),
            name: "hand_widened_servo".to_string(),
            dimensions: widened,
            position: (-50, 20, 0),
            area_type: "sensory".to_string(),
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
            properties: Some(HashMap::from([
                ("dev_count".to_string(), json!(area.declared_device_count)),
                (
                    "cortical_dimensions_per_device".to_string(),
                    json!([per_device.0, per_device.1, per_device.2]),
                ),
            ])),
        }])
        .await
        .expect("failed pre-creating widened servo area");
    (area.cortical_id.clone(), (dx, dy, dz), widened)
}

#[cfg(feature = "feagi-agent")]
fn registration_request_with(agent_id: &AgentID, registrations: Value) -> AgentRegistrationRequest {
    let mut capabilities: HashMap<String, Value> = HashMap::new();
    capabilities.insert("device_registrations".to_string(), registrations);
    AgentRegistrationRequest {
        agent_type: "embodiment".to_string(),
        agent_id: agent_id.to_base64(),
        agent_data_port: 0,
        agent_version: "0.0.0-test".to_string(),
        controller_version: "0.0.0-test".to_string(),
        capabilities,
        agent_ip: None,
        metadata: None,
        auth_token: Some(AuthToken::new([1u8; 32]).to_base64()),
        chosen_transport: None,
    }
}

#[cfg(feature = "feagi-agent")]
fn sample_multi_segmented_vision_device_registrations() -> Value {
    json!({
        "input_units_and_encoder_properties": {
            "SegmentedVision": [
                [
                    {
                        "friendly_name": "front_camera_segmented",
                        "cortical_unit_index": 0,
                        "frame_change_handling": "Absolute",
                        "device_grouping": [{"id": 0}]
                    },
                    {
                        "segment_xy_resolutions": {
                            "lower_left": {"width": 32, "height": 32},
                            "lower_middle": {"width": 32, "height": 32},
                            "lower_right": {"width": 32, "height": 32},
                            "middle_left": {"width": 32, "height": 32},
                            "center": {"width": 128, "height": 128},
                            "middle_right": {"width": 32, "height": 32},
                            "upper_left": {"width": 32, "height": 32},
                            "upper_middle": {"width": 32, "height": 32},
                            "upper_right": {"width": 32, "height": 32}
                        },
                        "center_color_channel": "RGB",
                        "peripheral_color_channels": "GrayScale"
                    }
                ],
                [
                    {
                        "friendly_name": "rear_camera_segmented",
                        "cortical_unit_index": 1,
                        "frame_change_handling": "Absolute",
                        "device_grouping": [{"id": 0}]
                    },
                    {
                        "segment_xy_resolutions": {
                            "lower_left": {"width": 32, "height": 32},
                            "lower_middle": {"width": 32, "height": 32},
                            "lower_right": {"width": 32, "height": 32},
                            "middle_left": {"width": 32, "height": 32},
                            "center": {"width": 128, "height": 128},
                            "middle_right": {"width": 32, "height": 32},
                            "upper_left": {"width": 32, "height": 32},
                            "upper_middle": {"width": 32, "height": 32},
                            "upper_right": {"width": 32, "height": 32}
                        },
                        "center_color_channel": "RGB",
                        "peripheral_color_channels": "GrayScale"
                    }
                ]
            ]
        },
        "feedbacks": {}
    })
}

#[cfg(feature = "feagi-agent")]
fn sample_depth_map_device_registrations() -> Value {
    json!({
        "input_units_and_encoder_properties": {
            "DepthMap": [
                [
                    {
                        "friendly_name": "front_depth_camera",
                        "cortical_unit_index": 0,
                        "device_grouping": [{"id": 0}],
                        "io_configuration_flags": {
                            "frame_change_handling": "Absolute"
                        }
                    },
                    {
                        "dimension_x": 64,
                        "dimension_y": 64,
                        "dimension_z": 1
                    }
                ]
            ]
        },
        "feedbacks": {}
    })
}

#[cfg(feature = "feagi-agent")]
fn sample_segmented_vision_group1_only_device_registrations() -> Value {
    json!({
        "input_units_and_encoder_properties": {
            "SegmentedVision": [
                [
                    {
                        "friendly_name": "rear_camera_segmented",
                        "cortical_unit_index": 1,
                        "frame_change_handling": "Absolute",
                        "device_grouping": [{"id": 0}]
                    },
                    {
                        "segment_xy_resolutions": {
                            "lower_left": {"width": 32, "height": 32},
                            "lower_middle": {"width": 32, "height": 32},
                            "lower_right": {"width": 32, "height": 32},
                            "middle_left": {"width": 32, "height": 32},
                            "center": {"width": 128, "height": 128},
                            "middle_right": {"width": 32, "height": 32},
                            "upper_left": {"width": 32, "height": 32},
                            "upper_middle": {"width": 32, "height": 32},
                            "upper_right": {"width": 32, "height": 32}
                        },
                        "center_color_channel": "RGB",
                        "peripheral_color_channels": "GrayScale"
                    }
                ]
            ]
        },
        "feedbacks": {}
    })
}

#[cfg(feature = "feagi-agent")]
fn sample_named_sensory_only_device_registrations() -> Value {
    json!({
        "input_units_and_encoder_properties": {
            "Vision": [
                [
                    {
                        "friendly_name": "head_camera",
                        "cortical_unit_index": 0,
                        "frame_change_handling": "Absolute",
                        "device_grouping": [
                            {
                                "friendly_name": "rgb_camera",
                                "device_properties": {
                                    "bundle_id": "head_camera",
                                    "bundle_type": "camera_rig"
                                }
                            }
                        ]
                    },
                    {}
                ]
            ]
        },
        "feedbacks": {}
    })
}

#[cfg(feature = "feagi-agent")]
fn sample_sensory_device_registrations_with_large_vision_encoder() -> Value {
    json!({
        "input_units_and_encoder_properties": {
            "Vision": [
                [
                    {
                        "friendly_name": "head_camera",
                        "cortical_unit_index": 0,
                        "frame_change_handling": "Absolute",
                        "device_grouping": [{"id": 0}]
                    },
                    {
                        "CartesianPlane": {
                            "image_resolution": {
                                "width": 128,
                                "height": 96
                            },
                            "color_space": "Gamma",
                            "color_channel_layout": "RGBA"
                        }
                    }
                ]
            ]
        },
        "feedbacks": {}
    })
}

/// Helper to make a request and get response as JSON
async fn request_json(
    app: axum::Router,
    method: &str,
    path: &str,
    body: Option<Value>,
) -> (StatusCode, Value) {
    let request_builder = Request::builder()
        .uri(path)
        .method(method)
        .header("content-type", "application/json");

    let request = if let Some(body_json) = body {
        request_builder
            .body(Body::from(serde_json::to_vec(&body_json).unwrap()))
            .unwrap()
    } else {
        request_builder.body(Body::empty()).unwrap()
    };

    let response = app.oneshot(request).await.unwrap();
    let status = response.status();

    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();

    let json: Value = if body_bytes.is_empty() {
        json!(null)
    } else {
        serde_json::from_slice(&body_bytes).unwrap_or(json!(null))
    };

    (status, json)
}

// ============================================================================
// HEALTH & SYSTEM TESTS
// ============================================================================

#[tokio::test]
async fn test_health_endpoint() {
    let app = create_test_server().await;

    let (status, response) = request_json(app, "GET", "/v1/system/health_check", None).await;

    assert_eq!(status, StatusCode::OK);
    assert!(response["status"].is_string() || !response.is_null());
}

#[tokio::test]
async fn test_modulator_by_id_routes_resolve() {
    // axum 0.7 path parameters are `:name`; a `{name}` segment is matched literally, so these
    // routes used to 404 with an empty body for every real id.
    for path in [
        "/v1/modulator/modulator/no_such_modulator",
        "/v1/modulator/usage/no_such_modulator",
    ] {
        let app = create_test_server().await;
        let (status, body) = request_json(app, "GET", path, None).await;
        assert!(
            !body.is_null(),
            "{path}: expected an API response body, got an empty router miss ({status})"
        );
    }
}

#[tokio::test]
async fn test_reset_weights_routes_return_404_for_unknown_areas() {
    let app = create_test_server().await;
    let (status, body) = request_json(
        app,
        "PUT",
        "/v1/cortical_mapping/reset_weights",
        Some(serde_json::json!({
            "src_cortical_area": "no_such_src",
            "dst_cortical_area": "no_such_dst"
        })),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND, "{body}");

    let app = create_test_server().await;
    let (status, body) = request_json(
        app,
        "PUT",
        "/v1/cortical_area/reset_weights",
        Some(serde_json::json!({ "area_list": ["no_such_area"] })),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND, "{body}");
}

#[tokio::test]
async fn test_cortical_area_read_waits_for_genome_transition() {
    let state = build_test_state();
    let transition_lock = Arc::clone(&state.genome_transition_lock);
    let transition_guard = transition_lock.lock_owned().await;
    state
        .genome_transition_in_progress
        .store(true, std::sync::atomic::Ordering::SeqCst);
    let app = create_http_server(state);

    let read_task = tokio::spawn(request_json(
        app,
        "GET",
        "/v1/cortical_area/cortical_area_name_list",
        None,
    ));
    tokio::task::yield_now().await;
    assert!(
        !read_task.is_finished(),
        "cortical-area read must wait until the active genome transition completes"
    );

    drop(transition_guard);
    let (status, _response) = read_task.await.expect("read task should complete");
    assert_eq!(status, StatusCode::OK);
}

#[tokio::test]
async fn test_system_status() {
    let app = create_test_server().await;

    let (status, response) = request_json(app, "GET", "/v1/monitoring/status", None).await;

    assert_eq!(status, StatusCode::OK);
    // Response should have burst_engine_active field
    assert!(response.is_object());
}

// ============================================================================
// AGENT REGISTRATION TESTS
// ============================================================================

#[cfg(feature = "feagi-agent")]
#[tokio::test]
async fn test_register_agent_returns_session_and_endpoints() {
    let _state = build_test_state();
    let _descriptor = AgentDescriptor::new("neuraville", "api-test", 1).unwrap();
    let agent_id = AgentID::new_random();
    let auth_token = AuthToken::new([1u8; 32]).to_base64();

    let device_registrations = json!({
        "input_units_and_encoder_properties": { "camera": [] },
        "output_units_and_decoder_properties": {},
        "feedbacks": {}
    });

    let mut capabilities: HashMap<String, Value> = HashMap::new();
    capabilities.insert("device_registrations".to_string(), device_registrations);

    let request = AgentRegistrationRequest {
        agent_type: "visualization".to_string(),
        agent_id: agent_id.to_base64(),
        agent_data_port: 0,
        agent_version: "0.0.0-test".to_string(),
        controller_version: "0.0.0-test".to_string(),
        capabilities,
        agent_ip: None,
        metadata: None,
        auth_token: Some(auth_token),
        chosen_transport: None,
    };

    let response = register_agent(ApiStateExtract(_state), ApiJson(request))
        .await
        .expect("Registration should succeed");
    let body = response.0;

    assert!(body.success);
    // The legacy `transport` field is deprecated; transport details are reported via
    // `transports` (see endpoints/agent.rs.disabled, commit 1ed916ca).
    assert!(
        body.transport.is_none(),
        "Legacy transport field must stay unset"
    );
    assert!(body.transports.is_some(), "Expected transports array");
}

#[cfg(feature = "feagi-agent")]
#[tokio::test]
async fn test_register_visualization_only_agent_returns_session_and_endpoints() {
    let _state = build_test_state();
    let agent_id = AgentID::new_random();
    let auth_token = AuthToken::new([4u8; 32]).to_base64();

    let mut capabilities: HashMap<String, Value> = HashMap::new();
    capabilities.insert(
        "visualization".to_string(),
        json!({ "visualization_type": "3d_brain", "rate_hz": 20.0, "bridge_proxy": false }),
    );

    let request = AgentRegistrationRequest {
        agent_type: "visualization".to_string(),
        agent_id: agent_id.to_base64(),
        agent_data_port: 0,
        agent_version: "0.0.0-test".to_string(),
        controller_version: "0.0.0-test".to_string(),
        capabilities,
        agent_ip: None,
        metadata: None,
        auth_token: Some(auth_token),
        chosen_transport: None,
    };

    let response = register_agent(ApiStateExtract(_state), ApiJson(request))
        .await
        .expect("Visualization-only registration should succeed");
    let body = response.0;

    assert!(body.success);
    // The legacy `transport` field is deprecated; transport details are reported via
    // `transports` (see endpoints/agent.rs.disabled, commit 1ed916ca).
    assert!(
        body.transport.is_none(),
        "Legacy transport field must stay unset"
    );
    assert!(body.transports.is_some(), "Expected transports array");
}

#[cfg(feature = "feagi-agent")]
#[tokio::test]
async fn test_register_agent_rejects_rate_above_burst_frequency() {
    let _state = build_test_state();
    let agent_id = AgentID::new_random();
    let auth_token = AuthToken::new([2u8; 32]).to_base64();

    let mut capabilities: HashMap<String, Value> = HashMap::new();
    capabilities.insert(
        "device_registrations".to_string(),
        sample_device_registrations(),
    );
    capabilities.insert("motor".to_string(), json!({ "rate_hz": 2000.0 }));

    let request = AgentRegistrationRequest {
        agent_type: "motor".to_string(),
        agent_id: agent_id.to_base64(),
        agent_data_port: 0,
        agent_version: "0.0.0-test".to_string(),
        controller_version: "0.0.0-test".to_string(),
        capabilities,
        agent_ip: None,
        metadata: None,
        auth_token: Some(auth_token),
        chosen_transport: None,
    };

    let result = register_agent(ApiStateExtract(_state), ApiJson(request)).await;
    assert!(result.is_err());
}

#[cfg(feature = "feagi-agent")]
#[tokio::test]
async fn test_register_agent_rejects_visualization_rate_above_burst_frequency() {
    let _state = build_test_state();
    let agent_id = AgentID::new_random();
    let auth_token = AuthToken::new([3u8; 32]).to_base64();

    let mut capabilities: HashMap<String, Value> = HashMap::new();
    capabilities.insert(
        "device_registrations".to_string(),
        sample_device_registrations(),
    );
    capabilities.insert("visualization".to_string(), json!({ "rate_hz": 2000.0 }));

    let request = AgentRegistrationRequest {
        agent_type: "visualization".to_string(),
        agent_id: agent_id.to_base64(),
        agent_data_port: 0,
        agent_version: "0.0.0-test".to_string(),
        controller_version: "0.0.0-test".to_string(),
        capabilities,
        agent_ip: None,
        metadata: None,
        auth_token: Some(auth_token),
        chosen_transport: None,
    };

    let result = register_agent(ApiStateExtract(_state), ApiJson(request)).await;
    assert!(result.is_err());
}

#[cfg(feature = "feagi-agent")]
#[tokio::test]
async fn test_auto_create_disabled_skips_creation() {
    let _guard = set_temp_config(false);
    let state = build_test_state();

    auto_create_cortical_areas_from_device_registrations(&state, &sample_device_registrations())
        .await;

    let areas = state
        .connectome_service
        .list_cortical_areas()
        .await
        .expect("Failed to list cortical areas");
    assert!(areas.is_empty());
}

#[cfg(feature = "feagi-agent")]
#[tokio::test]
async fn test_auto_create_enabled_creates_areas() {
    let _guard = set_temp_config(true);
    let state = build_test_state();

    auto_create_cortical_areas_from_device_registrations(&state, &sample_device_registrations())
        .await;

    let areas = state
        .connectome_service
        .list_cortical_areas()
        .await
        .expect("Failed to list cortical areas");
    assert!(!areas.is_empty());
}

#[cfg(feature = "feagi-agent")]
#[tokio::test]
async fn test_auto_create_creates_all_limb_cortical_areas() {
    let _guard = set_temp_config(true);
    let state = build_test_state();

    auto_create_cortical_areas_from_device_registrations(
        &state,
        &sample_multi_limb_device_registrations(),
    )
    .await;

    let areas = state
        .connectome_service
        .list_cortical_areas()
        .await
        .expect("Failed to list cortical areas");
    let motor_areas: Vec<_> = areas
        .iter()
        .filter(|a| a.area_type == "motor" || a.cortical_group == "OPU")
        .collect();
    assert_eq!(
        motor_areas.len(),
        12,
        "Expected 12 PositionalServo areas (abs+inc+speed per limb), got {}",
        motor_areas.len()
    );
    let mut abs_width_count = 0;
    let mut inc_width_count = 0;
    let mut speed_width_count = 0;
    for area in &motor_areas {
        if area.name.ends_with("-1") {
            assert_eq!(
                area.dimensions.0, 6,
                "Incremental subunit width should be 6"
            );
            inc_width_count += 1;
        } else if area.name.ends_with(" Speed") {
            assert_eq!(area.dimensions.0, 3, "Speed subunit width should be 3");
            speed_width_count += 1;
        } else if area.name.ends_with("-0") {
            assert_eq!(area.dimensions.0, 3, "Absolute subunit width should be 3");
            abs_width_count += 1;
        }
        assert_eq!(
            area.properties.get("dev_count").and_then(|v| v.as_u64()),
            Some(3),
            "Each limb area should have dev_count=3"
        );
    }
    assert_eq!(abs_width_count, 4, "Expected 4 absolute limb areas");
    assert_eq!(inc_width_count, 4, "Expected 4 incremental limb areas");
    assert_eq!(speed_width_count, 4, "Expected 4 speed limb areas");
}

#[cfg(feature = "feagi-agent")]
#[tokio::test]
async fn test_auto_create_places_segmented_vision_groups_horizontally_by_unit_index() {
    let _guard = set_temp_config(true);
    let state = build_test_state();
    let registrations = sample_multi_segmented_vision_device_registrations();

    auto_create_cortical_areas_from_device_registrations(&state, &registrations).await;

    let areas = state
        .connectome_service
        .list_cortical_areas()
        .await
        .expect("Failed to list cortical areas");

    let mut group0_by_subunit: HashMap<u8, (i32, i32, i32)> = HashMap::new();
    let mut group1_by_subunit: HashMap<u8, (i32, i32, i32)> = HashMap::new();

    for area in &areas {
        let Ok(cortical_id) = CorticalID::try_from_base_64(&area.cortical_id) else {
            continue;
        };
        let bytes = cortical_id.as_bytes();
        if bytes[0] != b'i' || &bytes[1..4] != b"svi" {
            continue;
        }
        let subunit_index = *cortical_id.io_cortical_sub_unit_index();
        let group_index = *cortical_id.io_cortical_unit_index();
        if group_index == 0 {
            group0_by_subunit.insert(subunit_index, area.position);
        } else if group_index == 1 {
            group1_by_subunit.insert(subunit_index, area.position);
        }
    }

    assert_eq!(
        group0_by_subunit.len(),
        9,
        "Expected 9 segmented-vision subunits for group 0"
    );
    assert_eq!(
        group1_by_subunit.len(),
        9,
        "Expected 9 segmented-vision subunits for group 1"
    );

    for subunit in 0u8..9u8 {
        let pos0 = group0_by_subunit
            .get(&subunit)
            .expect("Missing segmented-vision subunit in group 0");
        let pos1 = group1_by_subunit
            .get(&subunit)
            .expect("Missing segmented-vision subunit in group 1");

        assert_eq!(
            pos0.1, pos1.1,
            "Expected Y to match template for both segmented groups (subunit {})",
            subunit
        );
        assert_eq!(
            pos0.2, pos1.2,
            "Expected Z to match template for both segmented groups (subunit {})",
            subunit
        );
        assert!(
            pos1.0 > pos0.0,
            "Expected group 1 subunit {} to be to the right of group 0 (x1={}, x0={})",
            subunit,
            pos1.0,
            pos0.0
        );
    }

    let group0_max_x = group0_by_subunit
        .values()
        .map(|(x, _, _)| *x)
        .max()
        .expect("Group 0 should not be empty");
    let group1_min_x = group1_by_subunit
        .values()
        .map(|(x, _, _)| *x)
        .min()
        .expect("Group 1 should not be empty");
    assert!(
        group1_min_x > group0_max_x,
        "Expected segmented-vision group 1 assembly to be placed to the right of group 0 assembly"
    );
}

#[cfg(feature = "feagi-agent")]
#[tokio::test]
async fn test_auto_create_aligns_segmented_vision_yz_to_existing_scene_group() {
    let _guard = set_temp_config(true);
    let state = build_test_state();

    // Seed the scene with an existing segmented-vision group (group 0) at custom Y/Z positions.
    let mut config = serde_json::Map::new();
    config.insert(
        "frame_change_handling".to_string(),
        serde_json::to_value(FrameChangeHandling::Absolute).expect("Serialize frame handling"),
    );
    config.insert(
        "percentage_neuron_positioning".to_string(),
        serde_json::to_value(PercentageNeuronPositioning::Linear)
            .expect("Serialize percentage neuron positioning"),
    );

    let existing_ids = SensoryCorticalUnit::SegmentedVision
        .get_cortical_id_vector_from_index_and_serde_io_configuration_flags(
            CorticalUnitIndex::from(0u16),
            config,
        )
        .expect("Generate segmented-vision cortical IDs for group 0");

    let mut existing_params: Vec<CreateCorticalAreaParams> = Vec::new();
    for (subunit, cortical_id) in existing_ids.iter().enumerate() {
        let subunit_i32 = subunit as i32;
        existing_params.push(CreateCorticalAreaParams {
            cortical_id: cortical_id.as_base_64(),
            name: format!("seeded-segmented-{}", subunit),
            dimensions: if subunit == 4 {
                (128, 128, 3)
            } else {
                (32, 32, 1)
            },
            position: (-100 + subunit_i32, 200 + subunit_i32, -300 - subunit_i32),
            area_type: "sensory".to_string(),
            visible: None,
            sub_group: None,
            neurons_per_voxel: None,
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
        });
    }

    state
        .genome_service
        .create_cortical_areas(existing_params)
        .await
        .expect("Seed existing segmented-vision group");

    auto_create_cortical_areas_from_device_registrations(
        &state,
        &sample_segmented_vision_group1_only_device_registrations(),
    )
    .await;

    let areas = state
        .connectome_service
        .list_cortical_areas()
        .await
        .expect("Failed to list cortical areas");

    let mut group0_by_subunit: HashMap<u8, (i32, i32, i32)> = HashMap::new();
    let mut group1_by_subunit: HashMap<u8, (i32, i32, i32)> = HashMap::new();
    for area in &areas {
        let Ok(cortical_id) = CorticalID::try_from_base_64(&area.cortical_id) else {
            continue;
        };
        let bytes = cortical_id.as_bytes();
        if bytes[0] != b'i' || &bytes[1..4] != b"svi" {
            continue;
        }
        let subunit_index = *cortical_id.io_cortical_sub_unit_index();
        let group_index = *cortical_id.io_cortical_unit_index();
        if group_index == 0 {
            group0_by_subunit.insert(subunit_index, area.position);
        } else if group_index == 1 {
            group1_by_subunit.insert(subunit_index, area.position);
        }
    }

    assert_eq!(
        group0_by_subunit.len(),
        9,
        "Expected seeded group 0 subunits"
    );
    assert_eq!(
        group1_by_subunit.len(),
        9,
        "Expected newly created group 1 subunits"
    );

    for subunit in 0u8..9u8 {
        let pos0 = group0_by_subunit
            .get(&subunit)
            .expect("Missing seeded group 0 subunit");
        let pos1 = group1_by_subunit
            .get(&subunit)
            .expect("Missing created group 1 subunit");

        assert_eq!(
            pos1.1, pos0.1,
            "Expected group 1 subunit {} Y to align with existing segmented group",
            subunit
        );
        assert_eq!(
            pos1.2, pos0.2,
            "Expected group 1 subunit {} Z to align with existing segmented group",
            subunit
        );
        assert!(
            pos1.0 > pos0.0,
            "Expected group 1 subunit {} X to be to the right after horizontal expansion",
            subunit
        );
    }
}

#[cfg(feature = "feagi-agent")]
#[tokio::test]
async fn test_auto_create_segmented_vision_falls_back_to_template_yz_when_existing_anchor_incomplete(
) {
    let _guard = set_temp_config(true);
    let state = build_test_state();

    // Seed an INCOMPLETE existing segmented-vision group (group 0): only one subunit.
    let mut config = serde_json::Map::new();
    config.insert(
        "frame_change_handling".to_string(),
        serde_json::to_value(FrameChangeHandling::Absolute).expect("Serialize frame handling"),
    );
    config.insert(
        "percentage_neuron_positioning".to_string(),
        serde_json::to_value(PercentageNeuronPositioning::Linear)
            .expect("Serialize percentage neuron positioning"),
    );
    let existing_ids = SensoryCorticalUnit::SegmentedVision
        .get_cortical_id_vector_from_index_and_serde_io_configuration_flags(
            CorticalUnitIndex::from(0u16),
            config,
        )
        .expect("Generate segmented-vision cortical IDs for group 0");

    state
        .genome_service
        .create_cortical_areas(vec![CreateCorticalAreaParams {
            cortical_id: existing_ids[0].as_base_64(),
            name: "seeded-incomplete-segmented-0".to_string(),
            dimensions: (32, 32, 1),
            position: (999, 777, 555),
            area_type: "sensory".to_string(),
            visible: None,
            sub_group: None,
            neurons_per_voxel: None,
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
        }])
        .await
        .expect("Seed incomplete segmented-vision anchor");

    auto_create_cortical_areas_from_device_registrations(
        &state,
        &sample_segmented_vision_group1_only_device_registrations(),
    )
    .await;

    let areas = state
        .connectome_service
        .list_cortical_areas()
        .await
        .expect("Failed to list cortical areas");

    let mut group1_by_subunit: HashMap<u8, (i32, i32, i32)> = HashMap::new();
    for area in &areas {
        let Ok(cortical_id) = CorticalID::try_from_base_64(&area.cortical_id) else {
            continue;
        };
        let bytes = cortical_id.as_bytes();
        if bytes[0] != b'i' || &bytes[1..4] != b"svi" {
            continue;
        }
        if *cortical_id.io_cortical_unit_index() == 1 {
            group1_by_subunit.insert(*cortical_id.io_cortical_sub_unit_index(), area.position);
        }
    }

    assert_eq!(
        group1_by_subunit.len(),
        9,
        "Expected complete group 1 segmented-vision creation"
    );

    // Subunit 0 template-relative Y/Z for segmented vision is (-70, 0) in sensory template.
    // With an incomplete anchor, Y/Z must come from template (not the seeded 777/555).
    let subunit0 = group1_by_subunit
        .get(&0)
        .expect("Missing group 1 subunit 0");
    assert_eq!(
        subunit0.1, -70,
        "Expected template Y when existing segmented anchor is incomplete"
    );
    assert_eq!(
        subunit0.2, 0,
        "Expected template Z when existing segmented anchor is incomplete"
    );
}

#[cfg(feature = "feagi-agent")]
#[tokio::test]
async fn test_auto_create_uses_registration_friendly_name_for_motor_areas() {
    let _guard = set_temp_config(true);
    let state = build_test_state();

    auto_create_cortical_areas_from_device_registrations(
        &state,
        &sample_named_motor_device_registrations(),
    )
    .await;

    let areas = state
        .connectome_service
        .list_cortical_areas()
        .await
        .expect("Failed to list cortical areas");
    assert!(
        areas
            .iter()
            .any(|area| area.name.starts_with("front_left_leg")),
        "Expected created motor area names to use registration friendly name"
    );
}

#[cfg(feature = "feagi-agent")]
#[tokio::test]
async fn test_auto_create_sets_firing_threshold_for_simple_vision() {
    let _guard = set_temp_config(true);
    let state = build_test_state();

    auto_create_cortical_areas_from_device_registrations(&state, &sample_device_registrations())
        .await;

    let areas = state
        .connectome_service
        .list_cortical_areas()
        .await
        .expect("Failed to list cortical areas");
    let vision_area = areas
        .iter()
        .find(|area| {
            general_purpose::STANDARD
                .decode(&area.cortical_id)
                .ok()
                .is_some_and(|bytes| bytes.len() == 8 && bytes[0] == b'i' && &bytes[1..4] == b"img")
        })
        .expect("Expected simple vision area to be auto-created");

    assert_eq!(
        vision_area.firing_threshold, 150.0,
        "Expected simple vision auto-created area firing_threshold=150.0"
    );
    assert!(
        !vision_area.mp_charge_accumulation,
        "Expected simple vision auto-created area mp_charge_accumulation=false"
    );
}

#[cfg(feature = "feagi-agent")]
#[tokio::test]
async fn test_auto_create_sets_firing_threshold_for_segmented_vision() {
    let _guard = set_temp_config(true);
    let state = build_test_state();

    auto_create_cortical_areas_from_device_registrations(
        &state,
        &sample_multi_segmented_vision_device_registrations(),
    )
    .await;

    let areas = state
        .connectome_service
        .list_cortical_areas()
        .await
        .expect("Failed to list cortical areas");

    let segmented_vision_areas: Vec<_> = areas
        .iter()
        .filter(|area| {
            general_purpose::STANDARD
                .decode(&area.cortical_id)
                .ok()
                .is_some_and(|bytes| bytes.len() == 8 && bytes[0] == b'i' && &bytes[1..4] == b"svi")
        })
        .collect();

    assert!(
        !segmented_vision_areas.is_empty(),
        "Expected segmented vision areas to be auto-created"
    );
    for area in segmented_vision_areas {
        assert_eq!(
            area.firing_threshold, 150.0,
            "Expected segmented vision area {} firing_threshold=150.0",
            area.cortical_id
        );
        assert!(
            !area.mp_charge_accumulation,
            "Expected segmented vision area {} mp_charge_accumulation=false",
            area.cortical_id
        );
    }
}

#[cfg(feature = "feagi-agent")]
#[tokio::test]
async fn test_auto_create_sets_depth_map_threshold_defaults() {
    let _guard = set_temp_config(true);
    let state = build_test_state();

    let registrations = sample_depth_map_device_registrations();
    let sensory_ids = derive_sensory_cortical_ids_from_device_registrations(&registrations)
        .expect("Failed deriving sensory cortical IDs for depth map");
    let cortical_id = sensory_ids
        .iter()
        .next()
        .cloned()
        .expect("Expected derived depth map cortical ID");

    auto_create_cortical_areas_from_device_registrations(&state, &registrations).await;

    let areas = state
        .connectome_service
        .list_cortical_areas()
        .await
        .expect("Failed to list cortical areas");
    let depth_area = areas
        .iter()
        .find(|area| area.cortical_id == cortical_id)
        .expect("Expected depth map area to be auto-created");

    assert_eq!(
        depth_area.firing_threshold, 0.0001,
        "Expected depth map auto-created area firing_threshold=0.0001"
    );
    assert_eq!(
        depth_area.firing_threshold_increment,
        [0.0, 0.0, 0.0],
        "Depth map is one layer; depth rides on potential, not on graded Z thresholds"
    );
    assert_eq!(depth_area.dimensions.2, 1, "Depth map is one layer deep");
}

#[cfg(feature = "feagi-agent")]
#[tokio::test]
async fn test_auto_create_migrates_existing_depth_map_legacy_thresholds() {
    let _guard = set_temp_config(true);
    let state = build_test_state();
    let registrations = sample_depth_map_device_registrations();
    let sensory_ids = derive_sensory_cortical_ids_from_device_registrations(&registrations)
        .expect("Failed deriving sensory cortical IDs for depth map");
    let cortical_id = sensory_ids
        .iter()
        .next()
        .cloned()
        .expect("Expected derived depth map cortical ID");

    let mut props: HashMap<String, serde_json::Value> = HashMap::new();
    props.insert("firing_threshold".to_string(), serde_json::json!(150.0));
    props.insert(
        "firing_threshold_increment_x".to_string(),
        serde_json::json!(0.0),
    );
    props.insert(
        "firing_threshold_increment_y".to_string(),
        serde_json::json!(0.0),
    );
    props.insert(
        "firing_threshold_increment_z".to_string(),
        serde_json::json!(0.0),
    );
    props.insert("dev_count".to_string(), serde_json::json!(1));

    state
        .genome_service
        .create_cortical_areas(vec![CreateCorticalAreaParams {
            cortical_id: cortical_id.clone(),
            name: "legacy_depth".to_string(),
            dimensions: (64, 64, 64),
            position: (-140, 30, 0),
            area_type: "sensory".to_string(),
            visible: None,
            sub_group: None,
            neurons_per_voxel: None,
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
            properties: Some(props),
        }])
        .await
        .expect("Failed to pre-create legacy depth map area");

    auto_create_cortical_areas_from_device_registrations(&state, &registrations).await;

    let areas = state
        .connectome_service
        .list_cortical_areas()
        .await
        .expect("Failed to list cortical areas");
    let depth_area = areas
        .iter()
        .find(|area| area.cortical_id == cortical_id)
        .expect("Expected depth map area to exist after auto-create reconciliation");

    assert_eq!(
        depth_area.firing_threshold, 0.0001,
        "Expected legacy depth map threshold to migrate to 0.0001"
    );
    assert_eq!(
        depth_area.dimensions.2, 1,
        "Expected legacy binned depth map to be resized to one layer"
    );
}

#[cfg(feature = "feagi-agent")]
#[tokio::test]
async fn test_auto_create_migrates_depth_map_threshold_from_legacy_35() {
    let _guard = set_temp_config(true);
    let state = build_test_state();
    let registrations = sample_depth_map_device_registrations();
    let sensory_ids = derive_sensory_cortical_ids_from_device_registrations(&registrations)
        .expect("Failed deriving sensory cortical IDs for depth map");
    let cortical_id = sensory_ids
        .iter()
        .next()
        .cloned()
        .expect("Expected derived depth map cortical ID");

    let mut props: HashMap<String, serde_json::Value> = HashMap::new();
    props.insert("firing_threshold".to_string(), serde_json::json!(35.0));
    props.insert(
        "firing_threshold_increment_x".to_string(),
        serde_json::json!(0.0),
    );
    props.insert(
        "firing_threshold_increment_y".to_string(),
        serde_json::json!(0.0),
    );
    props.insert(
        "firing_threshold_increment_z".to_string(),
        serde_json::json!(0.0),
    );
    props.insert("dev_count".to_string(), serde_json::json!(1));

    state
        .genome_service
        .create_cortical_areas(vec![CreateCorticalAreaParams {
            cortical_id: cortical_id.clone(),
            name: "legacy_depth_35".to_string(),
            dimensions: (64, 64, 64),
            position: (-140, 30, 0),
            area_type: "sensory".to_string(),
            visible: None,
            sub_group: None,
            neurons_per_voxel: None,
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
            properties: Some(props),
        }])
        .await
        .expect("Failed to pre-create legacy depth map area");

    auto_create_cortical_areas_from_device_registrations(&state, &registrations).await;

    let areas = state
        .connectome_service
        .list_cortical_areas()
        .await
        .expect("Failed to list cortical areas");
    let depth_area = areas
        .iter()
        .find(|area| area.cortical_id == cortical_id)
        .expect("Expected depth map area to exist after auto-create reconciliation");

    assert_eq!(
        depth_area.firing_threshold, 0.0001,
        "Expected legacy 35.0 depth map threshold to migrate to 0.0001"
    );
}

#[cfg(feature = "feagi-agent")]
#[tokio::test]
async fn test_auto_create_supports_sensory_only_registrations() {
    let _guard = set_temp_config(true);
    let state = build_test_state();

    auto_create_cortical_areas_from_device_registrations(
        &state,
        &sample_named_sensory_only_device_registrations(),
    )
    .await;

    let areas = state
        .connectome_service
        .list_cortical_areas()
        .await
        .expect("Failed to list cortical areas");
    assert!(
        areas.iter().any(|area| area.name == "head_camera"),
        "Expected sensory-only registration to create sensory area with registration name"
    );
}

#[cfg(feature = "feagi-agent")]
#[tokio::test]
async fn test_auto_create_updates_existing_sensory_area_dimensions_from_encoder_properties() {
    let _guard = set_temp_config(true);
    let state = build_test_state();
    let registrations = sample_sensory_device_registrations_with_large_vision_encoder();
    let sensory_ids = derive_sensory_cortical_ids_from_device_registrations(&registrations)
        .expect("Failed deriving sensory cortical IDs");
    let cortical_id = sensory_ids
        .into_iter()
        .next()
        .expect("Expected a vision cortical ID");

    // Pre-create an undersized sensory area to verify auto-create update logic expands it.
    state
        .genome_service
        .create_cortical_areas(vec![CreateCorticalAreaParams {
            cortical_id: cortical_id.clone(),
            name: "legacy_vision".to_string(),
            dimensions: (64, 64, 3),
            position: (-100, 30, 0),
            area_type: "sensory".to_string(),
            visible: None,
            sub_group: None,
            neurons_per_voxel: None,
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
        }])
        .await
        .expect("Failed to pre-create sensory area");

    auto_create_cortical_areas_from_device_registrations(&state, &registrations).await;

    let areas = state
        .connectome_service
        .list_cortical_areas()
        .await
        .expect("Failed to list cortical areas");
    let resized = areas
        .iter()
        .find(|area| area.cortical_id == cortical_id)
        .expect("Expected sensory area to exist after auto-create");

    assert_eq!(
        resized.dimensions,
        (128, 96, 4),
        "Expected sensory area dimensions to expand to encoder properties"
    );
    assert_eq!(
        resized.position,
        (-100, 30, 0),
        "Existing sensory area position must remain unchanged during auto-create reconciliation"
    );
}

#[cfg(feature = "feagi-agent")]
#[tokio::test]
async fn test_post_cortical_area_resizes_existing_segmented_tile_for_the_same_group() {
    let _guard = set_temp_config(true);
    let state = build_test_state();
    let ids = SensoryCorticalUnit::get_cortical_ids_array_for_segmented_vision_with_parameters(
        FrameChangeHandling::Absolute,
        CorticalUnitIndex::from(0u16),
    );
    let mut seed_properties = HashMap::new();
    seed_properties.insert("dev_count".to_string(), json!(1));
    seed_properties.insert(
        "cortical_dimensions_per_device".to_string(),
        json!([8, 8, 1]),
    );
    state
        .genome_service
        .create_cortical_areas(vec![CreateCorticalAreaParams {
            cortical_id: ids[0].as_base_64(),
            name: "partial-segmented".to_string(),
            dimensions: (8, 8, 1),
            position: (999, 777, 555),
            area_type: "sensory".to_string(),
            visible: None,
            sub_group: None,
            neurons_per_voxel: None,
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
            properties: Some(seed_properties),
        }])
        .await
        .expect("Failed to seed one segmented-vision tile");

    let data_type = IOCorticalAreaConfigurationFlag::CartesianPlane(FrameChangeHandling::Absolute)
        .to_data_type_configuration_flag();
    let mut dimensions_by_subunit = serde_json::Map::new();
    let mut data_type_configs_by_subunit = serde_json::Map::new();
    for index in 0..9 {
        let volume = if index == 4 {
            json!([100, 80, 3])
        } else {
            json!([40, 24, 1])
        };
        dimensions_by_subunit.insert(index.to_string(), volume);
        data_type_configs_by_subunit.insert(index.to_string(), json!(data_type));
    }
    let mut request = HashMap::new();
    request.insert("cortical_id".to_string(), json!("isvi"));
    request.insert("cortical_type".to_string(), json!("IPU"));
    request.insert("group_id".to_string(), json!(0));
    request.insert("device_count".to_string(), json!(1));
    request.insert("coordinates_3d".to_string(), json!([0, 0, 0]));
    request.insert(
        "per_device_dimensions_by_subunit".to_string(),
        Value::Object(dimensions_by_subunit),
    );
    request.insert(
        "data_type_configs_by_subunit".to_string(),
        Value::Object(data_type_configs_by_subunit),
    );

    let _response = feagi_api::endpoints::cortical_area::post_cortical_area(
        ApiStateExtract(state.clone()),
        ApiJson(request),
    )
    .await
    .expect(
        "Same-group segmented vision align should resize the existing tile and create the rest",
    );

    let areas = state
        .connectome_service
        .list_cortical_areas()
        .await
        .expect("Failed to list cortical areas");
    let mut group0 = HashMap::new();
    for area in areas {
        let Ok(cortical_id) = CorticalID::try_from_base_64(&area.cortical_id) else {
            continue;
        };
        let bytes = cortical_id.as_bytes();
        if bytes[0] != b'i' || &bytes[1..4] != b"svi" {
            continue;
        }
        if *cortical_id.io_cortical_unit_index() != 0 {
            continue;
        }
        group0.insert(
            *cortical_id.io_cortical_sub_unit_index(),
            (
                area.dimensions.0,
                area.dimensions.1,
                area.dimensions.2,
                area.position,
            ),
        );
    }
    assert_eq!(group0.len(), 9, "Expected all nine tiles of group 0");
    assert_eq!(
        group0.get(&0).copied(),
        Some((40, 24, 1, (999, 777, 555))),
        "Existing tile of the targeted group must be resized without moving it"
    );
    assert_eq!(
        group0.get(&4).map(|tile| (tile.0, tile.1, tile.2)),
        Some((100, 80, 3)),
        "Missing center tile must be created at the registered size"
    );
}

#[cfg(feature = "feagi-agent")]
#[tokio::test]
async fn test_auto_create_preserves_existing_motor_area_position_while_reconciling_structure() {
    let _guard = set_temp_config(true);
    let state = build_test_state();
    let registrations = sample_motor_device_registrations_with_io_flags();
    let motor_ids = derive_motor_cortical_ids_from_device_registrations(&registrations)
        .expect("Failed deriving motor cortical IDs");
    let cortical_id = motor_ids
        .into_iter()
        .next()
        .expect("Expected at least one motor cortical ID");
    let preserved_position = (777, 888, 9);

    state
        .genome_service
        .create_cortical_areas(vec![CreateCorticalAreaParams {
            cortical_id: cortical_id.clone(),
            name: "legacy_motor".to_string(),
            dimensions: (1, 1, 1),
            position: preserved_position,
            area_type: "motor".to_string(),
            visible: None,
            sub_group: None,
            neurons_per_voxel: None,
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
        }])
        .await
        .expect("Failed to pre-create motor area");

    auto_create_cortical_areas_from_device_registrations(&state, &registrations).await;

    let areas = state
        .connectome_service
        .list_cortical_areas()
        .await
        .expect("Failed to list cortical areas");
    let reconciled = areas
        .iter()
        .find(|area| area.cortical_id == cortical_id)
        .expect("Expected motor area to exist after auto-create");

    assert_eq!(
        reconciled.position, preserved_position,
        "Existing motor area position must remain unchanged during auto-create reconciliation"
    );
    assert_ne!(
        reconciled.dimensions,
        (1, 1, 1),
        "Expected motor dimensions to reconcile from registration while preserving position"
    );
    assert_eq!(
        reconciled
            .properties
            .get("dev_count")
            .and_then(|v| v.as_u64()),
        Some(2),
        "Expected motor dev_count to reconcile from registration while preserving position"
    );
}

#[cfg(feature = "feagi-agent")]
#[tokio::test]
async fn test_reregistration_keeps_hand_widened_area_dimensions() {
    use feagi_api::endpoints::agent::get_registration_diff;
    use feagi_api::v1::RegistrationDiffQuery;

    let _guard = set_temp_config(true);
    let state = build_test_state();
    let registrations = sample_servo_sensory_device_registrations();
    let (cortical_id, declared, widened) =
        precreate_hand_widened_servo_area(&state, &registrations).await;
    assert!(widened.0 > declared.0);

    // Two registrations with fresh agent ids, like a controller restarting twice.
    for seed in [11u8, 12u8] {
        let agent_id = AgentID::new([seed; AgentID::NUMBER_BYTES]);
        let _ = register_agent(
            ApiStateExtract(state.clone()),
            ApiJson(registration_request_with(&agent_id, registrations.clone())),
        )
        .await
        .expect("registration should succeed");

        let live = state
            .connectome_service
            .get_cortical_area(&cortical_id)
            .await
            .expect("widened servo area must still exist");
        assert_eq!(
            live.dimensions, widened,
            "re-registration must not shrink a hand-widened area back to the declared {:?}",
            declared
        );
        assert_eq!(live.dev_count, Some(3));
    }

    // Diff route: declared < live, matches=false, for the last registered agent.
    let last_agent = AgentID::new([12u8; AgentID::NUMBER_BYTES]).to_base64();
    let diff = get_registration_diff(
        ApiStateExtract(state.clone()),
        feagi_api::common::Query(RegistrationDiffQuery {
            agent_id: Some(last_agent.clone()),
        }),
    )
    .await
    .expect("registration diff should succeed")
    .0;
    assert_eq!(diff.agents.len(), 1);
    assert_eq!(diff.agents[0].agent_id, last_agent);
    let area = diff.agents[0]
        .areas
        .iter()
        .find(|a| a.cortical_id == cortical_id)
        .expect("diff must list the servo IPU");
    assert_eq!(area.direction, "ipu");
    assert_eq!(
        area.declared_dimensions,
        [declared.0, declared.1, declared.2]
    );
    assert_eq!(area.declared_device_count, 3);
    let live_dims = area.live_dimensions.expect("live area exists");
    assert!(
        live_dims[0] > area.declared_dimensions[0],
        "live width {} must exceed declared {}",
        live_dims[0],
        area.declared_dimensions[0]
    );
    assert_eq!(area.live_device_count, Some(3));
    assert!(!area.matches);

    // Without agent_id every stored agent is listed.
    let all = get_registration_diff(
        ApiStateExtract(state.clone()),
        feagi_api::common::Query(RegistrationDiffQuery { agent_id: None }),
    )
    .await
    .expect("registration diff should succeed")
    .0;
    assert_eq!(all.agents.len(), 2);
}

#[cfg(feature = "feagi-agent")]
#[tokio::test]
async fn test_registration_diff_reports_missing_area_as_no_live() {
    let _guard = set_temp_config(true);
    let state = build_test_state();
    let diff = feagi_api::common::agent_registration::diff_device_registrations(
        &state,
        &sample_servo_sensory_device_registrations(),
    )
    .await;
    assert_eq!(diff.len(), 1);
    assert!(diff[0].live_dimensions.is_none());
    assert!(diff[0].live_device_count.is_none());
    assert!(!diff[0].matches);
    let areas = state
        .connectome_service
        .list_cortical_areas()
        .await
        .expect("list areas");
    assert!(
        !areas.iter().any(|a| a.cortical_id == diff[0].cortical_id),
        "diff must not create areas"
    );
}

#[cfg(feature = "feagi-agent")]
#[tokio::test]
async fn test_registration_against_existing_area_leaves_burst_frequency_unchanged() {
    let _guard = set_temp_config(true);
    let tracker = Arc::new(RuntimeTransitionTracker::default());
    let mut state = build_test_state();
    state.runtime_service = Arc::new(TrackingRuntimeService::new(Arc::clone(&tracker)))
        as Arc<dyn RuntimeService + Send + Sync>;
    let registrations = sample_servo_sensory_device_registrations();
    let _ = precreate_hand_widened_servo_area(&state, &registrations).await;

    let before = state
        .runtime_service
        .get_status()
        .await
        .expect("status")
        .frequency_hz;
    assert_eq!(before, 40.0);

    let agent_id = AgentID::new([21u8; AgentID::NUMBER_BYTES]);
    let _ = register_agent(
        ApiStateExtract(state.clone()),
        ApiJson(registration_request_with(&agent_id, registrations)),
    )
    .await
    .expect("registration should succeed");

    let after = state
        .runtime_service
        .get_status()
        .await
        .expect("status")
        .frequency_hz;
    assert_eq!(
        after, before,
        "agent registration must not touch the burst engine frequency"
    );
    assert!(
        tracker.frequency_calls.lock().unwrap().is_empty(),
        "registration must not call RuntimeService::set_frequency, got {:?}",
        tracker.frequency_calls.lock().unwrap()
    );
}

#[cfg(feature = "feagi-agent")]
#[tokio::test]
async fn test_genome_transition_clears_agents_subscriptions_and_blocks_sensory_until_reregister() {
    let _genome_state = GENOME_STATE_LOCK.lock().await;
    let tracker = Arc::new(RuntimeTransitionTracker::default());
    let runtime_service = Arc::new(TrackingRuntimeService::new(Arc::clone(&tracker)))
        as Arc<dyn RuntimeService + Send + Sync>;

    let mut state = build_test_state();
    state.runtime_service = runtime_service.clone();
    let handler = Arc::new(Mutex::new(feagi_agent::server::FeagiAgentHandler::new(
        Box::new(feagi_agent::server::auth::DummyAuth {}),
    )));
    state.agent_handler = Some(Arc::clone(&handler));

    let session_id = AgentID::new([7u8; AgentID::NUMBER_BYTES]);
    let descriptor = AgentDescriptor::new("neuraville", "transition-test-agent", 1)
        .expect("descriptor creation failed");
    {
        let mut guard = handler.lock().unwrap();
        guard.register_logical_agent(
            session_id,
            descriptor.clone(),
            vec![
                AgentCapabilities::SendSensorData,
                AgentCapabilities::ReceiveMotorData,
            ],
        );
    }
    let session_id_b64 = session_id.to_base64();
    assert_eq!(
        handler.lock().unwrap().get_all_registered_agents().len(),
        1,
        "expected an active registered agent before genome load"
    );

    runtime_service
        .register_motor_subscriptions(&session_id_b64, vec!["b21vdAUAAAA=".to_string()], 10.0)
        .await
        .expect("failed to seed motor subscriptions");
    runtime_service
        .register_visualization_subscriptions(&session_id_b64, 10.0)
        .await
        .expect("failed to seed visualization subscriptions");

    // Use a malformed genome payload so load fails fast while still exercising
    // strict transition teardown logic that runs before genome parsing/loading.
    let bad_genome_value = json!({"invalid": "payload"});
    let transition_result =
        post_upload(ApiStateExtract(state.clone()), ApiJson(bad_genome_value)).await;
    assert!(
        transition_result.is_err(),
        "expected malformed genome upload to fail"
    );

    assert_eq!(
        handler.lock().unwrap().get_all_registered_agents().len(),
        0,
        "agent list should be empty after genome transition"
    );
    assert!(
        tracker.motor_subscriptions.lock().unwrap().is_empty(),
        "motor subscriptions should be empty after genome transition"
    );
    assert!(
        tracker
            .visualization_subscriptions
            .lock()
            .unwrap()
            .is_empty(),
        "visualization subscriptions should be empty after genome transition"
    );

    // Before re-registration there is no active transport session, so handler
    // polling cannot consume sensory frames.
    {
        let mut guard = handler.lock().unwrap();
        assert!(
            guard
                .poll_agent_sensors()
                .expect("polling sensory should not fail")
                .is_none(),
            "sensory must not be consumed after genome transition before re-registration"
        );
    }

    // Re-registration path: a new logical session can be added after transition.
    let new_session_id = AgentID::new([9u8; AgentID::NUMBER_BYTES]);
    {
        let mut guard = handler.lock().unwrap();
        guard.register_logical_agent(
            new_session_id,
            descriptor,
            vec![
                AgentCapabilities::SendSensorData,
                AgentCapabilities::ReceiveMotorData,
            ],
        );
    }
    assert_eq!(
        handler.lock().unwrap().get_all_registered_agents().len(),
        1,
        "agent should reappear only after explicit re-registration"
    );
}

#[cfg(feature = "feagi-agent")]
#[tokio::test]
async fn test_force_deregister_preserves_descriptor_device_registrations_for_reconnect() {
    let mut handler = FeagiAgentHandler::new(Box::new(DummyAuth {}));
    let descriptor =
        AgentDescriptor::new("neuraville", "mujoco-agent", 1).expect("descriptor creation failed");
    let initial_session = AgentID::new([3u8; AgentID::NUMBER_BYTES]);

    handler.set_device_registrations_by_descriptor(
        initial_session.to_base64(),
        descriptor.clone(),
        json!({
            "outputs": {
                "servo": {
                    "type": "opu",
                    "properties": {"feagi_index": 0}
                }
            }
        }),
    );
    handler.register_logical_agent(
        initial_session,
        descriptor.clone(),
        vec![AgentCapabilities::ReceiveMotorData],
    );

    let removed = handler.force_deregister_all_agents("test transition");
    assert_eq!(removed.len(), 1, "expected one agent to be deregistered");
    assert!(
        handler.get_all_registered_agents().is_empty(),
        "all active agent sessions must be removed"
    );
    assert!(
        handler
            .get_device_registrations_by_descriptor(&descriptor)
            .is_some(),
        "descriptor device registrations should persist for reconnect mapping"
    );
}

#[cfg(feature = "feagi-agent")]
#[test]
fn test_device_registration_store_compare_flags_stale_segmented_vision() {
    let mut handler = FeagiAgentHandler::new(Box::new(DummyAuth {}));
    let descriptor =
        AgentDescriptor::new("neuraville", "Lite6", 1).expect("descriptor creation failed");
    let session_id = AgentID::new([5u8; AgentID::NUMBER_BYTES]);
    handler.set_device_registrations_by_descriptor(
        session_id.to_base64(),
        descriptor.clone(),
        json!({
            "input_units_and_encoder_properties": {
                "SegmentedVision": [[
                    {"friendly_name": "hand_bottom", "cortical_unit_index": 0},
                    {}
                ]]
            },
            "output_units_and_decoder_properties": {}
        }),
    );
    handler.set_device_registrations_by_agent(
        session_id,
        json!({
            "input_units_and_encoder_properties": {
                "Vision": [[
                    {"friendly_name": "scene_cam", "cortical_unit_index": 0},
                    {}
                ]]
            },
            "output_units_and_decoder_properties": {}
        }),
    );
    handler.register_logical_agent(
        session_id,
        descriptor,
        vec![AgentCapabilities::SendSensorData],
    );

    let compare = build_device_registration_store_compare(
        handler.get_all_registered_agents(),
        handler.get_device_registrations_by_descriptor_store(),
        handler.get_device_registrations_by_agent_store(),
        handler.get_descriptor_session_ids(),
    );
    assert_eq!(compare.count, 1);
    assert_eq!(compare.mismatch_count, 1);
    assert_eq!(compare.segmented_vision_descriptor_stale_count, 1);
    let row = &compare.agents[0];
    assert_eq!(row.poll_source, "descriptor");
    assert!(row.segmented_vision_only_in_descriptor);
    assert!(row
        .keys_only_in_descriptor
        .contains(&"SegmentedVision".to_string()));
    assert!(row.keys_only_in_session.contains(&"Vision".to_string()));
}

#[cfg(feature = "feagi-agent")]
#[tokio::test]
#[ignore = "real transport path can hang in CI; run manually for websocket validation"]
async fn test_genome_transition_realtime_transport_requires_reregistration() {
    let _genome_state = GENOME_STATE_LOCK.lock().await;
    let tracker = Arc::new(RuntimeTransitionTracker::default());
    let runtime_service = Arc::new(TrackingRuntimeService::new(Arc::clone(&tracker)))
        as Arc<dyn RuntimeService + Send + Sync>;

    let mut state = build_test_state();
    state.runtime_service = runtime_service.clone();
    let host = Ipv4Addr::LOCALHOST.to_string();
    let registration_port = reserve_free_port();
    let sensory_port = reserve_free_port();
    let motor_port = reserve_free_port();
    let viz_port = reserve_free_port();
    let registration_bind = format!("{host}:{registration_port}");
    let registration_remote = format!("ws://{host}:{registration_port}");
    let sensory_bind = format!("{host}:{sensory_port}");
    let sensory_remote = format!("ws://{host}:{sensory_port}");
    let motor_bind = format!("{host}:{motor_port}");
    let motor_remote = format!("ws://{host}:{motor_port}");
    let viz_bind = format!("{host}:{viz_port}");
    let viz_remote = format!("ws://{host}:{viz_port}");

    let mut test_handler = FeagiAgentHandler::new_with_liveness_config(
        Box::new(DummyAuth {}),
        AgentLivenessConfig::default(),
    );
    test_handler
        .add_and_start_command_control_server(Box::new(
            FeagiWebSocketServerRouterProperties::new_with_remote(
                &registration_bind,
                &registration_remote,
            )
            .expect("failed to create websocket router properties"),
        ))
        .expect("failed to start websocket command/control router");
    test_handler.add_puller_server(Box::new(
        FeagiWebSocketServerPullerProperties::new_with_remote(&sensory_bind, &sensory_remote)
            .expect("failed to create websocket sensory puller properties"),
    ));
    test_handler.add_publisher_server(Box::new(
        FeagiWebSocketServerPublisherProperties::new(&motor_bind, &motor_remote)
            .expect("failed to create websocket motor publisher properties"),
    ));
    test_handler.add_publisher_server(Box::new(
        FeagiWebSocketServerPublisherProperties::new(&viz_bind, &viz_remote)
            .expect("failed to create websocket viz publisher properties"),
    ));
    let handler = Arc::new(Mutex::new(test_handler));
    state.agent_handler = Some(Arc::clone(&handler));

    let registration_endpoint = {
        let guard = handler.lock().unwrap();
        guard
            .get_command_control_server_info()
            .into_iter()
            .next()
            .expect("no command/control server available")
            .get_agent_endpoint()
    };

    let mut client = CommandControlAgent::new(
        registration_endpoint
            .try_create_boxed_client_requester_properties()
            .expect("failed to create requester properties"),
    );
    client.request_connect().expect("client connect failed");
    client
        .request_registration(
            AgentDescriptor::new("neuraville", "transition-rt-agent", 1)
                .expect("descriptor creation failed"),
            AuthToken::new([0u8; 32]),
            vec![
                AgentCapabilities::SendSensorData,
                AgentCapabilities::ReceiveMotorData,
            ],
        )
        .expect("registration request failed");

    let (session_id_b64, sensory_endpoint) =
        wait_for_registered_agent(&handler, &mut client, Duration::from_secs(5));
    assert_eq!(
        handler.lock().unwrap().get_all_registered_agents().len(),
        1,
        "expected an active registered agent before genome load"
    );

    runtime_service
        .register_motor_subscriptions(&session_id_b64, vec!["b21vdAUAAAA=".to_string()], 10.0)
        .await
        .expect("failed to seed motor subscriptions");
    runtime_service
        .register_visualization_subscriptions(&session_id_b64, 10.0)
        .await
        .expect("failed to seed visualization subscriptions");

    let pusher_props = sensory_endpoint
        .try_create_boxed_client_pusher_properties()
        .expect("failed to create sensory pusher properties");
    let mut sensory_pusher = pusher_props.as_boxed_client_pusher();
    sensory_pusher
        .request_connect()
        .expect("sensory pusher connect failed");
    let pre_payload: FeagiByteContainer = FeagiMessage::HeartBeat.into();
    sensory_pusher
        .publish_data(pre_payload.get_byte_ref())
        .expect("failed to publish pre-load sensory payload");
    assert!(
        wait_for_sensory_data(&handler, Duration::from_secs(2)),
        "expected sensory data to be consumed before genome transition"
    );

    let bad_genome_value = json!({"invalid": "payload"});
    let transition_result =
        post_upload(ApiStateExtract(state.clone()), ApiJson(bad_genome_value)).await;
    assert!(
        transition_result.is_err(),
        "expected malformed genome upload to fail"
    );

    assert_eq!(
        handler.lock().unwrap().get_all_registered_agents().len(),
        0,
        "agent list should be empty after genome transition"
    );
    assert!(
        tracker.motor_subscriptions.lock().unwrap().is_empty(),
        "motor subscriptions should be empty after genome transition"
    );
    assert!(
        tracker
            .visualization_subscriptions
            .lock()
            .unwrap()
            .is_empty(),
        "visualization subscriptions should be empty after genome transition"
    );

    let post_payload: FeagiByteContainer = FeagiMessage::HeartBeat.into();
    let _ = sensory_pusher.publish_data(post_payload.get_byte_ref());
    assert!(
        !wait_for_sensory_data(&handler, Duration::from_millis(500)),
        "sensory data must not be consumed after genome transition before re-registration"
    );

    let mut reconnected_client = CommandControlAgent::new(
        registration_endpoint
            .try_create_boxed_client_requester_properties()
            .expect("failed to create requester properties for reconnect"),
    );
    reconnected_client
        .request_connect()
        .expect("reconnect client connect failed");
    reconnected_client
        .request_registration(
            AgentDescriptor::new("neuraville", "transition-rt-agent", 1)
                .expect("descriptor creation failed on reconnect"),
            AuthToken::new([0u8; 32]),
            vec![
                AgentCapabilities::SendSensorData,
                AgentCapabilities::ReceiveMotorData,
            ],
        )
        .expect("reconnect registration request failed");
    let (_new_session, new_sensory_endpoint) =
        wait_for_registered_agent(&handler, &mut reconnected_client, Duration::from_secs(5));

    let new_pusher_props = new_sensory_endpoint
        .try_create_boxed_client_pusher_properties()
        .expect("failed to create new sensory pusher properties");
    let mut new_sensory_pusher = new_pusher_props.as_boxed_client_pusher();
    new_sensory_pusher
        .request_connect()
        .expect("new sensory pusher connect failed");
    let rereg_payload: FeagiByteContainer = FeagiMessage::HeartBeat.into();
    new_sensory_pusher
        .publish_data(rereg_payload.get_byte_ref())
        .expect("failed to publish sensory payload after re-registration");
    assert!(
        wait_for_sensory_data(&handler, Duration::from_secs(2)),
        "sensory data should be consumed again after re-registration"
    );
}

// ============================================================================
// CORTICAL AREA TESTS
// ============================================================================

#[tokio::test]
async fn test_create_cortical_area_success() {
    let app = create_test_server().await;

    let create_request = json!({
        "cortical_id": "iinf",
        "cortical_type": "IPU",
        "device_count": 1,
        "coordinates_3d": [0, 0, 0],
        "data_type_configs_by_subunit": {
            "0": 0
        },
        "neurons_per_voxel": 1
    });

    let (status, response) = request_json(
        app,
        "POST",
        "/v1/cortical_area/cortical_area",
        Some(create_request),
    )
    .await;

    assert_eq!(status, StatusCode::OK, "response: {}", response);
    assert!(response.get("cortical_id").is_some());
}

#[tokio::test]
async fn test_create_cortical_area_invalid_id() {
    let app = create_test_server().await;

    // Invalid ID (not 6 characters)
    let create_request = json!({
        "cortical_id": "invalid",
        "cortical_type": "IPU",
        "device_count": 1,
        "coordinates_3d": [0, 0, 0],
        "data_type_configs_by_subunit": {
            "0": 0
        },
        "neurons_per_voxel": 1
    });

    let (status, _response) = request_json(
        app,
        "POST",
        "/v1/cortical_area/cortical_area",
        Some(create_request),
    )
    .await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_get_cortical_area_not_found() {
    let app = create_test_server().await;

    let (status, response) = request_json(
        app,
        "GET",
        "/v1/connectome/area_details?area_ids=notfnd",
        None,
    )
    .await;

    assert_eq!(status, StatusCode::OK, "response: {}", response);
    assert!(response.as_object().map(|o| o.is_empty()).unwrap_or(false));
}

#[tokio::test]
async fn test_list_cortical_areas_empty() {
    let app = create_test_server().await;

    let (status, response) =
        request_json(app, "GET", "/v1/cortical_area/cortical_area_id_list", None).await;

    assert_eq!(status, StatusCode::OK);
    let cortical_ids = response
        .get("cortical_ids")
        .and_then(|v| v.as_array())
        .expect("Expected cortical_ids array");
    assert!(cortical_ids.is_empty());
}

#[tokio::test]
async fn test_create_and_get_cortical_area() {
    let app = create_test_server().await;

    // Create
    let create_request = json!({
        "cortical_id": "iinf",
        "cortical_type": "IPU",
        "device_count": 1,
        "coordinates_3d": [0, 0, 0],
        "data_type_configs_by_subunit": {
            "0": 0
        },
        "neurons_per_voxel": 1
    });

    let (status, response) = request_json(
        app,
        "POST",
        "/v1/cortical_area/cortical_area",
        Some(create_request),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let created_id = response
        .get("cortical_id")
        .and_then(|v| v.as_str())
        .expect("Expected cortical_id in response")
        .to_string();

    // Get - need to recreate app because oneshot consumes it
    let app2 = create_test_server().await;
    let (status2, response2) = request_json(
        app2,
        "GET",
        &format!("/v1/connectome/area_details?area_ids={}", created_id),
        None,
    )
    .await;

    // Fresh manager: created area not present
    assert_eq!(status2, StatusCode::OK);
    assert!(response2.as_object().map(|o| o.is_empty()).unwrap_or(false));
}

#[tokio::test]
async fn test_io_dev_count_scales_x_and_per_device_stays_independent() {
    let state = build_test_state();
    let cortical_id = general_purpose::STANDARD.encode(*b"cDIMTEST");
    state
        .genome_service
        .create_cortical_areas(vec![CreateCorticalAreaParams {
            cortical_id: cortical_id.clone(),
            name: "io-dim-test".to_string(),
            dimensions: (1, 1, 1),
            position: (0, 0, 0),
            area_type: "Custom".to_string(),
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
        }])
        .await
        .expect("failed creating test area");

    state
        .genome_service
        .update_cortical_area(
            &cortical_id,
            HashMap::from([
                ("dev_count".to_string(), json!(3)),
                (
                    "cortical_dimensions_per_device".to_string(),
                    json!([4, 5, 7]),
                ),
            ]),
        )
        .await
        .expect("failed seeding per-device baseline");

    let updated_float_dev_count = state
        .genome_service
        .update_cortical_area(
            &cortical_id,
            HashMap::from([("dev_count".to_string(), json!(2.0))]),
        )
        .await
        .expect("failed updating float dev_count");
    assert_eq!(
        updated_float_dev_count.dimensions,
        (8, 5, 7),
        "float dev_count values should be parsed and scale only X"
    );
    assert_eq!(
        updated_float_dev_count.cortical_dimensions_per_device,
        Some((4, 5, 7)),
        "float dev_count updates must not change per-device dimensions"
    );

    let updated_dev_count = state
        .genome_service
        .update_cortical_area(
            &cortical_id,
            HashMap::from([
                ("dev_count".to_string(), json!(5)),
                ("cortical_dimensions".to_string(), json!([12, 5, 7])),
            ]),
        )
        .await
        .expect("failed updating dev_count");
    assert_eq!(
        updated_dev_count.dimensions,
        (20, 5, 7),
        "dev_count must scale only X from per-device dimensions"
    );
    assert_eq!(
        updated_dev_count.cortical_dimensions_per_device,
        Some((4, 5, 7)),
        "changing dev_count must not rewrite per-device dimensions"
    );

    let updated_per_device = state
        .genome_service
        .update_cortical_area(
            &cortical_id,
            HashMap::from([(
                "cortical_dimensions_per_device".to_string(),
                json!([6, 5, 7]),
            )]),
        )
        .await
        .expect("failed updating per-device dimensions");
    assert_eq!(
        updated_per_device.dimensions,
        (30, 5, 7),
        "per-device update must preserve Y/Z and apply dev_count on X only"
    );
    assert_eq!(
        updated_per_device.dev_count,
        Some(5),
        "per-device update must not change dev_count"
    );
    assert_eq!(
        updated_per_device.cortical_dimensions_per_device,
        Some((6, 5, 7)),
        "per-device dimensions should be stored and returned verbatim"
    );
}

#[tokio::test]
async fn test_registration_dimensions_replace_stale_per_device_depth() {
    let state = build_test_state();
    let cortical_id = general_purpose::STANDARD.encode(*b"cREGZ001");
    state
        .genome_service
        .create_cortical_areas(vec![CreateCorticalAreaParams {
            cortical_id: cortical_id.clone(),
            name: "servo-encoder-stale-z".to_string(),
            dimensions: (6, 1, 10),
            position: (0, 0, 0),
            area_type: "sensory".to_string(),
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
            properties: Some(HashMap::from([
                ("dev_count".to_string(), json!(6)),
                (
                    "cortical_dimensions_per_device".to_string(),
                    json!([1, 1, 10]),
                ),
            ])),
        }])
        .await
        .expect("failed creating stale servo encoder area");

    let updated = state
        .genome_service
        .update_cortical_area(
            &cortical_id,
            HashMap::from([
                ("dimensions".to_string(), json!([6, 1, 20])),
                ("dev_count".to_string(), json!(6)),
                (
                    "cortical_dimensions_per_device".to_string(),
                    json!([1, 1, 20]),
                ),
            ]),
        )
        .await
        .expect("failed applying registration dimensions");
    assert_eq!(
        updated.dimensions,
        (6, 1, 20),
        "registration z-depth must replace leftover genome per-device depth"
    );
    assert_eq!(
        updated.cortical_dimensions_per_device,
        Some((1, 1, 20)),
        "registration must persist the new per-device depth"
    );
    assert_eq!(updated.dev_count, Some(6));
}

#[tokio::test]
async fn test_delete_multi_cortical_area_success() {
    let state = build_test_state();

    let area_1 = general_purpose::STANDARD.encode(*b"cMDLT001");
    let area_2 = general_purpose::STANDARD.encode(*b"cMDLT002");

    let create_params_1 = CreateCorticalAreaParams {
        cortical_id: area_1.clone(),
        name: "multi-delete-area-1".to_string(),
        dimensions: (1, 1, 1),
        position: (0, 0, 0),
        area_type: "Custom".to_string(),
        visible: None,
        sub_group: None,
        neurons_per_voxel: None,
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
    let create_params_2 = CreateCorticalAreaParams {
        cortical_id: area_2.clone(),
        name: "multi-delete-area-2".to_string(),
        dimensions: (1, 1, 1),
        position: (1, 0, 0),
        area_type: "Custom".to_string(),
        visible: None,
        sub_group: None,
        neurons_per_voxel: None,
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

    state
        .connectome_service
        .create_cortical_area(create_params_1)
        .await
        .expect("Failed to create first cortical area for multi-delete test");
    state
        .connectome_service
        .create_cortical_area(create_params_2)
        .await
        .expect("Failed to create second cortical area for multi-delete test");

    let response = delete_multi_cortical_area(
        ApiStateExtract(state.clone()),
        ApiJson(vec![area_1.clone(), area_2.clone()]),
    )
    .await
    .expect("Multi-delete endpoint should succeed");

    let body = response.0;
    assert_eq!(
        body.get("message").map(String::as_str),
        Some("Deleted 2 cortical areas")
    );

    let area_1_exists = state
        .connectome_service
        .cortical_area_exists(&area_1)
        .await
        .expect("Failed to check first cortical area existence");
    let area_2_exists = state
        .connectome_service
        .cortical_area_exists(&area_2)
        .await
        .expect("Failed to check second cortical area existence");

    assert!(!area_1_exists);
    assert!(!area_2_exists);
}

// ============================================================================
// GENOME TESTS
// ============================================================================

#[tokio::test]
async fn test_genome_validate_minimal() {
    let app = create_test_server().await;

    // Minimal valid genome
    let genome = json!({
        "blueprint": {
            "cortical_areas": {}
        }
    });

    let (status, response) = request_json(
        app,
        "POST",
        "/v1/genome/validate",
        Some(json!({ "genome_json": genome.to_string() })),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    // Response should indicate validation result
    assert!(response.is_object());
}

// ============================================================================
// ERROR FORMAT TESTS
// ============================================================================

/// Build a multipart/form-data body with field `file`.
fn multipart_file_body(boundary: &str, file_name: &str, file_bytes: &[u8]) -> Vec<u8> {
    let mut body = Vec::with_capacity(file_bytes.len() + 256);
    body.extend_from_slice(
        format!(
            "--{boundary}\r\nContent-Disposition: form-data; name=\"file\"; filename=\"{file_name}\"\r\nContent-Type: application/octet-stream\r\n\r\n"
        )
        .as_bytes(),
    );
    body.extend_from_slice(file_bytes);
    body.extend_from_slice(format!("\r\n--{boundary}--\r\n").as_bytes());
    body
}

#[tokio::test]
async fn test_connectome_upload_accepts_payload_larger_than_axum_default_body_limit() {
    let app = create_test_server().await;
    let boundary = "----FEAGIConnectomeBoundary";
    let oversized = vec![0u8; (2 * 1024 * 1024) + 1];
    let request = Request::builder()
        .uri("/v1/connectome/upload")
        .method("POST")
        .header(
            "content-type",
            format!("multipart/form-data; boundary={boundary}"),
        )
        .body(Body::from(multipart_file_body(
            boundary,
            "oversize.connectome",
            &oversized,
        )))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    let status = response.status();
    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body_text = String::from_utf8_lossy(&body_bytes);

    assert_ne!(
        status,
        StatusCode::INTERNAL_SERVER_ERROR,
        "multipart connectome upload must not fail while buffering the body: {body_text}"
    );
    assert!(
        !body_text.contains("Invalid multipart upload"),
        "Axum default 2 MiB limit must not reject connectome multipart: status={status} body={body_text}"
    );
}

#[tokio::test]
async fn test_genome_upload_file_accepts_payload_larger_than_axum_default_body_limit() {
    let app = create_test_server().await;
    let boundary = "----FEAGIGenomeBoundary";
    let oversized = vec![b'{'; (2 * 1024 * 1024) + 1];
    let request = Request::builder()
        .uri("/v1/genome/upload/file")
        .method("POST")
        .header(
            "content-type",
            format!("multipart/form-data; boundary={boundary}"),
        )
        .body(Body::from(multipart_file_body(
            boundary,
            "oversize.genome",
            &oversized,
        )))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    let status = response.status();
    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body_text = String::from_utf8_lossy(&body_bytes);

    assert_ne!(
        status,
        StatusCode::INTERNAL_SERVER_ERROR,
        "multipart genome upload must not fail while buffering the body: {body_text}"
    );
    assert!(
        !body_text.contains("Invalid multipart upload"),
        "Axum default 2 MiB limit must not reject genome multipart: status={status} body={body_text}"
    );
}

#[tokio::test]
async fn test_last_failed_mutation_records_failed_connectome_upload() {
    let app = create_test_server().await;
    let boundary = "----FEAGIConnectomeBoundary";
    let payload = b"not-a-valid-connectome";
    let upload = Request::builder()
        .uri("/v1/connectome/upload")
        .method("POST")
        .header(
            "content-type",
            format!("multipart/form-data; boundary={boundary}"),
        )
        .header("content-length", payload.len().to_string())
        .body(Body::from(multipart_file_body(
            boundary,
            "bad.connectome",
            payload,
        )))
        .unwrap();

    let upload_response = app.clone().oneshot(upload).await.unwrap();
    assert!(
        upload_response.status().as_u16() >= 400,
        "invalid connectome must fail so last_failed_mutation has a record"
    );

    let (status, body) = request_json(app, "GET", "/v1/system/last_failed_mutation", None).await;
    assert_eq!(status, StatusCode::OK);
    let mutation = body
        .get("mutation")
        .expect("mutation field required")
        .as_object()
        .expect("mutation object expected after failed upload");
    assert_eq!(mutation["method"], "POST");
    assert_eq!(mutation["path"], "/v1/connectome/upload");
    assert!(mutation["status"].as_u64().unwrap() >= 400);
    assert!(
        mutation
            .get("content_type")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .contains("multipart/form-data"),
        "content_type should capture multipart upload: {mutation:?}"
    );
}

#[tokio::test]
async fn test_error_format_consistency() {
    let app = create_test_server().await;

    // All error responses should have consistent format
    let (status, response) = request_json(
        app,
        "POST",
        "/v1/cortical_area/cortical_area",
        Some(json!({})),
    )
    .await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    // Should have some error information
    assert!(response.is_object() || response.is_string());
}

#[test]
fn test_compilation() {
    // This test just ensures the code compiles
}

// ============================================================================
// GENOME CHANGE LEDGER TESTS
// ============================================================================

async fn seed_ledger_test_area(state: &ApiState, raw_id: &[u8; 8]) -> String {
    let cortical_id = general_purpose::STANDARD.encode(raw_id);
    state
        .genome_service
        .create_cortical_areas(vec![CreateCorticalAreaParams {
            cortical_id: cortical_id.clone(),
            name: "ledger-area".to_string(),
            dimensions: (1, 1, 1),
            position: (0, 0, 0),
            area_type: "Custom".to_string(),
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
        }])
        .await
        .expect("seed ledger test area");
    cortical_id
}

async fn request_json_as(
    app: axum::Router,
    method: &str,
    path: &str,
    agent_id: &str,
    body: Value,
) -> (StatusCode, Value) {
    let request = Request::builder()
        .uri(path)
        .method(method)
        .header("content-type", "application/json")
        .header("x-feagi-agent-id", agent_id)
        .body(Body::from(serde_json::to_vec(&body).unwrap()))
        .unwrap();
    let response = app.oneshot(request).await.unwrap();
    let status = response.status();
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    (
        status,
        serde_json::from_slice(&bytes).unwrap_or(json!(null)),
    )
}

#[tokio::test]
async fn test_genome_changes_record_http_caller_and_advance_health_sequence() {
    let state = build_test_state();
    let cortical_id = seed_ledger_test_area(&state, b"cLEDGER1").await;

    let (status, _) = request_json_as(
        create_http_server(state.clone()),
        "PUT",
        "/v1/cortical_area/cortical_area",
        "member-a",
        json!({"cortical_id": cortical_id, "cortical_name": "renamed over http"}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    let (status, page) = request_json(
        create_http_server(state.clone()),
        "GET",
        "/v1/genome/changes?since=0",
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let changes = page["changes"].as_array().expect("changes array");
    assert_eq!(changes.len(), 2, "seed create plus the HTTP update");
    assert!(changes[0].get("agent_id").is_none(), "seed had no caller");
    let update = &changes[1];
    assert_eq!(update["agent_id"], "member-a");
    assert_eq!(update["origin"], "local");
    assert_eq!(update["operation"]["kind"], "update_cortical_area");
    assert_eq!(update["operation"]["cortical_id"], cortical_id.as_str());
    assert_eq!(update["targets"][0]["kind"], "cortical_area");
    assert!(update["before"].is_object() && update["after"].is_object());

    let (_, health) = request_json(
        create_http_server(state.clone()),
        "GET",
        "/v1/system/health_check",
        None,
    )
    .await;
    assert_eq!(health["genome_change_sequence"], page["latest_sequence"]);

    let since = update["sequence"].as_u64().expect("sequence");
    let (_, empty) = request_json(
        create_http_server(state),
        "GET",
        &format!("/v1/genome/changes?since={since}"),
        None,
    )
    .await;
    assert!(empty["changes"].as_array().expect("array").is_empty());
}

#[tokio::test]
async fn test_genome_changes_reject_invalid_agent_header_and_limit() {
    let state = build_test_state();
    let too_long = "a".repeat(
        state
            .change_ledger
            .as_ref()
            .expect("ledger")
            .config()
            .agent_id_max_length
            + 1,
    );
    let (status, _) = request_json_as(
        create_http_server(state.clone()),
        "PUT",
        "/v1/cortical_area/cortical_area",
        &too_long,
        json!({"cortical_id": "x"}),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);

    let (status, _) = request_json(
        create_http_server(state),
        "GET",
        "/v1/genome/changes?since=0&limit=0",
        None,
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_genome_change_applies_once_on_peer_instance() {
    let source = build_test_state();
    let peer = build_test_state();
    let cortical_id = seed_ledger_test_area(&source, b"cLEDGER2").await;
    seed_ledger_test_area(&peer, b"cLEDGER2").await;

    let (status, _) = request_json_as(
        create_http_server(source.clone()),
        "PUT",
        "/v1/cortical_area/cortical_area",
        "member-a",
        json!({"cortical_id": cortical_id, "cortical_name": "from source"}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let (_, page) = request_json(
        create_http_server(source),
        "GET",
        "/v1/genome/changes?since=1",
        None,
    )
    .await;
    let change = page["changes"][0].clone();

    let (status, applied) = request_json(
        create_http_server(peer.clone()),
        "POST",
        "/v1/genome/changes/apply",
        Some(change.clone()),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(applied["status"], "applied");

    let (status, again) = request_json(
        create_http_server(peer.clone()),
        "POST",
        "/v1/genome/changes/apply",
        Some(change.clone()),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(again["status"], "already_applied");

    let area = peer
        .connectome_service
        .get_cortical_area(&cortical_id)
        .await
        .expect("area on peer");
    assert_eq!(area.name, "from source");
    let recorded = peer
        .change_ledger
        .as_ref()
        .expect("ledger")
        .find(change["change_id"].as_str().expect("change id"))
        .expect("recorded on peer");
    assert_eq!(recorded.agent_id.as_deref(), Some("member-a"));
    assert_eq!(
        serde_json::to_value(recorded.origin).expect("origin"),
        json!("replayed")
    );
}

#[tokio::test]
async fn test_genome_change_apply_rejects_markers_and_missing_targets() {
    let state = build_test_state();
    let marker = json!({
        "change_id": "marker-1",
        "group_id": "marker-1",
        "timestamp_ms": 0,
        "operation": {"kind": "connectome_reset"}
    });
    let (status, _) = request_json(
        create_http_server(state.clone()),
        "POST",
        "/v1/genome/changes/apply",
        Some(marker),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);

    let missing = json!({
        "change_id": "missing-1",
        "group_id": "missing-1",
        "timestamp_ms": 0,
        "operation": {"kind": "delete_brain_region", "region_id": "no-such-region"}
    });
    let (status, _) = request_json(
        create_http_server(state.clone()),
        "POST",
        "/v1/genome/changes/apply",
        Some(missing),
    )
    .await;
    assert!(
        status.is_client_error(),
        "missing target must be rejected, got {status}"
    );
    assert!(!state
        .change_ledger
        .as_ref()
        .expect("ledger")
        .contains_change_id("missing-1"));
}

#[tokio::test]
async fn test_genome_changes_not_enabled_without_ledger() {
    let mut state = build_test_state();
    state.change_ledger = None;
    let (status, _) = request_json(
        create_http_server(state),
        "GET",
        "/v1/genome/changes?since=0",
        None,
    )
    .await;
    assert_eq!(status, StatusCode::NOT_IMPLEMENTED);
}

#[tokio::test]
async fn test_clone_replicates_stored_neural_parameters() {
    let state = build_test_state();
    let root_id = "550e8400-e29b-41d4-a716-446655440000";
    let region_id = "550e8400-e29b-41d4-a716-446655440001";
    state
        .connectome_service
        .create_brain_region(CreateBrainRegionParams {
            region_id: root_id.to_string(),
            name: "root".to_string(),
            region_type: "Custom".to_string(),
            parent_id: None,
            properties: None,
        })
        .await
        .expect("root region");
    state
        .connectome_service
        .create_brain_region(CreateBrainRegionParams {
            region_id: region_id.to_string(),
            name: "circuit".to_string(),
            region_type: "Custom".to_string(),
            parent_id: Some(root_id.to_string()),
            properties: None,
        })
        .await
        .expect("circuit region");

    let source_id = general_purpose::STANDARD.encode(b"csource1");
    let mut properties = HashMap::new();
    properties.insert("parent_region_id".to_string(), json!(region_id));
    properties.insert("coordinate_2d".to_string(), json!([10, 20]));
    properties.insert("2d_coordinate".to_string(), json!([10, 20]));
    properties.insert("relative_coordinate".to_string(), json!([100, 0, 0]));
    properties.insert("visible".to_string(), json!(false));
    properties.insert("sub_group".to_string(), json!("assoc"));
    properties.insert("cortical_group".to_string(), json!("CUSTOM"));
    properties.insert("group_id".to_string(), json!("g1"));
    properties.insert("sub_group_id".to_string(), json!("sg1"));
    properties.insert("neurons_per_voxel".to_string(), json!(3));
    properties.insert("postsynaptic_current".to_string(), json!(1.25));
    properties.insert("postsynaptic_current_max".to_string(), json!(42.0));
    properties.insert("plasticity_constant".to_string(), json!(0.4));
    properties.insert("degeneration".to_string(), json!(0.15));
    properties.insert("psp_uniform_distribution".to_string(), json!(true));
    properties.insert("mp_driven_psp".to_string(), json!(true));
    properties.insert("firing_threshold".to_string(), json!(2.5));
    properties.insert("firing_threshold_increment_x".to_string(), json!(0.1));
    properties.insert("firing_threshold_increment_y".to_string(), json!(0.2));
    properties.insert("firing_threshold_increment_z".to_string(), json!(0.3));
    properties.insert("firing_threshold_limit".to_string(), json!(9.0));
    properties.insert("consecutive_fire_limit".to_string(), json!(4));
    properties.insert("snooze_period".to_string(), json!(5));
    properties.insert("refractory_period".to_string(), json!(2));
    properties.insert("leak_coefficient".to_string(), json!(0.25));
    properties.insert("leak_variability".to_string(), json!(0.05));
    properties.insert("mp_charge_accumulation".to_string(), json!(true));
    properties.insert("neuron_excitability".to_string(), json!(7.0));
    properties.insert("burst_engine_active".to_string(), json!(true));
    properties.insert("init_lifespan".to_string(), json!(9));
    properties.insert("lifespan_growth_rate".to_string(), json!(1.5));
    properties.insert("longterm_mem_threshold".to_string(), json!(6));
    properties.insert("synapse_attractivity".to_string(), json!(77.0));
    properties.insert("cortical_mapping_dst".to_string(), json!({}));

    state
        .genome_service
        .create_cortical_areas(vec![CreateCorticalAreaParams {
            cortical_id: source_id.clone(),
            name: "source-area".to_string(),
            dimensions: (2, 3, 1),
            position: (100, 0, 0),
            area_type: "Custom".to_string(),
            visible: Some(false),
            sub_group: Some("assoc".to_string()),
            neurons_per_voxel: Some(3),
            postsynaptic_current: Some(1.25),
            plasticity_constant: Some(0.4),
            degeneration: Some(0.15),
            psp_uniform_distribution: Some(true),
            firing_threshold_increment: None,
            firing_threshold_limit: Some(9.0),
            consecutive_fire_count: Some(4),
            snooze_period: Some(5),
            refractory_period: Some(2),
            leak_coefficient: Some(0.25),
            leak_variability: Some(0.05),
            burst_engine_active: Some(true),
            properties: Some(properties),
        }])
        .await
        .expect("create source area");

    let response = feagi_api::endpoints::cortical_area::post_clone(
        ApiStateExtract(state.clone()),
        ApiJson(
            feagi_api::endpoints::cortical_area::CloneCorticalAreaRequest {
                source_area_id: source_id.clone(),
                new_name: "cloned-area".to_string(),
                coordinates_3d: [200, 0, 0],
                coordinates_2d: [40, 50],
                parent_region_id: Some(region_id.to_string()),
                clone_cortical_mapping: false,
            },
        ),
    )
    .await
    .expect("clone cortical area");
    let new_area_id = response.0.get("new_area_id").expect("new area id").clone();

    let source = state
        .connectome_service
        .get_cortical_area(&source_id)
        .await
        .expect("source area");
    let cloned = state
        .connectome_service
        .get_cortical_area(&new_area_id)
        .await
        .expect("cloned area");

    assert_eq!(cloned.name, "cloned-area");
    assert_eq!(cloned.dimensions, source.dimensions);
    assert_eq!(cloned.position, (200, 0, 0));
    assert_eq!(cloned.visible, source.visible);
    assert_eq!(cloned.sub_group, source.sub_group);
    assert_eq!(cloned.neurons_per_voxel, source.neurons_per_voxel);
    assert_eq!(cloned.postsynaptic_current, source.postsynaptic_current);
    assert_eq!(
        cloned.postsynaptic_current_max,
        source.postsynaptic_current_max
    );
    assert_eq!(cloned.plasticity_constant, source.plasticity_constant);
    assert_eq!(cloned.degeneration, source.degeneration);
    assert_eq!(
        cloned.psp_uniform_distribution,
        source.psp_uniform_distribution
    );
    assert_eq!(cloned.mp_driven_psp, source.mp_driven_psp);
    assert_eq!(cloned.firing_threshold, source.firing_threshold);
    assert_eq!(
        cloned.firing_threshold_increment,
        source.firing_threshold_increment
    );
    assert_eq!(cloned.firing_threshold_limit, source.firing_threshold_limit);
    assert_eq!(cloned.consecutive_fire_count, source.consecutive_fire_count);
    assert_eq!(cloned.snooze_period, source.snooze_period);
    assert_eq!(cloned.refractory_period, source.refractory_period);
    assert_eq!(cloned.leak_coefficient, source.leak_coefficient);
    assert_eq!(cloned.leak_variability, source.leak_variability);
    assert_eq!(cloned.mp_charge_accumulation, source.mp_charge_accumulation);
    assert_eq!(cloned.neuron_excitability, source.neuron_excitability);
    assert_eq!(cloned.burst_engine_active, source.burst_engine_active);
    assert_eq!(cloned.init_lifespan, source.init_lifespan);
    assert_eq!(cloned.lifespan_growth_rate, source.lifespan_growth_rate);
    assert_eq!(cloned.longterm_mem_threshold, source.longterm_mem_threshold);

    let cloned_raw = state
        .connectome_service
        .get_cortical_area_properties(&new_area_id)
        .await
        .expect("cloned raw properties");
    assert_eq!(cloned_raw.get("synapse_attractivity"), Some(&json!(77.0)));
    assert_eq!(cloned_raw.get("cortical_group"), Some(&json!("CUSTOM")));
    assert_eq!(cloned_raw.get("group_id"), Some(&json!("g1")));
    assert_eq!(cloned_raw.get("sub_group_id"), Some(&json!("sg1")));
    assert_eq!(cloned_raw.get("coordinate_2d"), Some(&json!([40, 50])));
    assert_eq!(cloned_raw.get("2d_coordinate"), Some(&json!([40, 50])));
    assert_eq!(
        cloned_raw.get("relative_coordinate"),
        Some(&json!([200, 0, 0]))
    );
    assert!(
        cloned_raw
            .get("cortical_mapping_dst")
            .and_then(|v| v.as_object())
            .map(|map| map.is_empty())
            .unwrap_or(true),
        "mappings are reapplied separately, not copied as a stale destination map"
    );
}

#[tokio::test]
async fn test_saved_genome_carries_change_history() {
    let state = build_test_state();
    let cortical_id = seed_ledger_test_area(&state, b"cLEDGER3").await;
    let saved = state
        .genome_service
        .save_genome(feagi_services::types::SaveGenomeParams {
            genome_id: None,
            genome_title: None,
        })
        .await
        .expect("save");
    let saved: Value = serde_json::from_str(&saved).expect("saved JSON");
    let history = saved["change_history"].as_array().expect("history");
    assert_eq!(history.len(), 1);
    assert_eq!(history[0]["operation"]["kind"], "create_cortical_areas");
    assert_eq!(history[0]["targets"][0]["id"], cortical_id.as_str());
}

/// `sensory_hold_last_frame` is readable through the cortical area property routes,
/// writable through PUT as a metadata change, and persisted in the saved genome.
#[tokio::test]
async fn test_sensory_hold_last_frame_put_get_and_save() {
    use feagi_structures::genomic::cortical_area::descriptors::CorticalUnitIndex;
    use feagi_structures::genomic::SensoryCorticalUnit;

    let state = build_test_state();
    let imu_id = SensoryCorticalUnit::RawIMU
        .get_default_cortical_id_for_group(CorticalUnitIndex::from(0u16))
        .as_base_64();
    state
        .genome_service
        .create_cortical_areas(vec![CreateCorticalAreaParams {
            cortical_id: imu_id.clone(),
            name: "imu".to_string(),
            dimensions: (1, 1, 4),
            position: (0, 0, 0),
            area_type: "sensory".to_string(),
            visible: None,
            sub_group: None,
            neurons_per_voxel: None,
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
        }])
        .await
        .expect("create IMU area");

    // Template default for a scalar IMU input: hold on, even with no stored property.
    let (status, props) = request_json(
        create_http_server(state.clone()),
        "POST",
        "/v1/cortical_area/cortical_area_properties",
        Some(json!({"cortical_id": imu_id})),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(props["sensory_hold_last_frame"], json!(true));

    let (status, _) = request_json(
        create_http_server(state.clone()),
        "PUT",
        "/v1/cortical_area/cortical_area",
        Some(json!({"cortical_id": imu_id, "sensory_hold_last_frame": false})),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    let (status, props) = request_json(
        create_http_server(state.clone()),
        "POST",
        "/v1/cortical_area/cortical_area_properties",
        Some(json!({"cortical_id": imu_id})),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(props["sensory_hold_last_frame"], json!(false));

    let (status, geometry) = request_json(
        create_http_server(state.clone()),
        "GET",
        "/v1/cortical_area/cortical_area/geometry",
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(geometry[&imu_id]["sensory_hold_last_frame"], json!(false));

    // Persisted in the flat genome (`snhold-b`) so it survives save/load.
    let saved = state
        .genome_service
        .save_genome(feagi_services::types::SaveGenomeParams {
            genome_id: None,
            genome_title: None,
        })
        .await
        .expect("save");
    let saved: Value = serde_json::from_str(&saved).expect("saved JSON");
    let blueprint = saved["blueprint"].as_object().expect("flat blueprint");
    let hold_entry = blueprint
        .iter()
        .find(|(key, _)| key.ends_with("-snhold-b") && key.contains(&imu_id))
        .map(|(_, value)| value.clone());
    assert_eq!(hold_entry, Some(json!(false)));
}

// ---- simulation_timestep: physiology stores + applies live; burst_engine/config is live only ----

/// Tests share one process-wide mock frequency cell, so they run under this lock.
/// Async mutex so the guard can stay held across `.await` without blocking a worker.
static TIMESTEP_TEST_LOCK: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

fn mock_applied_frequency_hz() -> f64 {
    f64::from_bits(MOCK_RUNTIME_FREQUENCY_BITS.load(Ordering::SeqCst))
}

async fn stored_timestep(state: &ApiState) -> f64 {
    // Read the way a save/export does.
    let json = state
        .genome_service
        .save_genome(feagi_services::types::SaveGenomeParams {
            genome_id: None,
            genome_title: None,
        })
        .await
        .expect("save genome");
    let value: Value = serde_json::from_str(&json).expect("genome json");
    value["physiology"]["simulation_timestep"]
        .as_f64()
        .expect("physiology.simulation_timestep in saved genome")
}

#[tokio::test]
async fn test_physiology_timestep_is_stored_and_applied_live() {
    let _guard = TIMESTEP_TEST_LOCK.lock().await;
    let state = build_test_state();
    MOCK_RUNTIME_FREQUENCY_BITS.store(0, Ordering::SeqCst);
    let before = state
        .genome_service
        .get_genome_info()
        .await
        .map(|i| i.simulation_timestep)
        .unwrap_or(0.0);
    assert!((before - 0.0125).abs() > 1e-9);

    let (status, body) = request_json_as(
        create_http_server(state.clone()),
        "PUT",
        "/v1/physiology/",
        "t",
        json!({"physiology": {"simulation_timestep": 0.0125}}),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["success"], json!(true));
    assert_eq!(body["updated"]["simulation_timestep"], json!(0.0125));
    // Running rate follows.
    assert!((mock_applied_frequency_hz() - 80.0).abs() < 1e-9);
    // Stored genome follows, and a save/export carries it.
    assert!((stored_timestep(&state).await - 0.0125).abs() < 1e-12);
    let (status, _) = request_json_as(
        create_http_server(state.clone()),
        "GET",
        "/v1/genome/download",
        "t",
        json!(null),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
}

#[tokio::test]
async fn test_burst_engine_config_leaves_stored_timestep_unchanged() {
    let _guard = TIMESTEP_TEST_LOCK.lock().await;
    let state = build_test_state();
    MOCK_RUNTIME_FREQUENCY_BITS.store(0, Ordering::SeqCst);
    let (status, _) = request_json_as(
        create_http_server(state.clone()),
        "PUT",
        "/v1/physiology/",
        "t",
        json!({"physiology": {"simulation_timestep": 0.0125}}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let stored_before = stored_timestep(&state).await;
    assert!((stored_before - 0.0125).abs() < 1e-12);

    let (status, body) = request_json_as(
        create_http_server(state.clone()),
        "PUT",
        "/v1/burst_engine/config",
        "t",
        json!({"burst_frequency_hz": 40.0}),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert!((mock_applied_frequency_hz() - 40.0).abs() < 1e-9);
    assert!((stored_timestep(&state).await - stored_before).abs() < 1e-15);
}

#[tokio::test]
async fn test_physiology_timestep_rejects_invalid_values() {
    let _guard = TIMESTEP_TEST_LOCK.lock().await;
    let state = build_test_state();
    let stored_before = stored_timestep(&state).await;
    for bad in [json!(0.0), json!(-0.01), json!("fast"), json!(1e-320)] {
        MOCK_RUNTIME_FREQUENCY_BITS.store(0, Ordering::SeqCst);
        let (status, body) = request_json_as(
            create_http_server(state.clone()),
            "PUT",
            "/v1/physiology/",
            "t",
            json!({"physiology": {"simulation_timestep": bad}}),
        )
        .await;
        assert_eq!(status, StatusCode::BAD_REQUEST, "value {bad}: {body}");
        assert_eq!(mock_applied_frequency_hz(), 0.0, "rate changed for {bad}");
        assert!((stored_timestep(&state).await - stored_before).abs() < 1e-15);
    }
    // The burst_engine POST route shares the validation.
    let (status, _) = request_json_as(
        create_http_server(state.clone()),
        "POST",
        "/v1/burst_engine/simulation_timestep",
        "t",
        json!({"simulation_timestep": 0.0}),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

// ============================================================================
// ROUTE PATH PARAMETER STYLE
// ============================================================================

/// Source of the HTTP router. The axum `Router` cannot be introspected, so the
/// route table is read straight from the file that registers it.
const HTTP_SERVER_SOURCE: &str = include_str!("../src/transports/http/server.rs");

/// Every string literal passed as the first argument of a `.route(` call.
fn registered_route_paths(source: &str) -> Vec<String> {
    let mut paths = Vec::new();
    let mut rest = source;
    while let Some(pos) = rest.find(".route(") {
        rest = &rest[pos + ".route(".len()..];
        let after = rest.trim_start();
        let Some(literal) = after.strip_prefix('"') else {
            panic!(
                "`.route(` argument is not a string literal near: {:?}",
                &after[..after.len().min(60)]
            );
        };
        let end = literal
            .find('"')
            .expect("unterminated route string literal");
        paths.push(literal[..end].to_string());
        rest = &literal[end..];
    }
    paths
}

/// axum 0.7 matches `{id}` literally, so such a route 404s for every real id;
/// path parameters must be written `:name`.
fn route_path_style_violations(paths: &[String]) -> Vec<String> {
    let mut violations = Vec::new();
    for path in paths {
        if path.contains('{') || path.contains('}') {
            violations.push(format!("{path}: brace-style parameter (use `:name`)"));
            continue;
        }
        for segment in path.split('/') {
            if let Some(name) = segment.strip_prefix(':') {
                let valid =
                    !name.is_empty() && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_');
                if !valid {
                    violations.push(format!("{path}: malformed parameter segment `{segment}`"));
                }
            } else if segment.contains(':') {
                violations.push(format!(
                    "{path}: `:` must start a whole segment (`{segment}`)"
                ));
            }
        }
    }
    violations
}

#[test]
fn test_registered_routes_use_colon_path_parameters() {
    let paths = registered_route_paths(HTTP_SERVER_SOURCE);
    assert!(
        paths.len() > 100,
        "route parser found only {} routes; the parser or server.rs layout changed",
        paths.len()
    );
    assert!(
        paths.iter().any(|p| p.contains("/:")),
        "expected at least one `:name` route in server.rs"
    );
    let violations = route_path_style_violations(&paths);
    assert!(
        violations.is_empty(),
        "axum 0.7 path parameters must be `:name`, not `{{name}}`:\n{}",
        violations.join("\n")
    );
}

#[test]
fn test_route_path_style_check_rejects_brace_parameters() {
    let sample = r#"
        Router::new()
            .route("/v1/foo/:id", get(a))
            .route(
                "/v1/foo/{id}",
                get(b),
            )
            .route("/v1/bar/x:id", get(c))
    "#;
    let paths = registered_route_paths(sample);
    assert_eq!(paths, ["/v1/foo/:id", "/v1/foo/{id}", "/v1/bar/x:id"]);
    let violations = route_path_style_violations(&paths);
    assert_eq!(violations.len(), 2, "{violations:?}");
    assert!(violations[0].starts_with("/v1/foo/{id}"));
    assert!(violations[1].starts_with("/v1/bar/x:id"));
}

// ============================================================================
// BURST-ALIGNED FIRE HISTORY
// ============================================================================

/// `GET|DELETE /v1/burst_engine/fire_history` resolve cortical ids, enforce the area and
/// burst caps, and shape the service's frames into aligned voxel lists.
#[tokio::test]
async fn test_burst_engine_fire_history_route_shape_and_limits() {
    let state = build_test_state();
    let area_id = create_plain_custom_area(&state, b"cfirehA1", "fire-history", (0, 0, 0)).await;
    let cortical_idx = state
        .connectome_service
        .get_cortical_area(&area_id)
        .await
        .expect("area")
        .cortical_idx;

    let (status, body) = request_json(
        create_http_server(state.clone()),
        "GET",
        &format!("/v1/burst_engine/fire_history?areas={area_id}&bursts=5"),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["current_burst"], json!(42));
    assert_eq!(body["bursts"], json!(5));
    assert_eq!(body["max_bursts"], json!(2000));
    assert_eq!(body["max_areas"], json!(16));
    assert_eq!(body["subscription_ttl_s"], json!(300));
    let area = &body["areas"][&area_id];
    assert_eq!(area["cortical_idx"], json!(cortical_idx));
    assert_eq!(area["name"], json!("fire-history"));
    assert_eq!(area["window"], json!(5));
    assert_eq!(area["frames"][0]["burst"], json!(41));
    assert_eq!(area["frames"][0]["voxels"], json!([]));
    assert_eq!(area["frames"][1]["burst"], json!(42));
    assert_eq!(area["frames"][1]["voxels"], json!([[1, 2, 3]]));
    assert_eq!(area["frames"][1]["memory_neuron_ids"], json!([50000001]));

    // Default bursts, duplicate ids collapse to one area.
    let (status, body) = request_json(
        create_http_server(state.clone()),
        "GET",
        &format!("/v1/burst_engine/fire_history?areas={area_id},{area_id}"),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["bursts"], json!(100));
    assert_eq!(body["areas"].as_object().map(|m| m.len()), Some(1));

    for (path, expected) in [
        (
            "/v1/burst_engine/fire_history".to_string(),
            StatusCode::BAD_REQUEST,
        ),
        (
            "/v1/burst_engine/fire_history?areas=".to_string(),
            StatusCode::BAD_REQUEST,
        ),
        (
            format!("/v1/burst_engine/fire_history?areas={area_id}&bursts=0"),
            StatusCode::BAD_REQUEST,
        ),
        (
            format!("/v1/burst_engine/fire_history?areas={area_id}&bursts=2001"),
            StatusCode::BAD_REQUEST,
        ),
        (
            format!("/v1/burst_engine/fire_history?areas={area_id}&bursts=ten"),
            StatusCode::BAD_REQUEST,
        ),
        (
            format!(
                "/v1/burst_engine/fire_history?areas={}",
                (0..17)
                    .map(|i| format!("Y2ZpcmVoQ{i:02}"))
                    .collect::<Vec<_>>()
                    .join(",")
            ),
            StatusCode::BAD_REQUEST,
        ),
        (
            "/v1/burst_engine/fire_history?areas=Y25vc3VjaDE=".to_string(),
            StatusCode::NOT_FOUND,
        ),
    ] {
        let (status, body) =
            request_json(create_http_server(state.clone()), "GET", &path, None).await;
        assert_eq!(status, expected, "{path}: {body}");
    }

    let (status, body) = request_json(
        create_http_server(state.clone()),
        "DELETE",
        &format!("/v1/burst_engine/fire_history?areas={area_id}"),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["unsubscribed"], json!(1));
    assert_eq!(body["areas"], json!([area_id]));
}

// ============================================================================
// CORTICAL MAPPING: WEIGHT RESET AND PLASTICITY FREEZE
// ============================================================================

async fn create_plain_custom_area(
    state: &ApiState,
    id_bytes: &[u8; 8],
    name: &str,
    position: (i32, i32, i32),
) -> String {
    let cortical_id = general_purpose::STANDARD.encode(id_bytes);
    state
        .genome_service
        .create_cortical_areas(vec![CreateCorticalAreaParams {
            cortical_id: cortical_id.clone(),
            name: name.to_string(),
            dimensions: (1, 1, 1),
            position,
            area_type: "Custom".to_string(),
            visible: None,
            sub_group: None,
            neurons_per_voxel: Some(1),
            postsynaptic_current: Some(1.0),
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
            burst_engine_active: Some(true),
            properties: None,
        }])
        .await
        .expect("create custom area");
    cortical_id
}

fn plastic_projector_rule(weight: f64) -> Value {
    json!({
        "morphology_id": "projector",
        "morphology_scalar": [1, 1, 1],
        "postSynapticCurrent_multiplier": weight,
        "plasticity_flag": true,
        "plasticity_constant": 1,
        "ltp_multiplier": 1,
        "ltd_multiplier": 1,
        "plasticity_window": 1
    })
}

/// `GET|PUT /v1/cortical_mapping/plasticity_frozen`
/// drive one plastic mapping end to end through the real connectome service and NPU.
#[tokio::test]
async fn test_cortical_mapping_plasticity_frozen_routes() {
    let state = build_test_state();
    state
        .connectome_service
        .create_morphology(
            "projector".to_string(),
            feagi_evolutionary::Morphology {
                morphology_type: feagi_evolutionary::MorphologyType::Functions,
                parameters: feagi_evolutionary::MorphologyParameters::Functions {},
                class: "core".to_string(),
            },
        )
        .await
        .expect("register projector morphology");
    let src = create_plain_custom_area(&state, b"cwrsrcA1", "weights-src", (0, 0, 0)).await;
    let dst = create_plain_custom_area(&state, b"cwrdstA1", "weights-dst", (10, 0, 0)).await;

    let (status, body) = request_json(
        create_http_server(state.clone()),
        "PUT",
        "/v1/cortical_mapping/mapping_properties",
        Some(json!({
            "src_cortical_area": src,
            "dst_cortical_area": dst,
            "mapping_string": [plastic_projector_rule(2.0)]
        })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "create mapping: {body}");
    let created = body["synapse_count"].as_u64().expect("synapse_count");
    assert!(
        created >= 1,
        "projector between 1x1x1 areas must create a synapse"
    );

    let frozen_query = format!(
        "/v1/cortical_mapping/plasticity_frozen?src_cortical_area={src}&dst_cortical_area={dst}"
    );
    let (status, body) = request_json(
        create_http_server(state.clone()),
        "GET",
        &frozen_query,
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["frozen"], json!(false));

    let (status, body) = request_json(
        create_http_server(state.clone()),
        "PUT",
        "/v1/cortical_mapping/plasticity_frozen",
        Some(json!({"src_cortical_area": src, "dst_cortical_area": dst, "frozen": true})),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["frozen"], json!(true));
    assert_eq!(body["src_cortical_area"], json!(src));
    assert_eq!(body["dst_cortical_area"], json!(dst));

    let (status, body) = request_json(
        create_http_server(state.clone()),
        "GET",
        &frozen_query,
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["frozen"], json!(true));

    // The flag is visible on the rule itself through the mapping listing.
    let (status, body) = request_json(
        create_http_server(state.clone()),
        "GET",
        &format!("/v1/cortical_mapping/mapping?src_cortical_area={src}&dst_cortical_area={dst}"),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["connections"][0]["plasticity_frozen"], json!(true));
    assert_eq!(body["connections"][0]["morphology_id"], json!("projector"));

    // Validation: unknown mapping.
    let (status, _) = request_json(
        create_http_server(state.clone()),
        "PUT",
        "/v1/cortical_mapping/plasticity_frozen",
        Some(json!({"src_cortical_area": dst, "dst_cortical_area": src, "frozen": true})),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST, "no mapping dst -> src");
    let (status, body) = request_json(
        create_http_server(state.clone()),
        "GET",
        &format!("/v1/cortical_mapping/plasticity_frozen?src_cortical_area={dst}&dst_cortical_area={src}"),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["frozen"], json!(null));

    // Deleting the mapping drops its synapses; a recreate starts from the rule weight.
    let (status, body) = request_json(
        create_http_server(state.clone()),
        "DELETE",
        &format!("/v1/cortical_mapping/mapping?src_cortical_area={src}&dst_cortical_area={dst}"),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["synapse_count"].as_u64(), Some(0));
}
