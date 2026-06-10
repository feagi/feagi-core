use core::marker::PhantomData;
use feagi_structures::feagi_data::shared_quantization_sets::FeagiGlobalQuantization;
use crate::neural_processing_unit_data_structures::burst_engine_cluster::burst_engines::engines::burst_engine_fixed_interface::BurstEngineFixedInterface;
use crate::npu_descriptors::NPUGlobalBurstCounter;

pub(crate) struct BurstEngineCpuRayon<FGQ: FeagiGlobalQuantization> {
    // tables
    // vars
    _p: PhantomData<FGQ>,
}



impl<FGQ: FeagiGlobalQuantization> BurstEngineCpuRayon<FGQ>
{
    
}

impl<FGQ: FeagiGlobalQuantization> BurstEngineFixedInterface<FGQ> for BurstEngineCpuRayon<FGQ>
{
    fn run_bursts(&mut self, number_bursts: NPUGlobalBurstCounter<FGQ::GlobalBurstIndexQuant>) -> usize {
        return 0
    }

    fn inject_sensor_data(&mut self) {
        todo!()
    }

    fn force_neuron_indexes_to_fire_upcoming_burst(&mut self) {
        todo!()
    }

    fn extract_visualization_data(&mut self) {
        todo!()
    }

    fn extract_motor_data(&mut self) {
        todo!()
    }
}



