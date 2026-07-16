use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
use crate::engines::blocking::blocking_engine::BlockingEngine;

pub struct RayonBurstEngine<Q: FeagiIndexQuantization> {

    _p: core::marker::PhantomData<Q>,
}

impl<Q: FeagiIndexQuantization> BlockingEngine for RayonBurstEngine<Q> {
    fn exchange_agent_and_mp_data(&mut self) {
        todo!()
    }

    fn run_synapse_processing(&mut self) {
        todo!()
    }

    fn export_fcl_data(&self) {
        todo!()
    }

    fn import_fcl_data(&mut self) {
        todo!()
    }

    fn run_neuron_processing(&mut self, _force_update_visualization: bool) {
        todo!()
    }
}

impl<Q: FeagiIndexQuantization> RayonBurstEngine<Q> {
    pub fn new() -> Self {
        Self {
            _p: core::marker::PhantomData,
        }
    }
}