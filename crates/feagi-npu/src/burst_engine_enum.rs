
use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
use crate::burst_engine::burst_engine::BurstEngine;
use crate::burst_engine::composed_burst_engine::ComposableBurstEngine;
use crate::burst_engine::composable_implementations::tokio_rayon::tokio_rayon_burst_engine::TokioRayonBurstEngine;

/*
macro_rules! proxy_trait {
    ($trait_expr:expr) => {
        match self {

        }
    };
}

 */


/// All possible composable burst engines in a fast access enum, using the same common trait access
pub enum ComposableBurstEngineEnum<FIQ: FeagiIndexQuantization> {
    TokioRayonBurstEngine(TokioRayonBurstEngine<FIQ>)
}


impl<FIQ: FeagiIndexQuantization> ComposableBurstEngine<FIQ> for ComposableBurstEngineEnum<FIQ> {
    fn add_cortical_areas(&mut self) {
        match self {
            ComposableBurstEngineEnum::TokioRayonBurstEngine(engine) => {
                engine.add_cortical_areas()
            }
        }
    }

    fn remove_cortical_areas(&mut self) {
        todo!()
    }

    fn overwrite_neuron_firings(&mut self) {
        todo!()
    }

    fn add_cortical_mappings(&mut self) {
        todo!()
    }

    fn remove_cortical_mappings(&mut self) {
        todo!()
    }
}

impl<FIQ: FeagiIndexQuantization> BurstEngine<FIQ> for ComposableBurstEngineEnum<FIQ> {
    async fn run_burst(&mut self) {
        todo!()
    }

    fn exchange_data_for_sensor_motor_probes_and_potentials(&mut self) {
        todo!()
    }

    fn import_bridged_synapse_propagation_data(&mut self) {
        todo!()
    }

    fn export_bridged_neuron_mp_data(&mut self) {
        todo!()
    }

    fn import_bridged_neuron_mp_data(&mut self) {
        todo!()
    }

    fn edit_in_place_cortical_areas(&mut self) {
        todo!()
    }
}