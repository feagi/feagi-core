//! Containers of the `NeuronModel` family that cortical areas hold, for any cortical area type at
//! any member of its neuron model

use crate::cortical_area_types::cortical_area_type::CorticalAreaTypeEnum;
use feagi_connectome::neuron_models::spec_dispatch::CorticalAreaSpecContainer;

//region Data

/// The Genomic Data For a given Cortical Area
pub type CorticalGenomicDataEnum<FIQ> = CorticalAreaTypeEnum<CorticalAreaSpecContainer<FIQ>>;

//endregion
