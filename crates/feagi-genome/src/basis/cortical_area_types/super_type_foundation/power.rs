use feagi_basis::neurons::{CorticalAreaNeuronPotential, DimensionalCorticalAreaDimensions};
use feagi_basis::prelude::QuantizedElementBase;
use crate::basis::cortical_area_types::cortical_area_type::{CorticalAreaTypeKind, FoundationSuperTypeKind};
use crate::basis::specifications::cortical_area::cortical_area_type::{CorticalAreaType, FoundationCorticalAreaSubType};
use feagi_connectome::cortical_area::neuron_layout::implementations::voxels::{NeuronLayoutVoxelDefault, NeuronLayoutVoxelDefaultType};
use feagi_connectome::cortical_area::{CorticalAreaGeneratorFromDefaults, CorticalModelDefaultsSpawnData, NeuronModelUniversalCorticalProperties};
use feagi_connectome::cortical_area::neuron_layout::NeuronLayout;
use feagi_connectome::neuron_models::neuron_model::foundation_power::{NeuronModelFoundationPower, NeuronModelFoundationPowerFloat64};
use feagi_connectome::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;

/// Power management processing
pub struct CorticalAreaTypeFoundationPower<NM: NeuronModelFoundationPower>(core::marker::PhantomData<NM>);

impl<NM: NeuronModelFoundationPower> CorticalAreaType<NM> for CorticalAreaTypeFoundationPower<NM> {

    const CORTICAL_AREA_TYPE: CorticalAreaTypeKind = CorticalAreaTypeKind::Foundation(FoundationSuperTypeKind::Power);

    type NeuronLayout = NeuronLayoutVoxelDefaultType;

    type CorticalIDGenerator = ();

    type CorticalAreaGenerator<FIQ: FeagiIndexQuantization> = CorticalAreaGeneratorFromDefaults<FIQ, NM>;
}

impl<NM: NeuronModelFoundationPower> FoundationCorticalAreaSubType<NM> for CorticalAreaTypeFoundationPower<NM> {

}


// TODO make const

pub fn get_foundation_power_defaults<
    FIQ: FeagiIndexQuantization,
    NM: NeuronModelFoundationPower
>() -> CorticalModelDefaultsSpawnData<FIQ, NM> {
    CorticalModelDefaultsSpawnData {
        layout: NeuronLayoutVoxelDefault::new(
            DimensionalCorticalAreaDimensions::new_dimensions_from_usize(
                [1, 1, 1, 1]
            ).unwrap()),
        cortical_properties: NM::CorticalDataProperties::new(),
        cortical_internal: NM::CorticalDataInternal::new(),
        cortical_scratch: NM::CorticalDataScratch::new(),
        neuron_variance_properties: NM::CorticalFunctionContext::new(),
        neuron_internal: NM::NeuronDataInternal::new(),
        neuron_scratch: NM::NeuronDataScratch::new(),
        universal_data: NeuronModelUniversalCorticalProperties {
            cortical_level_psp: CorticalAreaNeuronPotential::QUANT_ONE,
            is_psp_uniform: false,
            is_psp_using_neuron_potential: false,
        },
    }
}