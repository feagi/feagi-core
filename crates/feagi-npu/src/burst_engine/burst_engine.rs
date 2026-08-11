use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;


pub trait BurstEngine<FIQ: FeagiIndexQuantization> {

    // NOTE: All functions take a mut self such that the borrow checker will refuse to let them
    // run if the async run_burst is running

    /// Run a number of bursts before stopping
    async fn run_bursts(&mut self);

    // TODO a function for half a burst for exchanging synapse data?

    /// Load into the engine sensor data and firing neuron data (from other engines) and export motor data and visualization data
    fn exchange_data_for_sensor_motor_probes_and_potentials(&mut self);

    fn exchange_synapse_propagation_data(&mut self);
    
    /// change properties of cortical areas in place (no reallocations needed)
    fn edit_in_place_cortical_areas(&mut self);
}


