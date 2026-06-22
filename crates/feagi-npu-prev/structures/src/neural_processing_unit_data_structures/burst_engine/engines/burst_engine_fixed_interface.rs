use feagi_structures::feagi_data::quantization_levels::feagi_global_quantization::FeagiGlobalQuantization;
use crate::neural_processing_unit_data_structures::wrappers::NPUWrappedBurstEngineBurstIndex;

pub trait BurstEngineFixedInterface<FGD: FeagiGlobalQuantization>
{

    /// Runs a given number of bursts. Stops right after neuron dynamics sim but before consolidation of
    /// firing neurons, as this is the best time for outside interference
    fn run_bursts(&mut self, number_bursts: NPUWrappedBurstEngineBurstIndex<FGD::GlobalBurstIndexQuant>) -> usize; // number microseconds

    fn inject_sensor_data(&mut self);
    
    fn force_neuron_indexes_to_fire_upcoming_burst(&mut self); // should run BEFORE below
    
    fn extract_visualization_data(&mut self); // TODO Bring up issue of noise
    
    fn extract_motor_data(&mut self);
}