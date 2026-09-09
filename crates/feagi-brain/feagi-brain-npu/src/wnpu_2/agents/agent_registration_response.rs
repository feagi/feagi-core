use feagi_data::bidirectional_channel_queue::MpscBidirectionalChannelSide;
use crate::wnpu::agents::data_exchange::force_fire::VoxelForceFire;
use crate::wnpu::agents::data_exchange::visualization::VoxelVisualization;
use crate::wnpu::agents::data_exchange::voxel_potentials::CorticalAreaVoxelPotentials;
// TODO remove fixed quantizations

/// Returned when registering an agent, gives channels to use to send / receive data
pub struct AgentRegistrationResponse {
    pub input_sensor: Option<MpscBidirectionalChannelSide<CorticalAreaVoxelPotentials<u32, f32>>>,
    pub input_force_fire: Option<MpscBidirectionalChannelSide<VoxelForceFire<u32>>>,
    pub output_visualization: Option<MpscBidirectionalChannelSide<VoxelVisualization<u32>>>,
    pub output_motor: Option<MpscBidirectionalChannelSide<CorticalAreaVoxelPotentials<u32, f32>>>,
}

// TODO expand something like this for also passing in edit commands?