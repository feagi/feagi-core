//! Containers of the `NeuronModel` family that cortical areas hold, for any cortical area type at
//! any member of its neuron model

use crate::basis::cortical_area_types::cortical_area_type::CorticalAreaTypeEnum;
use core::marker::PhantomData;
use feagi_connectome::cortical_area::{CorticalAreaGeneratorFromDefaults, NeuronModel};
use feagi_connectome::neuron_models::neuron_model::NeuronModelContainer;
use feagi_connectome::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;

//region Data

/// Holds the [`CorticalAreaGeneratorFromDefaults`] of a cortical area
pub struct FromDefaultsGenerator<FIQ>(PhantomData<fn() -> FIQ>);

impl<FIQ: FeagiIndexQuantization> NeuronModelContainer for FromDefaultsGenerator<FIQ> {
    type Of<M: NeuronModel> = CorticalAreaGeneratorFromDefaults<FIQ, M>;
}

/// The Genomic Data For a given Cortical Area
pub type CorticalGenomicDataEnum<FIQ> = CorticalAreaTypeEnum<FromDefaultsGenerator<FIQ>>;

//endregion
