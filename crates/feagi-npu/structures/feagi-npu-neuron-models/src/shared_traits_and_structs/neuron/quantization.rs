use feagi_structures::feagi_data::quantizable_linear::base_types::QuantizedDecimalTrait;
use feagi_structures::feagi_data::shared_quantization_sets::{CorticalAreaModelQuantizationBase, CorticalAreasIndexQuantization};

// TODO proc macro to generate what we need


// the options for quantizable types are:
// QuantizedIndexCountTrait (for indexing, counting)
// QuantizedUnsignedIntegerTrait (for unsigned integer values)
// QuantizedSignedIntegerTrait (for signed integer values)
// QuantizedDecimalTrait (for decimal (floating point) values)





/// Quantization level definitions for a given neuron model. This is the base implementation,
/// specific neuron models will have their own extension of this
pub trait NeuronModelTraitBase<CAIQ: CorticalAreasIndexQuantization>: CorticalAreaModelQuantizationBase {
    // As per CorticalAreasIndexQuantization, this takes in GlobalBurstIndexQuant,
    // and NeuronIndexCountQuant. These are not settable by the model and instead picked by
    // FEAGI's NPU. This trait extends CorticalAreaModelQuantizationBase, which means there is
    // access to NeuronPotentialQuant, which can be configured by various cortical model
    // extension settings

    // NOTE: Implementations of Neuron Models do not store their own membrane potential! They
    // will be passed in by reference if need be!

    const NEURON_NEEDS_TO_BE_INFORMED_OF_BURST_INDEX_ROLLOVER: bool;


    /// Neuron received input potential. Process it, updating any internal states and update
    /// this neurons potential. Return true if it results in this neuron firing, otherwise
    /// return false. Note that incoming neuron potential may be an arbitrary quantization.
    fn process_neuron_potential<IPQuant: QuantizedDecimalTrait>(
        &mut self,
        incoming_neuron_potential: &IPQuant,
        self_neuron_potential: &mut Self::NeuronPotentialQuant
    ) -> bool;


    /// If enabled via the const of this trait, this method will be called on all neurons of that
    /// neuron model type right before the global burst index overflows and resets to 0. Use this
    /// method to update any values that need to be updated in that case
    fn prepare_neuron_for_burst_index_rollover(&mut self) {
        // by default nothing. Override me if you have something you need to do, but remember
        // to have NEURON_NEEDS_TO_BE_INFORMED_OF_BURST_INDEX_ROLLOVER set to true
    }

}





/*
// TODO dont think we need this?

/// This is what is extended in this crate to represent a neuron models quantization
pub(crate) trait NeuronModelQuantizationTemplateBase {
    type NeuronPotentialQuant: QuantizedDecimalTrait;

    // Extend this trait with any quantization parameters a model required
}

 */