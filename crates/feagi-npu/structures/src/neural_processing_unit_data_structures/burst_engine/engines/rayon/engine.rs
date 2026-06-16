use core::marker::PhantomData;
use feagi_structures::feagi_data::quantization_levels::feagi_global_quantization::FeagiGlobalQuantization;
use crate::neural_processing_unit_data_structures::burst_engine::engines::burst_engine_fixed_interface::BurstEngineFixedInterface;
use crate::neural_processing_unit_data_structures::wrappers::NPUWrappedBurstEngineBurstIndex;

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
    fn run_bursts(&mut self, number_bursts: NPUWrappedBurstEngineBurstIndex<FGQ::GlobalBurstIndexQuant>) -> usize {
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



