use core::marker::PhantomData;
use feagi_data::quantization_levels::feagi_index_quantization::FeagiGlobalQuantization;

pub struct BurstEngineCpuRayon<FGQ: FeagiGlobalQuantization> {
    // tables
    // vars
    _p: PhantomData<FGQ>,
}


impl<FGQ: FeagiGlobalQuantization> BurstEngineCpuRayon<FGQ>
{
    
}

impl AsyncBurstEngineTrait

/*

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





 */