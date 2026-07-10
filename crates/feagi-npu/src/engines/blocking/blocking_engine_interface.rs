use crate::engines::blocking::blocking_engine::BlockingEngine;
use crate::engines::blocking::rayon::RayonBurstEngine;
use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;

/// Shorthand way to call through an enum a method implemented by all key members
macro_rules! dispatch {
    ($self:expr, $func:ident $(, $arg:expr)*) => {

        match $self {
            Self::RayonInterface(x) => x.$func($($arg),*),
        }
    };
}

pub enum BlockingEngineInterface<Q: FeagiIndexQuantization> {
    // TODO feature gate Rayon
    RayonInterface(RayonBurstEngine<Q>), // TODO other interfaces
}

impl<Q: FeagiIndexQuantization> BlockingEngineInterface<Q> {
    /// runs some number of bursts without foreign involvement
    pub fn run_complete_bursts(&mut self, number_bursts: usize, force_update_visualization: bool) {
        for _ in 0..(number_bursts - 1) {
            self.run_synapse_processing_phase();
            self.run_neuron_processing(false);
        }
        self.run_synapse_processing_phase();
        self.run_neuron_processing(force_update_visualization);
    }

    /// Import sensor data, and export motor, sensor, and neuron mp data (all optionally).
    pub fn exchange_agent_and_mp_data(&mut self) {
        todo!()
    }

    /// Potentially consolidate firing neurons, then execute synapse dynamics and merge results
    /// to the FCL
    pub fn run_synapse_processing(&mut self) {
        dispatch!(self, run_synapse_processing)
    }

    pub fn export_fcl_data(&self) {
        todo!()
    }

    pub fn import_fcl_data(&mut self) {
        todo!()
    }

    /// Runs neuron dynamics given FCL values from synapses, outputs if firing and updates the
    /// membrane potentially. Increments the burst index. Some backends may not update the
    /// visualizers with this step, so optionally this can be forced if needed.
    pub fn run_neuron_processing(&mut self, force_update_visualization: bool) {
        dispatch!(self, run_neuron_processing, force_update_visualization);
    }
}
