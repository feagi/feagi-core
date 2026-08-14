use crate::npu::burst_engine::ComposableBurstEngineEnum;
use crate::npu::burst_engine_worker::burst_engine_commands::{BurstEngineWorkerCommand, BurstEngineWorkerKernelCommand};
use crate::npu::burst_engine_worker::independent_burst_engine_worker::independent_burst_engine_worker;
use feagi_data::bidirectional_channel_queue::{BiDirectionalChannelQueue, BidirectionalChannelSide};
use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;

const CHANNEL_BUFFER_SIZE: usize = 1;

/// Queue storage for one burst-engine worker, owned by the NPU.
pub struct BurstEngineWorkerChannels {
    pub command_queue: BiDirectionalChannelQueue<BurstEngineWorkerCommand, CHANNEL_BUFFER_SIZE, CHANNEL_BUFFER_SIZE>,
    pub incoming_sensor_buffer: BiDirectionalChannelQueue<(), CHANNEL_BUFFER_SIZE, CHANNEL_BUFFER_SIZE>,
    pub outgoing_motor_buffer: BiDirectionalChannelQueue<(), CHANNEL_BUFFER_SIZE, CHANNEL_BUFFER_SIZE>,
    pub outgoing_visualization_buffer: BiDirectionalChannelQueue<(), CHANNEL_BUFFER_SIZE, CHANNEL_BUFFER_SIZE>,
}

impl BurstEngineWorkerChannels {
    pub fn new() -> Self {
        Self {
            command_queue: BiDirectionalChannelQueue::new(),
            incoming_sensor_buffer: BiDirectionalChannelQueue::new(),
            outgoing_motor_buffer: BiDirectionalChannelQueue::new(),
            outgoing_visualization_buffer: BiDirectionalChannelQueue::new(),
        }
    }

    /// Splits every queue into coordinator and worker sides.
    pub fn split_for_pool(
        &mut self,
    ) -> (
        BurstEngineWorkerCoordinatorSide<'_>,
        BurstEngineWorkerWorkerSide<'_>,
    ) {
        let (command_coordinator, command_worker) = self.command_queue.split();
        let (incoming_sensor_coordinator, incoming_sensor_worker) = self.incoming_sensor_buffer.split();
        let (outgoing_motor_coordinator, outgoing_motor_worker) = self.outgoing_motor_buffer.split();
        let (outgoing_visualization_coordinator, outgoing_visualization_worker) =
            self.outgoing_visualization_buffer.split();

        (
            BurstEngineWorkerCoordinatorSide {
                command: command_coordinator,
                incoming_sensor: incoming_sensor_coordinator,
                outgoing_motor: outgoing_motor_coordinator,
                outgoing_visualization: outgoing_visualization_coordinator,
            },
            BurstEngineWorkerWorkerSide {
                command: command_worker,
                incoming_sensor: incoming_sensor_worker,
                outgoing_motor: outgoing_motor_worker,
                outgoing_visualization: outgoing_visualization_worker,
            },
        )
    }
}

impl Default for BurstEngineWorkerChannels {
    fn default() -> Self {
        Self::new()
    }
}

/// NPU/coordinator side of a worker's channels.
pub struct BurstEngineWorkerCoordinatorSide<'a> {
    pub command: BidirectionalChannelSide<'a, BurstEngineWorkerCommand>,
    pub incoming_sensor: BidirectionalChannelSide<'a, ()>,
    pub outgoing_motor: BidirectionalChannelSide<'a, ()>,
    pub outgoing_visualization: BidirectionalChannelSide<'a, ()>,
}

/// Worker-thread side of a worker's channels.
pub struct BurstEngineWorkerWorkerSide<'a> {
    pub command: BidirectionalChannelSide<'a, BurstEngineWorkerCommand>,
    pub incoming_sensor: BidirectionalChannelSide<'a, ()>,
    pub outgoing_motor: BidirectionalChannelSide<'a, ()>,
    pub outgoing_visualization: BidirectionalChannelSide<'a, ()>,
}

/// Runs the coordinator loop while scoped worker threads borrow channel storage from `worker_channels`.
///
/// Workers are joined before this function returns. `coordinator` receives the NPU-owned coordinator
/// sides and is responsible for dispatching commands and routing data to/from each worker.
pub fn burst_engine_worker_pool<'scope, FIQ, F>(
    worker_channels: &'scope mut [BurstEngineWorkerChannels],
    burst_engines: Vec<ComposableBurstEngineEnum<FIQ>>,
    mut coordinator: F,
) where
    FIQ: FeagiIndexQuantization + Send + 'static,
    F: FnMut(&mut [BurstEngineWorkerCoordinatorSide<'scope>]),
{
    assert_eq!(
        worker_channels.len(),
        burst_engines.len(),
        "each burst engine must have a matching worker channel set"
    );

    std::thread::scope(|scope| {
        let mut coordinator_sides = Vec::with_capacity(worker_channels.len());

        for (channels, burst_engine) in worker_channels.iter_mut().zip(burst_engines) {
            let (coordinator_side, worker_side) = channels.split_for_pool();

            scope.spawn(move || {
                independent_burst_engine_worker(
                    burst_engine,
                    worker_side.command,
                    worker_side.incoming_sensor,
                    worker_side.outgoing_motor,
                    worker_side.outgoing_visualization,
                );
            });

            coordinator_sides.push(coordinator_side);
        }

        coordinator(&mut coordinator_sides);
    });
}

/// Default coordinator loop used until the NPU supplies its own synchronization logic.
pub fn default_burst_coordinator_loop(coordinator_sides: &mut [BurstEngineWorkerCoordinatorSide<'_>]) {
    loop {
        for coordinator_side in coordinator_sides.iter_mut() {
            let _ = coordinator_side.command.enqueue(BurstEngineWorkerCommand::RunKernel(
                BurstEngineWorkerKernelCommand::FullNeuronSynapseBurst,
            ));
        }

        // TODO wait for all workers to finish this iteration
        // TODO wait remaining delay
    }
}
