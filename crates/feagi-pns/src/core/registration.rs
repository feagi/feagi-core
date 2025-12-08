// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

// Registration Handler - processes agent registration requests

use super::agent_registry::{
    AgentCapabilities, AgentInfo, AgentRegistry, AgentTransport, AgentType, MotorCapability,
    SensoryCapability, VisualizationCapability, VisionCapability,
};
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{info, warn, error, debug};
use ahash::AHashSet;
use feagi_data_structures::genomic::cortical_area::CorticalID;
use feagi_data_structures::genomic::SensoryCorticalUnit;
use feagi_data_structures::genomic::cortical_area::descriptors::CorticalGroupIndex;
use feagi_data_structures::genomic::cortical_area::io_cortical_area_data_type::FrameChangeHandling;
pub use feagi_services::types::registration::{
    AreaStatus, CorticalAreaAvailability, CorticalAreaStatus, RegistrationRequest,
    RegistrationResponse, TransportConfig,
};
use feagi_services::traits::registration_handler::RegistrationHandlerTrait;

/// Type alias for registration callbacks
pub type RegistrationCallback =
    Arc<parking_lot::Mutex<Option<Box<dyn Fn(String, String, String) + Send + Sync>>>>;
pub type DeregistrationCallback =
    Arc<parking_lot::Mutex<Option<Box<dyn Fn(String) + Send + Sync>>>>;
/// Type alias for dynamic gating callbacks
pub type DynamicGatingCallback =
    Arc<parking_lot::Mutex<Option<Box<dyn Fn(String) + Send + Sync>>>>;

/// Registration Handler
pub struct RegistrationHandler {
    agent_registry: Arc<RwLock<AgentRegistry>>,
    shm_base_path: String,
    /// Optional reference to burst engine's sensory agent manager for SHM I/O
    sensory_agent_manager:
        Arc<parking_lot::Mutex<Option<Arc<std::sync::Mutex<feagi_burst_engine::AgentManager>>>>>,
    /// Optional reference to burst loop runner for motor subscription tracking
    burst_runner:
        Arc<parking_lot::Mutex<Option<Arc<parking_lot::RwLock<feagi_burst_engine::BurstLoopRunner>>>>>,
    /// Optional reference to GenomeService for creating cortical areas
    genome_service:
        Arc<parking_lot::Mutex<Option<Arc<dyn feagi_services::traits::GenomeService + Send + Sync>>>>,
    /// Optional reference to ConnectomeService for checking cortical area existence
    connectome_service:
        Arc<parking_lot::Mutex<Option<Arc<dyn feagi_services::traits::ConnectomeService + Send + Sync>>>>,
    /// Configuration for auto-creating missing cortical areas
    auto_create_missing_areas: bool,
    /// Actual ZMQ port numbers (from config, NOT hardcoded)
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
    pub fn new(agent_registry: Arc<RwLock<AgentRegistry>>, sensory_port: u16, motor_port: u16, viz_port: u16) -> Self {
        Self {
            agent_registry,
            shm_base_path: "/tmp".to_string(),
            sensory_agent_manager: Arc::new(parking_lot::Mutex::new(None)),
            burst_runner: Arc::new(parking_lot::Mutex::new(None)),
            genome_service: Arc::new(parking_lot::Mutex::new(None)),
            connectome_service: Arc::new(parking_lot::Mutex::new(None)),
            auto_create_missing_areas: true,  // Default enabled
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
    pub fn set_genome_service(&self, service: Arc<dyn feagi_services::traits::GenomeService + Send + Sync>) {
        *self.genome_service.lock() = Some(service);
        info!("🦀 [REGISTRATION] GenomeService connected for cortical area creation");
    }
    
    /// Set ConnectomeService for checking cortical area existence
    pub fn set_connectome_service(&self, service: Arc<dyn feagi_services::traits::ConnectomeService + Send + Sync>) {
        *self.connectome_service.lock() = Some(service);
        info!("🦀 [REGISTRATION] ConnectomeService connected for cortical area checking");
    }
    
    /// Set auto-create missing cortical areas configuration
    pub fn set_auto_create_missing_areas(&mut self, enabled: bool) {
        self.auto_create_missing_areas = enabled;
        info!("🦀 [REGISTRATION] Auto-create missing cortical areas: {}", enabled);
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
        info!("🦀 [REGISTRATION] WebSocket transport configured: enabled={}, ports={}:{}:{}:{}", 
              enabled, sensory_port, motor_port, viz_port, registration_port);
    }
    
    /// Set burst runner reference (for motor subscription tracking)
    pub fn set_burst_runner(
        &self,
        runner: Arc<parking_lot::RwLock<feagi_burst_engine::BurstLoopRunner>>,
    ) {
        *self.burst_runner.lock() = Some(runner);
        info!("🦀 [REGISTRATION] Burst runner connected for motor subscriptions");
    }

    /// Set the sensory agent manager (for SHM I/O coordination)
    pub fn set_sensory_agent_manager(
        &self,
        manager: Arc<std::sync::Mutex<feagi_burst_engine::AgentManager>>,
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
            area_name, area_name.len()
        ))
    }
    
