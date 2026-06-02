//! Base all-device traits that all Neuron Models extend off of. Since these are all device,
//! These implementations cannot actually hold any data. That must be added in the CPU extensions.

use feagi_structures::feagi_data::feagi_pdi::PDIElement;
use feagi_structures::feagi_data::quantizable_linear::base_types::QuantizedDecimalTrait;
use feagi_structures::feagi_data::shared_quantization_sets::{NeuronModelQuantization, FeagiGlobalQuantization};
use crate::shared_traits_and_structs::cortical_configuration::CorticalConfiguration;

/// Root trait for all cortical data implementations, essentially any cortical level data shared
/// by all neurons in a cortical area of a given neuron model. This should be extended with only
/// the cortical level data
pub trait CorticalModelData<FGQ, NMQ>:
PDIElement
where
    FGQ: FeagiGlobalQuantization,
    NMQ: NeuronModelQuantization,
{
    // Implement any per-neuron level data members
    
    // must implement a bool for mp_driven_psp

    // No methods!
}

/// Root trait for all neuron data implementation, essentially per neuron data for a given
/// neuron model. This should be extended with only the per neuron data
pub trait NeuronModelData<FGQ, NMQ>:
PDIElement
where
    FGQ: FeagiGlobalQuantization,
    NMQ: NeuronModelQuantization,
{
    // As per CorticalAreasIndexQuantization, this takes in GlobalBurstIndexQuant,
    // and NeuronIndexCountQuant. These are not settable by the model and instead picked by
    // FEAGI's NPU

    // NOTE: Implementations of Neuron Models do not store their own membrane potential! They
    // will be passed in by reference if need be!

    // Implement any per-neuron level data members
    
    // No methods!
}

/// Root base trait for defining neuron firing and other dynamics. Does NOT store actual data,
pub trait NeuronModelProcessor<FGQ, NMQ, CCC, CMD, NMD>:
PDIElement
where
    FGQ: FeagiGlobalQuantization,
    NMQ: NeuronModelQuantization,
    CCC: CorticalConfiguration<FGQ>,  // Extend to Dimensional or other
    CMD: CorticalModelData<FGQ, NMQ>,
    NMD: NeuronModelData<FGQ, NMQ>
{

    /// Set to true if the neuron model needs to be informed if the global burst index counter is
    /// about to overflow. Otherwise, set to false
    const MODEL_NEEDS_TO_BE_INFORMED_OF_BURST_INDEX_ROLLOVER: bool;

    // Methods for neuron firing, resetting cortical and neuron fields for burst index rollover

}