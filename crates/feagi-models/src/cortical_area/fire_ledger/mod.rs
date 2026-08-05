pub mod implementations;

use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;

pub trait FireLedger<FIQ: FeagiIndexQuantization> {

    /// Defines if the fire ledger exists (is not None)
    const IS_FIRE_LEDGER: bool;

    fn push_is_firing(&mut self, is_firing: bool);

    fn count_number_neurons_firing_in_window(&self) -> FIQ::NeuronIndexQuant;

    fn get_hash(&self) -> (); // TODO
}