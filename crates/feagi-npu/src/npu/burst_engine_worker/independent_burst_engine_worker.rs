
// TODO We probably shouldnt using channels directly cause it enforces dependency on std
// TODO we shouldnt be reallocating memory per burst / command, we should be passing a block of memory back and forth
// TODO We probably shouldnt be doing inputs / outputs one at a time, we should consolidate inputs / outputs
// TODO some actual error checking would be nice

use feagi_data::bidirectional_channel_queue::BidirectionalChannelSide;
use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
use crate::burst_engine_enum::ComposableBurstEngineEnum;
use crate::npu::burst_engine_worker::burst_engine_commands::BurstEngineWorkerCommand;


pub fn independent_burst_engine_worker<'a, FIQ: FeagiIndexQuantization>(
    mut burst_engine: ComposableBurstEngineEnum<FIQ>,
    mut incoming_command_buffer: BidirectionalChannelSide<'a, BurstEngineWorkerCommand>,
    mut incoming_sensor_buffer: BidirectionalChannelSide<'a, ()>,
    mut outgoing_motor_buffer: BidirectionalChannelSide<'a, ()>,
    mut outgoing_visualization_buffer: BidirectionalChannelSide<'a, ()>,
)
{
    loop {

        if let Some(command_buffer) = incoming_command_buffer.dequeue() {
            match command_buffer {
                BurstEngineWorkerCommand::RunKernel(kernel_command) => {
                    //burst_engine.run_burst().await
                }
                BurstEngineWorkerCommand::EditConnectome() => {
                    // TODO drain all incoming data to avoid errors
                    // TODO send edit request over
                }
                BurstEngineWorkerCommand::CommitSudoku => {
                    break; // break out of the loop
                }
            }
        }

        if let Some(incoming_sensor) = incoming_sensor_buffer.dequeue() {
            // TODO inject sensor, return
        }

        if let Some(outgoing_motor) = outgoing_motor_buffer.dequeue() {
            // TODO write to motor, return
        }

        if let Some(outgoing_visualisation) = outgoing_visualization_buffer.dequeue() {
            // TODO write to motor, return
        }

        // No command yet, defer
        std::thread::yield_now(); // TODO different behavior depending on platform, spinning may be better for latency (std::hint::spin_loop() )
    }
}