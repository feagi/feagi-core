
// TODO We probably shouldnt using channels directly cause it enforces dependency on std
// TODO we shouldnt be reallocating memory per burst / command, we should be passing a block of memory back and forth
// TODO We probably shouldnt be doing inputs / outputs one at a time, we should consolidate inputs / outputs
// TODO some actual error checking would be nice

use feagi_data::bidirectional_channel_queue::BidirectionalChannelSide;
use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
use crate::npu::burst_engine::ComposableBurstEngineEnum;
use crate::npu::burst_engine_worker::burst_engine_commands::BurstEngineWorkerCommand;
use crate::wnpu::agents::data_exchange::force_fire::VoxelForceFire;
use crate::wnpu::agents::data_exchange::visualization::VoxelVisualization;
use crate::wnpu::agents::data_exchange::voxel_potentials::CorticalAreaVoxelPotentials;

pub fn independent_burst_engine_worker<'a, FIQ: FeagiIndexQuantization>(
    mut burst_engine: ComposableBurstEngineEnum<FIQ>,
    mut incoming_command_buffer: BidirectionalChannelSide<'a, BurstEngineWorkerCommand>,
    mut incoming_sensor_buffer: BidirectionalChannelSide<'a, CorticalAreaVoxelPotentials<u32, f32>>,
    mut incoming_force_fire_buffer: BidirectionalChannelSide<'a, VoxelForceFire<u32>>,
    mut outgoing_motor_buffer: BidirectionalChannelSide<'a, CorticalAreaVoxelPotentials<u32, f32>>,
    mut outgoing_visualization_buffer: BidirectionalChannelSide<'a, VoxelVisualization<u32>>,
)
{
    loop {

        if let Some(command_buffer) = incoming_command_buffer.dequeue() {
            match command_buffer {
                BurstEngineWorkerCommand::RunKernel(kernel_command) => {

                    // TODO depending on kernel command, we may not want to do all data exchanges

                    if let Some(incoming_sensor) = incoming_sensor_buffer.dequeue() {
                        // TODO inject sensor, return
                    }
                    if let Some(incoming_force_fire) = incoming_force_fire_buffer.dequeue() {
                        // TODO inject force fire, return
                    }

                    //burst_engine.run_burst().await

                    if let Some(outgoing_motor) = outgoing_motor_buffer.dequeue() {
                        // TODO write to motor, return
                    }

                    if let Some(outgoing_visualisation) = outgoing_visualization_buffer.dequeue() {

                    }
                }
                BurstEngineWorkerCommand::EditConnectome() => {

                    incoming_sensor_buffer.completely_return_queue(); // TODO error handling

                    // TODO send edit request over
                }
                BurstEngineWorkerCommand::CommitSudoku => {
                    break; // break out of the loop
                }
            }
        }
        // No command yet, defer
        std::thread::yield_now(); // TODO different behavior depending on platform, spinning may be better for latency (std::hint::spin_loop() )
    }
}

pub struct BurstEngineWorkerBuffer<'a> {
    pub incoming_command_buffer: BidirectionalChannelSide<'a, BurstEngineWorkerCommand>,
    pub incoming_sensor_buffer: BidirectionalChannelSide<'a, CorticalAreaVoxelPotentials<u32, f32>>,
    pub incoming_force_fire_buffer: BidirectionalChannelSide<'a, VoxelForceFire<u32>>,
    pub outgoing_motor_buffer: BidirectionalChannelSide<'a, CorticalAreaVoxelPotentials<u32, f32>>,
    pub outgoing_visualization_buffer: BidirectionalChannelSide<'a, VoxelVisualization<u32>>,
}