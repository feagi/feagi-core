use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
use crate::burst_engine::burst_engine::BurstEngine;

// TODO consolidate some functions to lower the number of calls


pub trait ComposableBurstEngine<FIQ: FeagiIndexQuantization>: BurstEngine<FIQ>
{
    fn add_cortical_areas(&mut self);

    fn remove_cortical_areas(&mut self);

    fn overwrite_neuron_firings(&mut self);

    fn add_cortical_mappings(&mut self);

    fn remove_cortical_mappings(&mut self);
}


pub enum BurstEngineEnum<FIQ: FeagiIndexQuantization> {

}

impl<FIQ: FeagiIndexQuantization> BurstEngine<FIQ> for BurstEngineEnum<FIQ> {
    async fn run_bursts(&mut self, number_bursts: FIQ::GlobalBurstIndexQuant) {
        todo!()
    }

    fn exchange_data_for_sensor_motor_probes_and_potentials(&mut self) {
        todo!()
    }

    fn exchange_synapse_propagation_data(&mut self) {
        todo!()
    }

    fn edit_in_place_cortical_areas(&mut self) {
        todo!()
    }
}

impl<FIQ: FeagiIndexQuantization> ComposableBurstEngine<FIQ> for BurstEngineEnum<FIQ> {
    fn add_cortical_areas(&mut self) {
        todo!()
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
