//! Base all-device traits that all Neuron Models extend off of. Since these are all device,
//! These implementations cannot actually hold any data. That must be added in the CPU extensions.

use feagi_structures::feagi_data::feagi_pdi::PDIElement;
use feagi_structures::feagi_data::feagi_pdi::tag_device::PDITagGenericDevice;
use feagi_structures::feagi_data::shared_quantization_sets::{NeuronModelQuantization, FeagiGlobalQuantization, FeagiGlobalQuantizationLevel, CorticalPotentialQuantizationLevel};
use crate::neural_processing_unit_data_structures::burst_engine_cluster::burst_engines::data_structures::tables::cortical_structure_configuration::cortical_configuration::CorticalConfigurationBase;

/// Root trait for all cortical data implementations, essentially any cortical level data shared
/// by all neurons in a cortical area of a given neuron model. This should be extended with only
/// the cortical level data
pub trait NeuronModelCorticalData<FGQ, NMQ, CCB>:
PDIElement
+ PDITagGenericDevice
where
    FGQ: FeagiGlobalQuantization,
    NMQ: NeuronModelQuantization,
    CCB: CorticalConfigurationBase<FGQ, NMQ>
{
    // Implement any cortical level data

    // No methods!
}

/// Root trait for all neuron data implementation, essentially per neuron data for a given
/// neuron model. This should be extended with only the per neuron data
pub trait NeuronModelNeuronData<FGQ, NMQ, CCB, NMCD>:
PDIElement
+ PDITagGenericDevice
where
    FGQ: FeagiGlobalQuantization,
    NMQ: NeuronModelQuantization,
    CCB: CorticalConfigurationBase<FGQ, NMQ>,
    NMCD: NeuronModelCorticalData<FGQ, NMQ, CCB>
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
pub trait NeuronModelProcessor<FGQ, NMQ, CCB, NMCD, NMND>:
PDIElement // Is this really an element? this technically has no state and isnt actually data lol
+ PDITagGenericDevice
where
    FGQ: FeagiGlobalQuantization,
    NMQ: NeuronModelQuantization,
    CCB: CorticalConfigurationBase<FGQ, NMQ>,
    NMCD: NeuronModelCorticalData<FGQ, NMQ, CCB>,
    NMND: NeuronModelNeuronData<FGQ, NMQ, CCB, NMCD>
{

    /// Set to true if the neuron model needs to be informed if the global burst index counter is
    /// about to overflow. Otherwise, set to false
    const MODEL_NEEDS_TO_BE_INFORMED_OF_BURST_INDEX_ROLLOVER: bool;

    // Methods for
    // blank neuron instantiation
    // neuron firing,
    // resetting cortical/neuron fields for burst index rollover

}