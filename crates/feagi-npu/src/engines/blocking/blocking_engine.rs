use crate::engines::blocking::rayon::RayonBurstEngine;
use feagi_data::feagi_quantization_levels::feagi_index_quantization::FeagiIndexQuantization;

pub(crate) trait BlockingEngine {
    /// Import sensor data, and export motor, sensor, and neuron mp data (all optionally).
    fn exchange_agent_and_mp_data(&mut self);

    /// Potentially consolidate firing neurons, then execute synapse dynamics and merge results
    /// to the FCL
    fn run_synapse_processing(&mut self);

    fn export_fcl_data(&self);

    fn import_fcl_data(&mut self);

    /// Runs neuron dynamics given FCL values from synapses, outputs if firing and updates the
    /// membrane potentially. Increments the burst index. Some backends may not update the
    /// visualizers with this step, so optionally this can be forced if needed.
    fn run_neuron_processing(&mut self, force_update_visualization: bool);
}
