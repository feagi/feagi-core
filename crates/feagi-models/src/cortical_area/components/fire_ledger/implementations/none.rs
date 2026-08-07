use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
use crate::cortical_area::components::fire_ledger::FireLedger;

/// A FireLedger to be used when one needs to technically be defined
pub struct FireLedgerNone;

impl<FIQ: FeagiIndexQuantization> FireLedger<FIQ> for FireLedgerNone {
    
    const IS_FIRE_LEDGER: bool = false;
    
    fn push_is_firing(&mut self, is_firing: bool) {
        panic!("None FireLedger cannot be pushed!");
    }

    fn count_number_neurons_firing_in_window(&self) -> FIQ::NeuronIndexQuant {
        panic!("None FireLedger cannot be counted!");
    }

    fn get_hash(&self) -> () {
        panic!("None FireLedger cannot be hashed!");
    }
}