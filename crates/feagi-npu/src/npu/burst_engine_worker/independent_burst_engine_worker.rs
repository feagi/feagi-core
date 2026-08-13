
// TODO We probably shouldnt using channels directly cause it enforces dependency on std
// TODO we shouldnt be reallocating memory per burst / command, we should be passing a block of memory back and forth
// TODO We probably shouldnt be doing inputs / outputs one at a time, we should consolidate inputs / outputs
// TODO some actual error checking would be nice

use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
use crate::burst_engine::burst_engine::BurstEngine;
use crate::burst_engine_enum::ComposableBurstEngineEnum;
use crate::npu::burst_engine_worker::burst_engine_commands::BurstEngineWorkerCommand;

pub struct IndependentBurstEngineWorker<FIQ: FeagiIndexQuantization> {
    burst_engine: ComposableBurstEngineEnum<FIQ>,
    command_rx: std::sync::mpsc::Receiver<BurstEngineWorkerCommand>,
    completion_tx: std::sync::mpsc::SyncSender<()>,
    visualization_tx: std::sync::mpsc::SyncSender<()>,
}

impl<FIQ: FeagiIndexQuantization> IndependentBurstEngineWorker<FIQ> {

    /// Creates a new `IndependentBurstEngineWorker` and its in/out channels
    pub fn new(burst_engine: ComposableBurstEngineEnum<FIQ>) -> IndependentBurstEngineWorkerInstantiation<FIQ> {
        let (command_tx, command_rx) = std::sync::mpsc::sync_channel(1);
        let (completion_tx, completion_rx) = std::sync::mpsc::sync_channel(1);

        //let (sensor_tx, sensor_rx) = std::sync::mpsc::sync_channel(1);
        //let (motor_tx, motor_rx) = std::sync::mpsc::sync_channel(1);
        let (visualization_tx, visualization_rx) = std::sync::mpsc::sync_channel(1);

        // TODO probe, force fire

        IndependentBurstEngineWorkerInstantiation {
            worker: IndependentBurstEngineWorker{
                burst_engine,
                command_rx,
                completion_tx,
                visualization_tx,
            },
            command_tx,
            completion_rx,
            visualization_rx,
        }
    }

    /// Starts a loop internally
    pub fn burst_engine_loop(&mut self)
    {
        loop {
            let command = self.command_rx.recv().unwrap();

            match command {

                BurstEngineWorkerCommand::RunFullBurst => {
                    self.burst_engine.run_burst().await;
                }
                BurstEngineWorkerCommand::CommitSudoku => {
                    // TODO handle exiting a bit more gracefully
                    self.completion_tx.send(()).unwrap();
                    break;
                }
                
                
                #[allow(unreachable_patterns)]
                _ => {
                    panic!("Unexpected command!");
                }
            }
            
            self.completion_tx.send(()).unwrap();
        }
    }

}

// TODO proper deallocation


/// a created `IndependentBurstEngineWorker` and the channel interfaces to communicate with it
pub struct IndependentBurstEngineWorkerInstantiation<FIQ: FeagiIndexQuantization> {
    pub worker: IndependentBurstEngineWorker<FIQ>,
    pub command_tx: std::sync::mpsc::SyncSender<BurstEngineWorkerCommand>,
    pub completion_rx: std::sync::mpsc::Receiver<()>,
    pub visualization_rx: std::sync::mpsc::Receiver<()>
}