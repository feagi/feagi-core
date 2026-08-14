use crate::burst_engine_enum::ComposableBurstEngineEnum;
use crate::npu::burst_engine_worker::burst_engine_commands::{BurstEngineWorkerCommand, BurstEngineWorkerKernelCommand};
use feagi_data::bidirectional_channel_queue::{BiDirectionalChannelQueue, BidirectionalChannelSide};
use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
use crate::npu::burst_engine_worker::independent_burst_engine_worker::independent_burst_engine_worker;
// TODO we should have a substruct input

// TODO actually have a vector input lol

pub fn burst_engine_worker_pool<'a, FIQ: FeagiIndexQuantization>(
    burst_engines: (
        ComposableBurstEngineEnum<FIQ>,
        BidirectionalChannelSide<'a, ()>,
        BidirectionalChannelSide<'a, ()>,
        BidirectionalChannelSide<'a, ()>,
    ),
) {

    let mut worker_queue: BiDirectionalChannelQueue<BurstEngineWorkerCommand, 1, 1>  = BiDirectionalChannelQueue::new();

    let (mut command_tx, receive_rx) = worker_queue.split();

    let worker_handle = std::thread::spawn(
        || {
            independent_burst_engine_worker(
                burst_engines.0,
                receive_rx,
                burst_engines.1,
                burst_engines.2,
                burst_engines.3
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
