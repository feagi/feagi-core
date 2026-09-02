use std::future::Future;
use std::marker::PhantomData;
use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
use feagi_models::wrapped_indexes::BurstIndex;
use feagi_npu_burst_core::burst_engine_definitions::burst_engine::{BurstEngine, NonComposableBurstEngine};
use feagi_npu_burst_core::burst_engine_definitions::burst_phase_output::BurstPhaseOutput;
use feagi_npu_burst_core::burst_engine_definitions::burst_phases::RunBurstPhase;
use feagi_npu_burst_core::errors::{BurstEngineError, FeagiFailPhase};

pub struct ESP32BoardESP32BurstEngine<FIQ: FeagiIndexQuantization> {
    _p: core::marker::PhantomData<FIQ>
}

impl<FIQ: FeagiIndexQuantization> BurstEngine<FIQ> for ESP32BoardESP32BurstEngine<FIQ> {
    fn execute_phase(&mut self, phases: RunBurstPhase, burst_index: BurstIndex<FIQ::BurstIndexQuant>)
        -> impl Future<Output=Result<BurstPhaseOutput<FIQ>, BurstEngineError>> {
        core::future::ready(Err(FeagiFailPhase::new("not implemented!").into()))
    }
}

impl<FIQ: FeagiIndexQuantization> NonComposableBurstEngine<FIQ> for ESP32BoardESP32BurstEngine<FIQ> {}

impl<FIQ: FeagiIndexQuantization> ESP32BoardESP32BurstEngine<FIQ> {
    // TODO vars?

    pub fn new() -> Self {
        Self {
            _p: PhantomData
        }
    }
}