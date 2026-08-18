use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
use crate::standard::npu::burst_engine::burst_engine_communication::{EngineConnectomeEditRequest, EngineConnectomeEditResponse, KernelCommand};
use crate::standard::npu::burst_engine::burst_engine_error::FeagiBurstEngineError;
// TODO we should probably consolidate a lot of these functions as in the case of WGPU, these seperate functions
// TODO result in many additional draw calls. Do this later as this setup does allow easier debugging for now
// TODO actually this may not be the case as like this we can use free time to exchange data. We should bench

/// Burst engines that run in sync (CPU bound)
pub trait SyncBurstEngine<FIQ: FeagiIndexQuantization>:
Send
{
    /// Runs a given kernel computation
    fn run_kernel(&mut self, kernel: KernelCommand) -> Result<(), FeagiBurstEngineError>;

    /// Inject MP into neurons, used for importing sensor data or reading in intra-engine cortical data
    fn inject_membrane_potentials(&mut self, injections: ()) -> Result<(), FeagiBurstEngineError>;

    /// Force given neurons to fire in the upcoming burst
    fn inject_force_firings(&mut self, injections: ()) -> Result<(), FeagiBurstEngineError>;

    /// Extract MP from neurons, used for exporting motor data or dumping in intra-engine cortical data
    fn extract_membrane_potentials(&mut self, extractions: ()) -> Result<(), FeagiBurstEngineError>;

    /// Extract visualization data for cortical areas
    fn extract_visualizations(&mut self, visualizations: ()) -> Result<(),FeagiBurstEngineError>;

    /// Initialize or free context for handling agent data sync.
    fn edit_agent_registrations(&mut self, agent_registration_context: ()) -> Result<(), FeagiBurstEngineError>;

    /// Edit the connectome in some manner
    fn edit_connectome(&mut self, burst_engine_connectome_request: EngineConnectomeEditRequest<FIQ>) -> Result<EngineConnectomeEditResponse<FIQ>, FeagiBurstEngineError>; // TODO return state of what changed
}

/// Burst engines that are async (generally external device bound)
pub trait AsyncBurstEngine<FIQ: FeagiIndexQuantization>:
Send
{
    /// Runs a given kernel computation
    async fn run_kernel(&mut self, kernel: KernelCommand) -> Result<(), FeagiBurstEngineError>;

    /// Inject MP into neurons, used for importing sensor data or reading in intra-engine cortical data
    async fn inject_membrane_potentials(&mut self, injections: ()) -> Result<(), FeagiBurstEngineError>;

    /// Force given neurons to fire in the upcoming burst
    async fn inject_force_firings(&mut self, injections: ()) -> Result<(), FeagiBurstEngineError>;

    /// Extract MP from neurons, used for exporting motor data or dumping in intra-engine cortical data
    async fn extract_membrane_potentials(&mut self, extractions: ()) -> Result<(), FeagiBurstEngineError>;

    /// Extract visualization data for cortical areas
    async fn extract_visualizations(&mut self, visualizations: ()) -> Result<(), FeagiBurstEngineError>;

    /// Initialize or free context for handling agent data sync.
    async fn edit_agent_registrations(&mut self, agent_registration_context: ()) -> Result<(), FeagiBurstEngineError>;

    /// Edit the connectome in some manner
    async fn edit_connectome(&mut self, burst_engine_connectome_request: EngineConnectomeEditRequest<FIQ>) -> Result<EngineConnectomeEditResponse<FIQ>, FeagiBurstEngineError>; // TODO return state of what changed
}