    /// Find SensoryCorticalUnit by unit identifier bytes
    fn find_sensory_unit_by_identifier(&self, identifier: [u8; 3]) -> Result<SensoryCorticalUnit, String> {
        // Iterate through all SensoryCorticalUnit variants to find matching identifier
        use SensoryCorticalUnit::*;
        for unit in [
            Infrared, Proximity, Shock, Battery, Servo, AnalogGPIO, DigitalGPIO, 
            MiscData, Vision, SegmentedVision, Accelerometer, Gyroscope
        ] {
            if unit.get_cortical_id_unit_reference() == identifier {
                return Ok(unit);
            }
        }
        Err(format!("No SensoryCorticalUnit found for identifier: {:?}", identifier))
    }
    
    /// Get all cortical IDs for a given SensoryCorticalUnit using generic method dispatch
    fn get_all_cortical_ids_for_unit(
        &self,
        unit: SensoryCorticalUnit,
        frame_change_handling: FrameChangeHandling,
        percentage_neuron_positioning: feagi_data_structures::genomic::cortical_area::io_cortical_area_data_type::PercentageNeuronPositioning,
        group: CorticalGroupIndex,
    ) -> Result<Vec<CorticalID>, String> {
        // Dispatch to the appropriate get_cortical_ids_array_for method based on unit type
        // This is systematic (covers all types) not hardcoded for one specific type
        // Note: Method signatures vary based on cortical_type_parameters in the template:
        // - Units with Percentage type need: (frame_change_handling, percentage_neuron_positioning, group)
        // - Units with ImageFrame/SegmentedImageFrame/MiscData need: (frame_change_handling, group)
        // - Units with Boolean type need: (group) only
        use SensoryCorticalUnit::*;
        let cortical_ids_array: Vec<CorticalID> = match unit {
            Infrared => SensoryCorticalUnit::get_cortical_ids_array_for_infrared(frame_change_handling, percentage_neuron_positioning, group).to_vec(),
            Proximity => SensoryCorticalUnit::get_cortical_ids_array_for_proximity(frame_change_handling, percentage_neuron_positioning, group).to_vec(),
            Shock => SensoryCorticalUnit::get_cortical_ids_array_for_shock(frame_change_handling, percentage_neuron_positioning, group).to_vec(),
            Battery => SensoryCorticalUnit::get_cortical_ids_array_for_battery(frame_change_handling, percentage_neuron_positioning, group).to_vec(),
            Servo => SensoryCorticalUnit::get_cortical_ids_array_for_servo(frame_change_handling, percentage_neuron_positioning, group).to_vec(),
            AnalogGPIO => SensoryCorticalUnit::get_cortical_ids_array_for_analog_gpio(frame_change_handling, percentage_neuron_positioning, group).to_vec(),
            DigitalGPIO => SensoryCorticalUnit::get_cortical_ids_array_for_digital_gpio(group).to_vec(),
            MiscData => SensoryCorticalUnit::get_cortical_ids_array_for_miscellaneous(frame_change_handling, group).to_vec(),
            Vision => SensoryCorticalUnit::get_cortical_ids_array_for_simple_vision(frame_change_handling, group).to_vec(),
            SegmentedVision => SensoryCorticalUnit::get_cortical_ids_array_for_segmented_vision(frame_change_handling, group).to_vec(),
            Accelerometer => SensoryCorticalUnit::get_cortical_ids_array_for_accelerometer(frame_change_handling, percentage_neuron_positioning, group).to_vec(),
            Gyroscope => SensoryCorticalUnit::get_cortical_ids_array_for_gyroscope(frame_change_handling, percentage_neuron_positioning, group).to_vec(),
        };
        
        Ok(cortical_ids_array)
    }

