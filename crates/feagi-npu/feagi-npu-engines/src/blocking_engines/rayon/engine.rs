use feagi_data::quantization_levels::feagi_index_quantization::FeagiGlobalQuantization;
use crate::blocking_engines::blocking_engine_interface::BlockingEngine;

pub struct RayonBurstEngine<Q: FeagiGlobalQuantization> {

    _p: core::marker::PhantomData<Q>,
}

impl<Q: FeagiGlobalQuantization> BlockingEngine for RayonBurstEngine<Q> {
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

    fn run_neuron_processing(&mut self, force_update_visualization: bool) {
        todo!()
    }
}

impl<Q: FeagiGlobalQuantization> RayonBurstEngine<Q> {
    
}