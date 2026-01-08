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
use feagi_structures::genomic::cortical_area::descriptors::{CorticalUnitIndex, CorticalSubUnitIndex};
use feagi_structures::genomic::cortical_area::io_cortical_area_configuration_flag::FrameChangeHandling;
use feagi_structures::genomic::cortical_area::CorticalID;
use feagi_structures::genomic::{MotorCorticalUnit, SensoryCorticalUnit};

use super::agent_registry::{
    AgentCapabilities, AgentInfo, AgentRegistry, AgentTransport, AgentType, MotorCapability,
    MotorUnit, SensoryCapability, SensoryUnit, VisionCapability, VisualizationCapability,
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
    /// For IPU/OPU areas, the name might be a short prefix (e.g., "svi").
    /// We try multiple approaches to create a valid CorticalID:
    /// 1. Try direct padding with null bytes (standard approach)
    /// 2. If that fails, try padding with spaces (legacy compatibility)
    /// 3. If both fail, try using try_from_base_64 if the name looks like base64
    /// 4. If all fail, return an error with helpful message
    fn area_name_to_cortical_id(&self, area_name: &str) -> Result<String, String> {
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

        // If null padding fails, try space padding (for legacy compatibility)
        let mut bytes = [b' '; 8];
        bytes[..copy_len].copy_from_slice(&name_bytes[..copy_len]);

        if let Ok(cortical_id) = CorticalID::try_from_bytes(&bytes) {
            return Ok(cortical_id.as_base_64());
        }

        // Both approaches failed - return detailed error
        Err(format!(
            "Failed to create CorticalID from area name '{}' (length: {}). \
            Tried null-byte and space padding. \
            The area name may be too short or in an invalid format. \
            For IPU/OPU areas, use a valid 4-character prefix like 'isvi', 'imot', etc., \
            or provide a base64-encoded CorticalID.",
            area_name,
            area_name.len()
        ))
    }

    /// Find SensoryCorticalUnit by unit identifier bytes
    fn find_sensory_unit_by_identifier(
        &self,
        identifier: [u8; 3],
    ) -> Result<SensoryCorticalUnit, String> {
        // Iterate through all SensoryCorticalUnit variants to find matching identifier
        use SensoryCorticalUnit::*;
        for unit in [
            Infrared,
            Proximity,
            Shock,
            Battery,
            Servo,
            AnalogGPIO,
            DigitalGPIO,
            MiscData,
            TextEnglishInput,
            Vision,
            SegmentedVision,
            Accelerometer,
            Gyroscope,
        ] {
            if unit.get_cortical_id_unit_reference() == identifier {
                return Ok(unit);
            }
        }
        Err(format!(
            "No SensoryCorticalUnit found for identifier: {:?}",
            identifier
        ))
    }

    /// Find MotorCorticalUnit by its 3-byte identifier (e.g., [b's', b'e', b'g'] for ObjectSegmentation)
    fn find_motor_unit_by_identifier(
        &self,
        identifier: [u8; 3],
    ) -> Result<MotorCorticalUnit, String> {
        // Iterate through all MotorCorticalUnit variants to find matching identifier
        use MotorCorticalUnit::*;
        for unit in [
            RotaryMotor,
            PositionalServo,
            Gaze,
            MiscData,
            TextEnglishOutput,
            ObjectSegmentation,
            SimpleVisionOutput,
        ] {
            if unit.get_cortical_id_unit_reference() == identifier {
                return Ok(unit);
            }
        }
        Err(format!(
            "No MotorCorticalUnit found for identifier: {:?}",
            identifier
        ))
    }

    /// Get all cortical IDs for a given SensoryCorticalUnit using generic method dispatch
    fn get_all_cortical_ids_for_unit(
        &self,
        unit: SensoryCorticalUnit,
        frame_change_handling: FrameChangeHandling,
        percentage_neuron_positioning: feagi_structures::genomic::cortical_area::io_cortical_area_configuration_flag::PercentageNeuronPositioning,
        group: CorticalUnitIndex,
    ) -> Result<Vec<CorticalID>, String> {
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
            AnalogGPIO => SensoryCorticalUnit::get_cortical_ids_array_for_analog_g_p_i_o_with_parameters(
                frame_change_handling,
                percentage_neuron_positioning,
                group,
            )
            .to_vec(),
            DigitalGPIO => {
                SensoryCorticalUnit::get_cortical_ids_array_for_digital_g_p_i_o_with_parameters(group).to_vec()
            }
            MiscData => SensoryCorticalUnit::get_cortical_ids_array_for_misc_data_with_parameters(
                frame_change_handling,
                group,
            )
            .to_vec(),
            TextEnglishInput => SensoryCorticalUnit::get_cortical_ids_array_for_text_english_input_with_parameters(
                frame_change_handling,
                group,
            )
            .to_vec(),
            Vision => SensoryCorticalUnit::get_cortical_ids_array_for_segmented_vision_with_parameters(
                frame_change_handling,
                group,
            )
            .to_vec(),
            SegmentedVision => SensoryCorticalUnit::get_cortical_ids_array_for_segmented_vision_with_parameters(
                frame_change_handling,
                group,
            )
            .to_vec(),
            Accelerometer => SensoryCorticalUnit::get_cortical_ids_array_for_accelerometer_with_parameters(
                frame_change_handling,
                percentage_neuron_positioning,
                group,
            )
            .to_vec(),
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
    fn block_on_async_service<F>(&self, future_factory: F) -> Result<bool, String>
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
            let result = (|| -> Result<bool, String> {
                debug!("🦀 [REGISTRATION] Creating new tokio runtime in thread");
                let rt = tokio::runtime::Runtime::new()
                    .map_err(|e| format!("Failed to create runtime: {}", e))?;
                debug!("🦀 [REGISTRATION] Blocking on async future");
                let result = rt
                    .block_on(future)
                    .map_err(|e| format!("Service error: {}", e))?;
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
                Err("Timeout waiting for cortical area existence check (5s)".to_string())
            }
            Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                error!(
                    "🦀 [REGISTRATION] Thread disconnected while checking cortical area existence"
                );
                Err("Thread disconnected while checking cortical area existence".to_string())
            }
        }
    }

    /// Check and ensure required cortical areas exist, creating missing ones if enabled
    /// Returns availability status for all required areas
    fn ensure_cortical_areas_exist(
        &self,
        capabilities: &AgentCapabilities,
    ) -> Result<CorticalAreaAvailability, String> {
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
                "GenomeService not available - required for cortical area management".to_string()
            })?
            .clone();
        let connectome_service = self
            .connectome_service
            .lock()
            .as_ref()
            .ok_or_else(|| {
                "ConnectomeService not available - required for cortical area checking".to_string()
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
                    let (width, height, channels, x, y, z) =
                        if let Some(unit_topology) =
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
                            (vision.dimensions.0, vision.dimensions.1, vision.channels, 0, 0, 0)
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
                            format!(
                                "Failed to check cortical area existence for IPU area '{}': {}",
                                area_key, e
                            )
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
                        return Err(format!(
                            "Required IPU area '{}' is missing. Enable auto_create_missing_cortical_areas in config to auto-create.",
                            area_key
                        ));
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
                        let rt = tokio::runtime::Runtime::new()
                            .map_err(|e| format!("Failed to create tokio runtime: {}", e))?;
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
                            return Err(format!("Failed to create IPU area '{}': {}", area_key, e));
                        }
                    }
                }

            } else {
            // For IPU areas, ensure 'i' prefix is present (e.g., "svi" -> "isvi")
            let area_name = if vision.target_cortical_area.starts_with('i') {
                vision.target_cortical_area.clone()
            } else {
                format!("i{}", vision.target_cortical_area)
            };

            // Convert area name to CorticalID to extract unit information
            let base_cortical_id = match self.area_name_to_cortical_id(&area_name) {
                Ok(id) => CorticalID::try_from_base_64(&id)
                    .map_err(|e| format!("Failed to parse cortical ID: {}", e))?,
                Err(e) => {
                    error!(
                        "🦀 [REGISTRATION] ❌ Failed to convert area name '{}' to CorticalID: {}",
                        area_name, e
                    );
                    return Err(format!(
                        "Invalid cortical area name '{}': {}. \
                        For IPU areas, use a valid 4-character prefix like 'isvi', 'imot', etc.",
                        vision.target_cortical_area, e
                    ));
                }
            };

            // Extract unit identifier from cortical ID (bytes 1-3 after 'i' prefix)
            let cortical_id_bytes = base_cortical_id.as_bytes();
            if cortical_id_bytes.len() < 4 || cortical_id_bytes[0] != b'i' {
                return Err(format!(
                    "Invalid IPU cortical ID format for '{}'",
                    area_name
                ));
            }
            let unit_identifier = [
                cortical_id_bytes[1],
                cortical_id_bytes[2],
                cortical_id_bytes[3],
            ];

            // Extract group index from IO cortical ID (byte 7)
            let group_index = cortical_id_bytes[7];
            let group: CorticalUnitIndex = group_index.into();

            // Find matching SensoryCorticalUnit by unit identifier
            let sensory_unit = self.find_sensory_unit_by_identifier(unit_identifier)?;
            let number_areas = sensory_unit.get_number_cortical_areas();

            info!(
                "🦀 [REGISTRATION] Detected cortical unit type: {} ({} areas)",
                sensory_unit.get_snake_case_name(),
                number_areas
            );

            if number_areas > 1 && self.auto_create_missing_areas {
                // This cortical type requires multiple areas - create all of them
                info!(
                    "🦀 [REGISTRATION] Creating all {} cortical areas for {}",
                    number_areas,
                    sensory_unit.get_snake_case_name()
                );

                // Use default frame_change_handling (Absolute) - this should match what the encoder uses
                // TODO: Get frame_change_handling from registration request if available
                let frame_change_handling = FrameChangeHandling::Absolute;
                use feagi_structures::genomic::cortical_area::io_cortical_area_configuration_flag::PercentageNeuronPositioning;
                let percentage_neuron_positioning = PercentageNeuronPositioning::Linear; // Default

                // Get all cortical IDs for this unit type using generic method dispatch
                let cortical_ids = self.get_all_cortical_ids_for_unit(
                    sensory_unit,
                    frame_change_handling,
                    percentage_neuron_positioning,
                    group,
                )?;

                // Get topology (dimensions and positions) for all areas from the unit definition
                let topology = sensory_unit.get_unit_default_topology();

                info!(
                    "🦀 [REGISTRATION] Generated {} cortical IDs for {} (group={})",
                    cortical_ids.len(),
                    sensory_unit.get_snake_case_name(),
                    group_index
                );

                // Create all cortical areas using topology information
                let mut create_params_list = Vec::new();
                for (i, cortical_id) in cortical_ids.iter().enumerate() {
                    let cortical_id_base64 = cortical_id.as_base_64();

                    // Get dimensions and position from topology
                    let (width, height, channels, x, y, z) =
                        if let Some(unit_topology) = topology.get(&CorticalSubUnitIndex::from(i as u8)) {
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
                            // Fallback to default dimensions from vision capability if topology not available
                            (
                                vision.dimensions.0,
                                vision.dimensions.1,
                                vision.channels,
                                0,
                                0,
                                0,
                            )
                        };

                    // Check if this area already exists
                    let exists = {
                        let cortical_id_clone = cortical_id_base64.clone();
                        let connectome_service_clone = connectome_service.clone();
                        self.block_on_async_service(move || {
                            let service = connectome_service_clone.clone();
                            let id = cortical_id_clone.clone();
                            Box::pin(async move { service.cortical_area_exists(&id).await })
                        })
                    }
                    .map_err(|e| {
                        error!(
                            "🦀 [REGISTRATION] Failed to check existence for area {}: {}",
                            i, e
                        );
                        format!("Failed to check cortical area existence: {}", e)
                    })?;

                    if exists {
                        info!(
                            "🦀 [REGISTRATION] Cortical area {} already exists (cortical_id: {})",
                            i, cortical_id_base64
                        );
                        ipu_statuses.push(CorticalAreaStatus {
                            area_name: format!("{}_area{}", area_name, i),
                            cortical_id: cortical_id_base64,
                            status: AreaStatus::Existing,
                            dimensions: Some((width, height, channels)),
                            message: None,
                        });
                    } else {
                        let area_name_suffix = if number_areas > 1 {
                            format!("{}_area{}", area_name, i)
                        } else {
                            area_name.clone()
                        };
                        create_params_list.push(feagi_services::types::CreateCorticalAreaParams {
                            cortical_id: cortical_id_base64.clone(),
                            name: area_name_suffix.clone(),
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
                        });

                        info!("🦀 [REGISTRATION] Will create cortical area {}: {} ({}x{}x{}) at ({},{},{})",
                              i, cortical_id_base64, width, height, channels, x, y, z);
                    }
                }

                // Create all missing areas in one batch
                if !create_params_list.is_empty() {
                    info!(
                        "🦀 [REGISTRATION] Creating {} cortical areas for {}...",
                        create_params_list.len(),
                        sensory_unit.get_snake_case_name()
                    );

                    let result = if let Ok(handle) = tokio::runtime::Handle::try_current() {
                        handle.block_on(
                            genome_service.create_cortical_areas(create_params_list.clone()),
                        )
                    } else {
                        let rt = tokio::runtime::Runtime::new()
                            .map_err(|e| format!("Failed to create tokio runtime: {}", e))?;
                        rt.block_on(
                            genome_service.create_cortical_areas(create_params_list.clone()),
                        )
                    };

                    match result {
                        Ok(_) => {
                            info!("🦀 [REGISTRATION] ✅ Successfully created {} cortical areas for {}", create_params_list.len(), sensory_unit.get_snake_case_name());
                            for params in create_params_list {
                                ipu_statuses.push(CorticalAreaStatus {
                                    area_name: params.name.clone(),
                                    cortical_id: params.cortical_id.clone(),
                                    status: AreaStatus::Created,
                                    dimensions: Some(params.dimensions),
                                    message: Some("Auto-created during registration".to_string()),
                                });
                            }
                        }
                        Err(e) => {
                            error!(
                                "🦀 [REGISTRATION] ❌ Failed to create cortical areas for {}: {}",
                                sensory_unit.get_snake_case_name(),
                                e
                            );
                            return Err(format!(
                                "Failed to create cortical areas for {}: {}",
                                sensory_unit.get_snake_case_name(),
                                e
                            ));
                        }
                    }
                } else {
                    info!(
                        "🦀 [REGISTRATION] All {} cortical areas already exist",
                        sensory_unit.get_snake_case_name()
                    );
                }
            } else if number_areas == 1 {
                // Handle single cortical area - use the same generic approach
                let cortical_id_base64 = base_cortical_id.as_base_64();

                // Get topology for single area (index 0)
                let topology = sensory_unit.get_unit_default_topology();
                let (width, height, channels, x, y, z) =
                    if let Some(unit_topology) = topology.get(&CorticalSubUnitIndex::from(0u8)) {
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
                        // Fallback to dimensions from vision capability
                        (
                            vision.dimensions.0,
                            vision.dimensions.1,
                            vision.channels,
                            0,
                            0,
                            0,
                        )
                    };

                // Check if area exists
                debug!(
                    "🦀 [REGISTRATION] Checking IPU area existence for '{}' (cortical_id: {})",
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
                            debug!("🦀 [REGISTRATION] Calling cortical_area_exists for IPU area '{}'", id);
                            let result = service.cortical_area_exists(&id).await;
                            debug!("🦀 [REGISTRATION] cortical_area_exists result for IPU area '{}': {:?}", id, result);
                            result
                        })
                    })
                }
                .map_err(|e| {
                    error!("🦀 [REGISTRATION] Failed to check IPU area existence for '{}' (cortical_id: {}): {}", area_name, cortical_id_base64, e);
                    format!("Failed to check cortical area existence for IPU area '{}': {}", area_name, e)
                })?;
                debug!(
                    "🦀 [REGISTRATION] IPU area '{}' exists: {}",
                    area_name, exists
                );

                if exists {
                    info!("🦀 [REGISTRATION] IPU area '{}' already exists", area_name);
                    ipu_statuses.push(CorticalAreaStatus {
                        area_name: area_name.clone(),
                        cortical_id: cortical_id_base64,
                        status: AreaStatus::Existing,
                        dimensions: Some((width, height, channels)),
                        message: None,
                    });
                } else if self.auto_create_missing_areas {
                    // Create missing IPU area
                    info!(
                        "🦀 [REGISTRATION] Auto-creating missing IPU area '{}'",
                        area_name
                    );

                    let create_params = feagi_services::types::CreateCorticalAreaParams {
                        cortical_id: cortical_id_base64.clone(),
                        name: area_name.clone(),
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

                    // Try to use current runtime handle first, fallback to creating new runtime
                    let result = if let Ok(handle) = tokio::runtime::Handle::try_current() {
                        // We're in an async context - use block_on on the handle
                        handle.block_on(genome_service.create_cortical_areas(vec![create_params]))
                    } else {
                        // Not in async context - create a new runtime
                        let rt = tokio::runtime::Runtime::new()
                            .map_err(|e| format!("Failed to create tokio runtime: {}", e))?;
                        rt.block_on(genome_service.create_cortical_areas(vec![create_params]))
                    };

                    match result {
                        Ok(_) => {
                            info!(
                                "🦀 [REGISTRATION] ✅ Successfully created IPU area '{}'",
                                area_name
                            );
                            ipu_statuses.push(CorticalAreaStatus {
                                area_name: area_name.clone(),
                                cortical_id: cortical_id_base64,
                                status: AreaStatus::Created,
                                dimensions: Some((width, height, channels)),
                                message: Some("Auto-created during registration".to_string()),
                            });
                        }
                        Err(e) => {
                            error!(
                                "🦀 [REGISTRATION] ❌ Failed to create IPU area '{}': {}",
                                area_name, e
                            );
                            ipu_statuses.push(CorticalAreaStatus {
                                area_name: area_name.clone(),
                                cortical_id: cortical_id_base64,
                                status: AreaStatus::Error,
                                dimensions: None,
                                message: Some(format!("Creation failed: {}", e)),
                            });
                            return Err(format!(
                                "Failed to create IPU area '{}': {}",
                                area_name, e
                            ));
                        }
                    }
                } else {
                    warn!(
                        "🦀 [REGISTRATION] ⚠️ IPU area '{}' is missing and auto-create is disabled",
                        area_name
                    );
                    ipu_statuses.push(CorticalAreaStatus {
                        area_name: area_name.clone(),
                        cortical_id: cortical_id_base64,
                        status: AreaStatus::Missing,
                        dimensions: None,
                        message: Some("Area missing and auto-create disabled".to_string()),
                    });
                    return Err(format!("Required IPU area '{}' is missing. Enable auto_create_missing_cortical_areas in config to auto-create.", area_name));
                }
            } else {
                // number_areas == 0 or auto-create disabled - skip
                warn!("🦀 [REGISTRATION] ⚠️ Cortical unit {} has {} areas but auto-create is disabled or invalid",
                      sensory_unit.get_snake_case_name(), number_areas);
            }
        }
        }

        // Handle sensory-mapped IPU areas (legacy sensory capability with cortical_mappings).
        //
        // This enables non-vision sensory modalities (e.g., iten) to trigger deterministic
        // cortical area creation during agent registration.
        if let Some(ref sensory) = capabilities.sensory {
            for (area_key, _idx) in &sensory.cortical_mappings {
                let cortical_id_base64 = self.area_name_to_cortical_id(area_key)?;
                let cortical_id = CorticalID::try_from_base_64(&cortical_id_base64)
                    .map_err(|e| format!("Failed to parse cortical ID: {}", e))?;

                let cortical_id_bytes = cortical_id.as_bytes();
                if cortical_id_bytes[0] != b'i' {
                    return Err(format!(
                        "Sensory cortical mapping '{}' does not refer to an IPU cortical ID",
                        area_key
                    ));
                }

                let unit_identifier = [
                    cortical_id_bytes[1],
                    cortical_id_bytes[2],
                    cortical_id_bytes[3],
                ];
                let _group: CorticalUnitIndex = cortical_id_bytes[7].into();

                let sensory_unit = self.find_sensory_unit_by_identifier(unit_identifier)?;
                let number_areas = sensory_unit.get_number_cortical_areas();

                // For multi-area units (like segmented_vision with 9 areas), skip auto-creation
                // but still process mappings if vision capability already created the areas
                let should_skip_creation = number_areas != 1;
                let has_vision_capability = capabilities.vision.is_some();
                
                if should_skip_creation && !has_vision_capability {
                    warn!(
                        "🦀 [REGISTRATION] Sensory unit {} has {} areas; skipping auto-create via sensory.cortical_mappings (use vision-capability path for multi-area units)",
                        sensory_unit.get_snake_case_name(),
                        number_areas
                    );
                    continue;
                }
                
                // For multi-area units with vision capability, skip creation but still verify existence
                let skip_creation = should_skip_creation && has_vision_capability;
                if skip_creation {
                    info!(
                        "🦀 [REGISTRATION] Sensory unit {} has {} areas; vision capability should have created areas, verifying existence",
                        sensory_unit.get_snake_case_name(),
                        number_areas
                    );
                }

                // Check if this specific cortical ID exists
                let exists = {
                    let cortical_id_clone = cortical_id_base64.clone();
                    let connectome_service_clone = connectome_service.clone();
                    self.block_on_async_service(move || {
                        let service = connectome_service_clone.clone();
                        let id = cortical_id_clone.clone();
                        Box::pin(async move { service.cortical_area_exists(&id).await })
                    })
                }
                .map_err(|e| {
                    format!(
                        "Failed to check cortical area existence for sensory mapping '{}': {}",
                        area_key, e
                    )
                })?;

                // Derive deterministic name/dimensions from the unit template topology
                let topology = sensory_unit.get_unit_default_topology();
                let unit_topology = topology.get(&CorticalSubUnitIndex::from(0u8)).ok_or_else(|| {
                    format!(
                        "Sensory unit {} missing topology entry for area 0",
                        sensory_unit.get_snake_case_name()
                    )
                })?;
                let dims = unit_topology.channel_dimensions_default;
                let pos = unit_topology.relative_position;

                let area_prefix = std::str::from_utf8(&cortical_id_bytes[0..4])
                    .map_err(|_| "Failed to decode IPU cortical ID prefix as UTF-8".to_string())?
                    .trim_end_matches('\0')
                    .trim_end_matches('_')
                    .to_string();
                let area_name = format!("{}_g{}", area_prefix, cortical_id_bytes[7]);

                if exists {
                    ipu_statuses.push(CorticalAreaStatus {
                        area_name,
                        cortical_id: cortical_id_base64,
                        status: AreaStatus::Existing,
                        dimensions: Some((dims[0] as usize, dims[1] as usize, dims[2] as usize)),
                        message: None,
                    });
                } else if !skip_creation && self.auto_create_missing_areas {
                    let create_params = feagi_services::types::CreateCorticalAreaParams {
                        cortical_id: cortical_id_base64.clone(),
                        name: area_name.clone(),
                        dimensions: (dims[0] as usize, dims[1] as usize, dims[2] as usize),
                        position: (pos[0], pos[1], pos[2]),
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
                        let rt = tokio::runtime::Runtime::new()
                            .map_err(|e| format!("Failed to create tokio runtime: {}", e))?;
                        rt.block_on(genome_service.create_cortical_areas(vec![create_params]))
                    };

                    match result {
                        Ok(_) => {
                            ipu_statuses.push(CorticalAreaStatus {
                                area_name,
                                cortical_id: cortical_id_base64,
                                status: AreaStatus::Created,
                                dimensions: Some((dims[0] as usize, dims[1] as usize, dims[2] as usize)),
                                message: Some("Auto-created during registration".to_string()),
                            });
                        }
                        Err(e) => {
                            ipu_statuses.push(CorticalAreaStatus {
                                area_name,
                                cortical_id: cortical_id_base64,
                                status: AreaStatus::Error,
                                dimensions: None,
                                message: Some(format!("Creation failed: {}", e)),
                            });
                            return Err(format!(
                                "Failed to create IPU area '{}' from sensory mapping: {}",
                                area_key, e
                            ));
                        }
                    }
                } else {
                    ipu_statuses.push(CorticalAreaStatus {
                        area_name,
                        cortical_id: cortical_id_base64,
                        status: AreaStatus::Missing,
                        dimensions: None,
                        message: Some("Area missing and auto-create disabled".to_string()),
                    });
                    return Err(format!(
                        "Required IPU area '{}' is missing. Enable auto_create_missing_cortical_areas in config to auto-create.",
                        area_key
                    ));
                }
            }
        }

        // Handle OPU areas (from motor capabilities)
        if let Some(ref motor) = capabilities.motor {
            // Preferred Option B path: semantic unit + group, so agents don't need to know
            // internal 3-letter unit identifiers embedded in cortical IDs.
            let source_areas: Vec<String> = if let Some(source_units) = motor.source_units.as_ref() {
                let frame_change_handling = FrameChangeHandling::Absolute;
                use feagi_structures::genomic::cortical_area::io_cortical_area_configuration_flag::PercentageNeuronPositioning;
                let percentage_neuron_positioning = PercentageNeuronPositioning::Linear; // Default

                let mut all_ids: Vec<String> = Vec::new();
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
                    };

                    all_ids.extend(cortical_ids.into_iter().map(|id| id.as_base_64()));
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
                };

                cortical_ids.into_iter().map(|id| id.as_base_64()).collect()
            } else {
                motor.source_cortical_areas.clone()
            };

            for area_name in &source_areas {
                let cortical_id_base64 = self.area_name_to_cortical_id(area_name)?;
                let cortical_id = CorticalID::try_from_base_64(&cortical_id_base64)
                    .map_err(|e| format!("Failed to parse cortical ID: {}", e))?;

                // Parse cortical ID to extract motor unit type and dimensions
                let cortical_id_bytes = cortical_id.as_bytes();
                let (motor_unit, dimensions, position) = if cortical_id_bytes.len() >= 4 && cortical_id_bytes[0] == b'o' {
                    // Extract unit identifier from cortical ID (bytes 1-3 after 'o' prefix)
                    let unit_identifier = [
                        cortical_id_bytes[1],
                        cortical_id_bytes[2],
                        cortical_id_bytes[3],
                    ];
                    
                    // Find matching MotorCorticalUnit
                    match self.find_motor_unit_by_identifier(unit_identifier) {
                        Ok(unit) => {
                            // Get default topology/dimensions from motor unit template
                            let topology = unit.get_unit_default_topology();
                            if let Some(unit_topology) = topology.get(&CorticalSubUnitIndex::from(0u8)) {
                                let dims = unit_topology.channel_dimensions_default;
                                let pos = unit_topology.relative_position;
                                (
                                    Some(unit),
                                    (dims[0] as usize, dims[1] as usize, dims[2] as usize),
                                    (pos[0], pos[1], pos[2]),
                                )
                            } else {
                                // Fallback to generic dimensions
                                (
                                    Some(unit),
                                    (motor.output_count, 1, 1),
                                    (0, 0, 0),
                                )
                            }
                        }
                        Err(e) => {
                            warn!(
                                "🦀 [REGISTRATION] Could not find motor unit for identifier {:?}: {}. Using generic dimensions.",
                                unit_identifier, e
                            );
                            (None, (motor.output_count, 1, 1), (0, 0, 0))
                        }
                    }
                } else {
                    // Not an OPU cortical ID format - use generic dimensions
                    warn!(
                        "🦀 [REGISTRATION] OPU area '{}' does not have 'o' prefix - using generic dimensions",
                        area_name
                    );
                    (None, (motor.output_count, 1, 1), (0, 0, 0))
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
                    format!("Failed to check cortical area existence for OPU area '{}': {}", area_name, e)
                })?;
                debug!(
                    "🦀 [REGISTRATION] OPU area '{}' exists: {}",
                    area_name, exists
                );

                if exists {
                    let motor_unit_name = motor_unit.as_ref()
                        .map(|u| u.get_snake_case_name())
                        .unwrap_or_else(|| "unknown");
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
                    let motor_unit_name = motor_unit.as_ref()
                        .map(|u| u.get_snake_case_name())
                        .unwrap_or_else(|| "generic");
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
                        let rt = tokio::runtime::Runtime::new()
                            .map_err(|e| format!("Failed to create tokio runtime: {}", e))?;
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
                            return Err(format!(
                                "Failed to create OPU area '{}': {}",
                                area_name, e
                            ));
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
                    return Err(format!("Required OPU area '{}' is missing. Enable auto_create_missing_cortical_areas in config to auto-create.", area_name));
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
    }

    /// Internal implementation
    fn process_registration_impl(
        &self,
        request: RegistrationRequest,
    ) -> Result<RegistrationResponse, String> {
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
                        return Err(format!(
                            "Transport '{}' not supported: WebSocket is disabled in FEAGI configuration",
                            requested_transport
                        ));
                    }
                }
                "zmq" | "shm" | "hybrid" => {
                    // ZMQ/SHM always available (for now)
                }
                other => {
                    return Err(format!(
                        "Transport '{}' not supported: Available transports are: zmq, websocket (if enabled), shm",
                        other
                    ));
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
            Some("shm") | Some("hybrid") => true,      // Agent explicitly wants SHM
            None => {
                // If agent didn't specify transport, check if they provided shm_path
                // If shm_path is None, agent is using ZMQ-only (don't auto-allocate SHM)
                // If shm_path is Some, agent wants SHM (or already has a path)
                capabilities.sensory.as_ref()
                    .and_then(|s| s.shm_path.as_ref())
                    .is_some()
            },
            Some(_) => false,                         // Unknown transport, don't offer SHM
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
            _ => return Err(format!("Invalid agent type: {}", request.agent_type)),
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
            .map_err(|e| format!("Failed to register agent: {}", e))?;

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

                    // Convert String keys to CorticalID for zero-copy hot path optimization
                    let area_mapping: HashMap<CorticalID, u32> = sensory
                        .cortical_mappings
                        .iter()
                        .filter_map(|(name, &idx)| {
                            // Try base64 first (standard format from SDK), then fall back to raw bytes
                            match CorticalID::try_from_base_64(name) {
                                Ok(id) => Some((id, idx)),
                                Err(_) => {
                                    // Fallback: try parsing as raw bytes (for legacy compatibility)
                                    match CorticalID::try_from_bytes(&{
                                        let mut bytes = [b'\0'; 8]; // Use null bytes, not spaces
                                        let name_bytes = name.as_bytes();
                                        let copy_len = name_bytes.len().min(8);
                                        bytes[..copy_len].copy_from_slice(&name_bytes[..copy_len]);
                                        bytes
                                    }) {
                                        Ok(id) => Some((id, idx)),
                                        Err(e) => {
                                            warn!("[REGISTRATION] Invalid cortical ID '{}': {:?}", name, e);
                                            None
                                        }
                                    }
                                }
                            }
                        })
                        .collect();

                    let config = feagi_npu_burst_engine::AgentConfig {
                        agent_id: request.agent_id.clone(),
                        shm_path: std::path::PathBuf::from(shm_path),
                        rate_hz: sensory.rate_hz,
                        area_mapping,
                    };

                    let register_start = std::time::Instant::now();
                    sensory_mgr
                        .register_agent(config)
                        .map_err(|e| format!("Failed to register with burst engine: {}", e))?;
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
                for area_input in &motor.source_cortical_areas {
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
    fn parse_capabilities(
        &self,
        caps_json: &serde_json::Value,
    ) -> Result<AgentCapabilities, String> {
        // Check if this is feagi-sensorimotor format (has "input"/"output" keys)
        // This must be checked BEFORE attempting direct deserialization because
        // AgentCapabilities has #[serde(flatten)] which will absorb these keys
        // into the custom map, resulting in empty capabilities.
        let has_legacy_format =
            caps_json.get("input").is_some() || caps_json.get("output").is_some();

        // Try to deserialize directly from JSON first (handles new agent SDK format)
        // But skip this if we detected the legacy format
        if !has_legacy_format {
            if let Ok(capabilities) = serde_json::from_value::<AgentCapabilities>(caps_json.clone())
            {
                return Ok(capabilities);
            }
        }

        // Support feagi-sensorimotor format: {"capabilities": {"input": {...}, "output": {...}}}
        // Unwrap if wrapped in "capabilities" key
        let caps_json = if let Some(caps_wrapper) = caps_json.get("capabilities") {
            caps_wrapper
        } else {
            caps_json
        };

        // Fall back to manual parsing for legacy format
        let mut capabilities = AgentCapabilities::default();

        // Parse vision capability (FEAGI 2.0)
        if let Some(vision) = caps_json.get("vision") {
            let modality = vision
                .get("modality")
                .and_then(|v| v.as_str())
                .unwrap_or("camera")
                .to_string();

            // Parse dimensions - can be array [width, height] or tuple
            let dimensions = if let Some(dims) = vision.get("dimensions") {
                if let Some(arr) = dims.as_array() {
                    if arr.len() >= 2 {
                        (
                            arr[0].as_u64().unwrap_or(0) as usize,
                            arr[1].as_u64().unwrap_or(0) as usize,
                        )
                    } else {
                        (0, 0)
                    }
                } else {
                    (0, 0)
                }
            } else {
                (0, 0)
            };

            let channels = vision.get("channels").and_then(|v| v.as_u64()).unwrap_or(3) as usize;

            let target_cortical_area = vision
                .get("target_cortical_area")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            let unit: Option<SensoryUnit> = vision
                .get("unit")
                .and_then(|v| serde_json::from_value::<SensoryUnit>(v.clone()).ok());
            let group: Option<u8> = vision.get("group").and_then(|v| v.as_u64()).map(|v| v as u8);

            // Accept either legacy target_cortical_area or semantic unit+group.
            if !target_cortical_area.is_empty() || (unit.is_some() && group.is_some()) {
                capabilities.vision = Some(VisionCapability {
                    modality,
                    dimensions,
                    channels,
                    target_cortical_area,
                    unit,
                    group,
                });
            }
        }

        // Parse input capability (feagi-sensorimotor format: ONLY "input"/"output" keys)
        // Format: {"input": ["cortical_id1", "cortical_id2", ...]}
        if let Some(input) = caps_json.get("input") {
            if let Some(cortical_areas) = input.as_array() {
                let mut cortical_mappings = HashMap::new();
                for area_id in cortical_areas.iter().filter_map(|v| v.as_str()) {
                    cortical_mappings.insert(area_id.to_string(), 0);
                }

                if !cortical_mappings.is_empty() {
                    capabilities.sensory = Some(SensoryCapability {
                        rate_hz: 30.0, // Default rate
                        shm_path: None,
                        cortical_mappings,
                    });
                }
            }
        }

        // Parse output capability (feagi-sensorimotor format: ONLY "input"/"output" keys)
        // Format: {"output": ["cortical_id1", "cortical_id2", ...]}
        if let Some(output) = caps_json.get("output") {
            if let Some(cortical_areas) = output.as_array() {
                let source_cortical_areas: Vec<String> = cortical_areas
                    .iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect();

                // Only create motor capability if source_cortical_areas is non-empty
                // ARCHITECTURAL PRINCIPLE: Don't invent capabilities that weren't requested
                if !source_cortical_areas.is_empty() {
                    capabilities.motor = Some(MotorCapability {
                        modality: "generic".to_string(),
                        output_count: source_cortical_areas.len(),
                        source_cortical_areas,
                        unit: None,
                        group: None,
                        source_units: None,
                    });
                }
            }
        }

        // Parse visualization capability
        if let Some(viz) = caps_json.get("visualization") {
            if viz
                .get("enabled")
                .and_then(|v| v.as_bool())
                .unwrap_or(false)
            {
                capabilities.visualization = Some(VisualizationCapability {
                    visualization_type: viz
                        .get("type")
                        .and_then(|v| v.as_str())
                        .unwrap_or("generic")
                        .to_string(),
                    resolution: None,
                    refresh_rate: viz.get("rate_hz").and_then(|v| v.as_f64()),
                    bridge_proxy: viz
                        .get("bridge_proxy")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(false),
                });
            }
        }

        Ok(capabilities)
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
                registry.get_all().iter().map(|a| a.agent_id.clone()).collect()
            };
            warn!(
                "⚠️ [REGISTRATION] Heartbeat failed for '{}': Agent not found in registry. Registered agents ({}): {:?}",
                agent_id,
                all_agents.len(),
                all_agents
            );
            return Err(format!(
                "Agent {} not found in registry (total registered: {})",
                agent_id,
                all_agents.len()
            ));
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
            agent_type: "both".to_string(),
            capabilities: serde_json::json!({
                "input": ["aXN2aQkAAAA="],  // feagi-sensorimotor format
                "output": ["b21vdDAwAAA="]  // feagi-sensorimotor format
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
    }
}