    /// Helper function to safely call async code from sync context
    /// Always uses a separate thread to avoid blocking the current runtime
    fn block_on_async_service<F>(&self, future_factory: F) -> Result<bool, String>
    where
        F: FnOnce() -> std::pin::Pin<Box<dyn std::future::Future<Output = feagi_services::types::ServiceResult<bool>> + Send>> + Send + 'static,
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
                let result = rt.block_on(future)
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
                error!("🦀 [REGISTRATION] Thread disconnected while checking cortical area existence");
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

        // Get services (required for cortical area management)
        let genome_service = self.genome_service.lock()
            .as_ref()
            .ok_or_else(|| "GenomeService not available - required for cortical area management".to_string())?
            .clone();
        let connectome_service = self.connectome_service.lock()
            .as_ref()
            .ok_or_else(|| "ConnectomeService not available - required for cortical area checking".to_string())?
            .clone();

        // Handle IPU areas (from vision capabilities)
        if let Some(ref vision) = capabilities.vision {
            // For IPU areas, ensure 'i' prefix is present (e.g., "svi" -> "isvi")
            let area_name = if vision.target_cortical_area.starts_with('i') {
                vision.target_cortical_area.clone()
            } else {
                format!("i{}", vision.target_cortical_area)
            };
            
            // Convert area name to CorticalID to extract unit information
            let base_cortical_id = match self.area_name_to_cortical_id(&area_name) {
                Ok(id) => CorticalID::try_from_base_64(&id).map_err(|e| format!("Failed to parse cortical ID: {}", e))?,
                Err(e) => {
                    error!("🦀 [REGISTRATION] ❌ Failed to convert area name '{}' to CorticalID: {}", area_name, e);
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
                return Err(format!("Invalid IPU cortical ID format for '{}'", area_name));
            }
            let unit_identifier = [cortical_id_bytes[1], cortical_id_bytes[2], cortical_id_bytes[3]];
            
            // Extract group index from cortical ID (byte 4, or default to 0)
            let group_index = if cortical_id_bytes.len() >= 5 {
                cortical_id_bytes[4] as u8
            } else {
                0
            };
            let group: CorticalGroupIndex = group_index.into();
            
            // Find matching SensoryCorticalUnit by unit identifier
            let sensory_unit = self.find_sensory_unit_by_identifier(unit_identifier)?;
            let number_areas = sensory_unit.get_number_cortical_areas();
            
            info!("🦀 [REGISTRATION] Detected cortical unit type: {} ({} areas)", sensory_unit.get_snake_case_name(), number_areas);
            
            if number_areas > 1 && self.auto_create_missing_areas {
                // This cortical type requires multiple areas - create all of them
                info!("🦀 [REGISTRATION] Creating all {} cortical areas for {}", number_areas, sensory_unit.get_snake_case_name());
                
                // Use default frame_change_handling (Absolute) - this should match what the encoder uses
                // TODO: Get frame_change_handling from registration request if available
                let frame_change_handling = FrameChangeHandling::Absolute;
                use feagi_data_structures::genomic::cortical_area::io_cortical_area_data_type::PercentageNeuronPositioning;
                let percentage_neuron_positioning = PercentageNeuronPositioning::Linear; // Default
                
                // Get all cortical IDs for this unit type using generic method dispatch
                let cortical_ids = self.get_all_cortical_ids_for_unit(sensory_unit, frame_change_handling, percentage_neuron_positioning, group)?;
                
                // Get topology (dimensions and positions) for all areas from the unit definition
                let topology = sensory_unit.get_unit_default_topology();
                
                info!("🦀 [REGISTRATION] Generated {} cortical IDs for {} (group={})", cortical_ids.len(), sensory_unit.get_snake_case_name(), group_index);
                
                // Create all cortical areas using topology information
                let mut create_params_list = Vec::new();
                for (i, cortical_id) in cortical_ids.iter().enumerate() {
                    let cortical_id_base64 = cortical_id.as_base_64();
                    
                    // Get dimensions and position from topology
                    let (width, height, channels, x, y, z) = if let Some(unit_topology) = topology.get(&i) {
                        let dims = unit_topology.channel_dimensions_default;
                        let pos = unit_topology.relative_position;
                        (dims[0] as usize, dims[1] as usize, dims[2] as usize, pos[0], pos[1], pos[2])
                    } else {
                        // Fallback to default dimensions from vision capability if topology not available
                        (vision.dimensions.0, vision.dimensions.1, vision.channels, 0, 0, 0)
                    };
                    
                    // Check if this area already exists
                    let exists = {
                        let cortical_id_clone = cortical_id_base64.clone();
                        let connectome_service_clone = connectome_service.clone();
                        self.block_on_async_service(move || {
                            let service = connectome_service_clone.clone();
                            let id = cortical_id_clone.clone();
                            Box::pin(async move {
                                service.cortical_area_exists(&id).await
                            })
                        })
                    }
                    .map_err(|e| {
                        error!("🦀 [REGISTRATION] Failed to check existence for area {}: {}", i, e);
                        format!("Failed to check cortical area existence: {}", e)
                    })?;
                    
                    if exists {
                        info!("🦀 [REGISTRATION] Cortical area {} already exists (cortical_id: {})", i, cortical_id_base64);
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
                    info!("🦀 [REGISTRATION] Creating {} cortical areas for {}...", create_params_list.len(), sensory_unit.get_snake_case_name());
                    
                    let result = if let Ok(handle) = tokio::runtime::Handle::try_current() {
                        handle.block_on(genome_service.create_cortical_areas(create_params_list.clone()))
                    } else {
                        let rt = tokio::runtime::Runtime::new()
                            .map_err(|e| format!("Failed to create tokio runtime: {}", e))?;
                        rt.block_on(genome_service.create_cortical_areas(create_params_list.clone()))
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
                            error!("🦀 [REGISTRATION] ❌ Failed to create cortical areas for {}: {}", sensory_unit.get_snake_case_name(), e);
                            return Err(format!("Failed to create cortical areas for {}: {}", sensory_unit.get_snake_case_name(), e));
                        }
                    }
                } else {
                    info!("🦀 [REGISTRATION] All {} cortical areas already exist", sensory_unit.get_snake_case_name());
                }
            } else if number_areas == 1 {
                // Handle single cortical area - use the same generic approach
                let cortical_id_base64 = base_cortical_id.as_base_64();
                
                // Get topology for single area (index 0)
                let topology = sensory_unit.get_unit_default_topology();
                let (width, height, channels, x, y, z) = if let Some(unit_topology) = topology.get(&0) {
                    let dims = unit_topology.channel_dimensions_default;
                    let pos = unit_topology.relative_position;
                    (dims[0] as usize, dims[1] as usize, dims[2] as usize, pos[0], pos[1], pos[2])
                } else {
                    // Fallback to dimensions from vision capability
                    (vision.dimensions.0, vision.dimensions.1, vision.channels, 0, 0, 0)
                };
                
                // Check if area exists
                debug!("🦀 [REGISTRATION] Checking IPU area existence for '{}' (cortical_id: {})", area_name, cortical_id_base64);
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
                debug!("🦀 [REGISTRATION] IPU area '{}' exists: {}", area_name, exists);

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
                    info!("🦀 [REGISTRATION] Auto-creating missing IPU area '{}'", area_name);
                    
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
                            info!("🦀 [REGISTRATION] ✅ Successfully created IPU area '{}'", area_name);
                            ipu_statuses.push(CorticalAreaStatus {
                                area_name: area_name.clone(),
                                cortical_id: cortical_id_base64,
                                status: AreaStatus::Created,
                                dimensions: Some((width, height, channels)),
                                message: Some("Auto-created during registration".to_string()),
                            });
                        }
                        Err(e) => {
                            error!("🦀 [REGISTRATION] ❌ Failed to create IPU area '{}': {}", area_name, e);
                            ipu_statuses.push(CorticalAreaStatus {
                                area_name: area_name.clone(),
                                cortical_id: cortical_id_base64,
                                status: AreaStatus::Error,
                                dimensions: None,
                                message: Some(format!("Creation failed: {}", e)),
                            });
                            return Err(format!("Failed to create IPU area '{}': {}", area_name, e));
                        }
                    }
                } else {
                    warn!("🦀 [REGISTRATION] ⚠️ IPU area '{}' is missing and auto-create is disabled", area_name);
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

        // Handle OPU areas (from motor capabilities)
        if let Some(ref motor) = capabilities.motor {
            for area_name in &motor.source_cortical_areas {
                let cortical_id_base64 = self.area_name_to_cortical_id(area_name)?;
                
                // Check if area exists (blocking call)
                // Use helper function to safely call async code from sync context
                debug!("🦀 [REGISTRATION] Checking OPU area existence for '{}' (cortical_id: {})", area_name, cortical_id_base64);
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
                debug!("🦀 [REGISTRATION] OPU area '{}' exists: {}", area_name, exists);

                if exists {
                    info!("🦀 [REGISTRATION] OPU area '{}' already exists", area_name);
                    opu_statuses.push(CorticalAreaStatus {
                        area_name: area_name.clone(),
                        cortical_id: cortical_id_base64,
                        status: AreaStatus::Existing,
                        dimensions: Some((motor.output_count, 1, 1)),  // Default OPU dimensions
                        message: None,
                    });
                } else if self.auto_create_missing_areas {
                    // Create missing OPU area
                    info!("🦀 [REGISTRATION] Auto-creating missing OPU area '{}'", area_name);
                    
                    let create_params = feagi_services::types::CreateCorticalAreaParams {
                        cortical_id: cortical_id_base64.clone(),
                        name: area_name.clone(),
                        dimensions: (motor.output_count, 1, 1),  // OPU: output_count x 1 x 1
                        position: (0, 0, 0),  // Default position in root region
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
                            info!("🦀 [REGISTRATION] ✅ Successfully created OPU area '{}'", area_name);
                            opu_statuses.push(CorticalAreaStatus {
                                area_name: area_name.clone(),
                                cortical_id: cortical_id_base64,
                                status: AreaStatus::Created,
                                dimensions: Some((motor.output_count, 1, 1)),
                                message: Some("Auto-created during registration".to_string()),
                            });
                        }
                        Err(e) => {
                            error!("🦀 [REGISTRATION] ❌ Failed to create OPU area '{}': {}", area_name, e);
                            opu_statuses.push(CorticalAreaStatus {
                                area_name: area_name.clone(),
                                cortical_id: cortical_id_base64,
                                status: AreaStatus::Error,
                                dimensions: None,
                                message: Some(format!("Creation failed: {}", e)),
                            });
                            return Err(format!("Failed to create OPU area '{}': {}", area_name, e));
                        }
                    }
                } else {
                    warn!("🦀 [REGISTRATION] ⚠️ OPU area '{}' is missing and auto-create is disabled", area_name);
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
            info!("🦀 [REGISTRATION] Agent requests transport: {}", requested_transport);
            
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
            info!("🦀 [REGISTRATION] ✅ Transport '{}' is supported", requested_transport);
        }

        // Parse capabilities
        let capabilities = self.parse_capabilities(&request.capabilities)?;

        // Check and ensure required cortical areas exist (auto-create if enabled)
        let cortical_areas_availability = self.ensure_cortical_areas_exist(&capabilities)?;

        // Allocate SHM paths ONLY if agent didn't explicitly choose a non-SHM transport
        let mut shm_paths = HashMap::new();
        let mut allocated_capabilities = capabilities.clone();
        
        let should_provide_shm = match request.chosen_transport.as_ref().map(|s| s.as_str()) {
            Some("websocket") | Some("zmq") => false, // Agent explicitly chose non-SHM transport
            Some("shm") | Some("hybrid") | None => true, // Agent wants SHM or didn't specify
            Some(_) => false, // Unknown transport, don't offer SHM
        };

        if should_provide_shm {
            if let Some(ref mut sensory) = allocated_capabilities.sensory {
                let shm_path = format!(
                    "{}/feagi-shm-{}-sensory.bin",
                    self.shm_base_path, request.agent_id
                );
                sensory.shm_path = Some(shm_path.clone());
                shm_paths.insert("sensory".to_string(), shm_path);
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
            info!("🦀 [REGISTRATION] Skipping SHM paths - agent chose transport: {:?}", request.chosen_transport);
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
            info!("🦀 [REGISTRATION] Agent '{}' chose transport: {}", request.agent_id, chosen);
        }

        // Register in registry
        info!("🦀 [REGISTRATION] 🔍 Registering agent '{}' in AgentRegistry...", request.agent_id);
        self.agent_registry
            .write()
            .register(agent_info.clone())
            .map_err(|e| format!("Failed to register agent: {}", e))?;
        
        // Verify registration
        let registry_count = self.agent_registry.read().get_all().len();
        let all_agents: Vec<String> = self.agent_registry.read().get_all().iter().map(|a| a.agent_id.clone()).collect();
        info!("🦀 [REGISTRATION] ✅ Agent '{}' registered in AgentRegistry (total agents: {})", request.agent_id, registry_count);
        info!("🦀 [REGISTRATION] Registry contents: {:?}", all_agents);
        info!("🦀 [REGISTRATION] Registry pointer: {:p}", &*self.agent_registry as *const _);

        // Register with burst engine's sensory agent manager (if sensory capability exists)
        if let Some(ref sensory) = agent_info.capabilities.sensory {
            if let Some(sensory_mgr_lock) = self.sensory_agent_manager.lock().as_ref() {
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
                    let area_mapping: HashMap<CorticalID, u32> = sensory.cortical_mappings
                        .iter()
                        .filter_map(|(name, &idx)| {
                            match CorticalID::try_from_bytes(&{
                                let mut bytes = [b'\0'; 8];  // Use null bytes, not spaces
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
                        })
                        .collect();
                    
                    let config = feagi_burst_engine::AgentConfig {
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
                    warn!("🦀 [REGISTRATION] ⚠️  Sensory capability exists but no SHM path");
                }
            } else {
                warn!("🦀 [REGISTRATION] ⚠️  Sensory agent manager not connected - skipping burst engine registration");
            }
        }

        // DEBUG: Log all capabilities received
        info!("🦀 [REGISTRATION] 📋 DEBUG: Full capabilities for '{}': {:?}", request.agent_id, agent_info.capabilities);
        
        // Register motor subscriptions with burst engine (if motor capability exists)
        if let Some(ref motor) = agent_info.capabilities.motor {
            info!(
                "🦀 [REGISTRATION] 🎮 Motor capability DETECTED for '{}': modality='{}', output_count={}, source_cortical_areas={:?}",
                request.agent_id, motor.modality, motor.output_count, motor.source_cortical_areas
            );
            
            if let Some(burst_runner_lock) = self.burst_runner.lock().as_ref() {
                // Convert cortical area names to proper 8-byte CorticalID strings
                // SDK may send either plain names ("omot") or base64 encoded IDs ("b21vdAQAAAA=")
                use feagi_data_structures::genomic::cortical_area::CorticalID;
                use base64::{Engine as _, engine::general_purpose};
                
                let mut cortical_ids: AHashSet<String> = AHashSet::new();
                for area_input in &motor.source_cortical_areas {
                    info!("🦀 [REGISTRATION] 🎮 Processing motor cortical area: '{}'", area_input);
                    
                    // Try to parse as base64 first (if coming from Python SDK)
                    let cortical_id = if let Ok(decoded) = general_purpose::STANDARD.decode(area_input) {
                        info!("🦀 [REGISTRATION] 🎮   → Decoded as base64: {} bytes: {:02x?}", decoded.len(), decoded);
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
                            warn!("🦀 [REGISTRATION] ⚠️   → Wrong length: {} (expected 8)", decoded.len());
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
                                warn!("🦀 [REGISTRATION] ⚠️   → CorticalID validation failed: {:?}", e);
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
                        warn!("🦀 [REGISTRATION] ❌ Failed to create cortical ID from '{}'", area_input);
                    }
                }
                
                info!(
                    "🦀 [REGISTRATION] 🎮 Registering {} cortical IDs for motor subscriptions",
                    cortical_ids.len()
                );
                
                burst_runner_lock.read().register_motor_subscriptions(
                    request.agent_id.clone(),
                    cortical_ids.clone(),
                );
                
                info!("🦀 [REGISTRATION] ✅ Motor subscriptions CONFIRMED registered for '{}'", request.agent_id);
            } else {
                info!("🦀 [REGISTRATION] ⚠️ Agent {} has motor capability but burst runner not connected yet", request.agent_id);
            }
        } else {
            info!("🦀 [REGISTRATION] 🎮 DEBUG: Agent {} has NO motor capability in registration", request.agent_id);
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
            info!("🦀 [REGISTRATION] ✅ Dynamic gating callback found, invoking for agent: {}", request.agent_id);
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
        info!("🦀 [REGISTRATION] Available transports: {} (ZMQ + {})", 
              transports.len(), 
              if self.ws_enabled { "WebSocket" } else { "no WebSocket" });
        
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
        // Try to deserialize directly from JSON first (handles new agent SDK format)
        if let Ok(capabilities) = serde_json::from_value::<AgentCapabilities>(caps_json.clone()) {
            return Ok(capabilities);
        }

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
            
            let channels = vision
                .get("channels")
                .and_then(|v| v.as_u64())
                .unwrap_or(3) as usize;
            
            let target_cortical_area = vision
                .get("target_cortical_area")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            
            if !target_cortical_area.is_empty() {
                capabilities.vision = Some(VisionCapability {
                    modality,
                    dimensions,
                    channels,
                    target_cortical_area,
                });
            }
        }

        // Parse legacy sensory capability
        if let Some(sensory) = caps_json.get("sensory") {
            if let Some(rate_hz) = sensory.get("rate_hz").and_then(|v| v.as_f64()) {
                capabilities.sensory = Some(SensoryCapability {
                    rate_hz,
                    shm_path: None,
                    cortical_mappings: HashMap::new(),
                });
            }
        }

        // Parse motor capability (support both legacy and new format)
        if let Some(motor) = caps_json.get("motor") {
            // Check if motor is enabled (legacy format) or if it exists (new format)
            let is_enabled = motor
                .get("enabled")
                .and_then(|v| v.as_bool())
                .unwrap_or(true); // Default to true if 'enabled' key doesn't exist (new format)
            
            if is_enabled {
                // Extract source_cortical_areas from JSON
                let source_cortical_areas: Vec<String> = motor
                    .get("source_cortical_areas")
                    .and_then(|v| v.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|v| v.as_str().map(|s| s.to_string()))
                            .collect()
                    })
                    .unwrap_or_else(|| vec!["omot00".to_string()]); // Default to omot00 for backward compatibility
                
                capabilities.motor = Some(MotorCapability {
                    modality: motor
                        .get("modality")
                        .and_then(|v| v.as_str())
                        .unwrap_or("generic")
                        .to_string(),
                    output_count: motor
                        .get("output_count")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(source_cortical_areas.len() as u64) as usize,
                    source_cortical_areas,
                });
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
                info!("🦀 [REGISTRATION] ✅ Invoking dynamic gating callback for deregistration: {}", agent_id);
                callback(agent_id.to_string());
            } else {
                debug!("[REGISTRATION] No dynamic gating deregistration callback set");
            }
        }

        result
    }

    /// Process heartbeat
    pub fn process_heartbeat(&self, agent_id: &str) -> Result<String, String> {
        use tracing::debug;
        
        debug!("💓 [REGISTRATION] Processing heartbeat for '{}'", agent_id);
        
        self.agent_registry
            .write()
            .heartbeat(agent_id)
            .map(|_| {
                debug!("💓 [REGISTRATION] Heartbeat successfully recorded for '{}'", agent_id);
                format!("Heartbeat recorded for {}", agent_id)
            })
            .map_err(|e| {
                use tracing::warn;
                warn!("⚠️ [REGISTRATION] Heartbeat failed for '{}': {}", agent_id, e);
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
        let handler = RegistrationHandler::new(registry.clone(), 8000, 8001, 8002);

        let request = RegistrationRequest {
            agent_id: "test-agent".to_string(),
            agent_type: "both".to_string(),
            capabilities: serde_json::json!({
                "sensory": {"rate_hz": 30.0},
                "motor": {"enabled": true, "rate_hz": 20.0, "modality": "servo", "output_count": 2}
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
impl RegistrationHandlerTrait for RegistrationHandler {
    fn process_registration(&self, request: RegistrationRequest) -> Result<RegistrationResponse, String> {
        self.process_registration_impl(request)
    }
}
