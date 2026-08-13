use core::time::Duration;
use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
use crate::burst_engine::burst_engine::BurstEngine;
use crate::burst_engine::composable_implementations::tokio_rayon::tokio_rayon_burst_engine::TokioRayonBurstEngine;
use crate::burst_engine_enum::ComposableBurstEngineEnum;
use crate::npu::burst_engine::implementations::null_burst_engine::NullBurstEngine;
use crate::npu::burst_engine_worker::burst_engine_commands::BurstEngineWorkerCommand;
use crate::npu::burst_engine_worker::independent_burst_engine_worker::IndependentBurstEngineWorker;

/// Number of bursts per second
pub type BurstFrequency = f64;

// TODO multi burst engine support exposure
// TODO allow defining types of burst engine


fn test() {

    let handle = std::thread::spawn(neuron_processing_unit);

}

fn neuron_processing_unit() {

}







pub struct NeuronProcessingUnitComposable<FIQ: FeagiIndexQuantization> {
    burst_engine: Option<ComposableBurstEngineEnum<FIQ>>,
    command_rx: std::sync::mpsc::Receiver<NeuronProcessingUnitRuntimeCommand>,
    time_between_bursts: Duration
}

impl<FIQ: FeagiIndexQuantization> NeuronProcessingUnitComposable<FIQ> {


    pub fn new() -> NeuronProcessingUnitComposableReturn<FIQ> {

    }

    pub fn neuron_processing_unit_loop(&mut self)
    {
        let burst_engine = self.burst_engine.take().unwrap();
        let burst_engine_worker_instantiation = IndependentBurstEngineWorker::new(burst_engine);

        let mut burst_engine_worker = burst_engine_worker_instantiation.worker;

        let worker_handle = std::thread::spawn(move || {
            burst_engine_worker.burst_engine_loop()
        });

        loop {

            let command = self.command_rx.try_recv();

            let current_time = std::time::Instant::now();


            if let Ok(command) = command {
                match command {
                    NeuronProcessingUnitRuntimeCommand::SetTargetFrequency(freq) => {
                        self.time_between_bursts = Duration::from_secs_f64(1.0 / freq);
                    }
                    NeuronProcessingUnitRuntimeCommand::Pause => {



                        break;
                    }
                }
            }

            // TODO check type of error



        }
    }
}

pub struct NeuronProcessingUnitComposableReturn<FIQ: FeagiIndexQuantization> {
    pub npu: NeuronProcessingUnitComposable<FIQ>,
    pub command_tx: std::sync::mpsc::SyncSender<NeuronProcessingUnitRuntimeCommand>,
}

/// While the NPU is running, you can send these commands to alter its running state
#[derive(Clone, Copy, PartialEq)]
pub enum NeuronProcessingUnitRuntimeCommand {
    /// Change the frequency the burst engines aim to run at
    SetTargetFrequency(BurstFrequency),
    /// Pauses all the engines in place
    Pause
}


