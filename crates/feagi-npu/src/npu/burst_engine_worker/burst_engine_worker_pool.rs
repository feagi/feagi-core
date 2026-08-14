use crate::burst_engine_enum::ComposableBurstEngineEnum;
use crate::npu::burst_engine_worker::burst_engine_commands::{BurstEngineWorkerCommand, BurstEngineWorkerKernelCommand};
use feagi_data::bidirectional_channel_queue::{BiDirectionalChannelQueue, BidirectionalChannelSide};
use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
use crate::npu::burst_engine_worker::independent_burst_engine_worker::independent_burst_engine_worker;
// TODO we should have a substruct input

// TODO actually have a vector input lol

pub fn burst_engine_worker_pool<'a, FIQ: FeagiIndexQuantization>(
    burst_engines: BurstEngineWorkerContext<FIQ>,
) {

    let mut worker_queue: BiDirectionalChannelQueue<BurstEngineWorkerCommand, 1, 1>  = BiDirectionalChannelQueue::new();

    let (mut command_tx, receive_rx) = worker_queue.split();

    // split apart the incoming struct
    let BurstEngineWorkerContext {
        burst_engine,
        incoming_sensor_buffer,
        outgoing_motor_buffer,
        outgoing_visualization_buffer,
    } = burst_engines;
    
    
    let worker_handle = std::thread::spawn(
        || {
            independent_burst_engine_worker(
                burst_engine,
                receive_rx,
                incoming_sensor_buffer,
                outgoing_motor_buffer,
                outgoing_visualization_buffer
            )
        }
    );

    loop {
        // TODO match higher rx command system

        // if no other command
        _ = command_tx.enqueue(BurstEngineWorkerCommand::RunKernel(
            BurstEngineWorkerKernelCommand::FullNeuronSynapseBurst
        ));



        // TODO wait remaining delay

    }
}

/// Used to transport context for a burst engine worker
pub struct BurstEngineWorkerContext<'a, FIQ: FeagiIndexQuantization>
{
    pub burst_engine: ComposableBurstEngineEnum<FIQ>,
    pub incoming_sensor_buffer: BidirectionalChannelSide<'a, ()>,
    pub outgoing_motor_buffer: BidirectionalChannelSide<'a, ()>,
    pub outgoing_visualization_buffer: BidirectionalChannelSide<'a, ()>,
}