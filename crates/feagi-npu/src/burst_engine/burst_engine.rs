use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;


pub trait BurstEngine<FIQ: FeagiIndexQuantization> {

    // NOTE: All functions take a mut self such that the borrow checker will refuse to let them
    // run if the async run_burst is running

    /// Run a number of bursts before stopping
    async fn run_burst(&mut self);

    // TODO a function for half a burst for exchanging synapse data?

    /// Load into the engine sensor data and firing neuron data (from other engines) and export motor data and visualization data
    /// Exchanges data with the burst engine right after neurons have received input from synapses
    ///
    /// Burst Engine Imports:
    /// - Sensor Data
    ///
    /// Burst Engine Exports:
    /// - Voxel Activation Data
    /// - Motor Data
    /// - Probe Data
    /// - Outgoing Synapse Data for bridged cortical areas // TODO
    fn exchange_data_for_sensor_motor_probes_and_potentials(&mut self);

    /// Import bridged Synapse data from other burst engines
    fn import_bridged_synapse_propagation_data(&mut self);

    /// Export bridged neuron mp data for other burst engines
    fn export_bridged_neuron_mp_data(&mut self);

    /// Import bridged neuron mp data from other burst engines
    fn import_bridged_neuron_mp_data(&mut self);
    
    /// change properties of cortical areas in place (no reallocations needed)
    fn edit_in_place_cortical_areas(&mut self);
}


