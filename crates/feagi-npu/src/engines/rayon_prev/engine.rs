use core::marker::PhantomData;
use feagi_data::quantization_levels::feagi_index_quantization::FeagiGlobalQuantization;

pub struct BurstEngineCpuRayon<FIQ: FeagiGlobalQuantization> {
    // tables
    // vars
    _p: PhantomData<FIQ>,
}


impl<FIQ: FeagiGlobalQuantization> BurstEngineCpuRayon<FIQ>
{
    
}

impl AsyncBurstEngineTrait

/*

impl<FIQ: FeagiGlobalQuantization> BurstEngineFixedInterface<FIQ> for BurstEngineCpuRayon<FIQ>
{
    fn run_bursts(&mut self, number_bursts: NPUWrappedBurstEngineBurstIndex<FIQ::GlobalBurstIndexQuant>) -> usize {
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