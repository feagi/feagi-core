use crate::basis::cortical_area::cortical_area_type::{CorticalAreaType, FoundationCorticalAreaSubType};
use crate::cortical_area_types::cortical_area_type::{CorticalAreaTypeKind, FoundationSuperTypeKind};
use feagi_basis::neurons::{CorticalAreaNeuronPotential, DimensionalCorticalAreaDimensions};
use feagi_basis::prelude::QuantizedElementBase;
use feagi_connectome::cortical_area::neuron_layout::voxel::{NeuronLayoutTypeVoxel, NeuronLayoutVoxel};
use feagi_connectome::cortical_area::neuron_layout::NeuronLayout;
use feagi_connectome::cortical_area::{CorticalAreaSpec, NeuronModelUniversalCorticalProperties};
use feagi_connectome::neuron_models::neuron_model::foundation_power::NeuronModelFoundationPower;
use feagi_connectome::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;

/// Power management processing
pub struct CorticalAreaTypeFoundationPower<NM: NeuronModelFoundationPower>(core::marker::PhantomData<NM>);

impl<NM: NeuronModelFoundationPower> CorticalAreaType<NM> for CorticalAreaTypeFoundationPower<NM> {

    const CORTICAL_AREA_TYPE: CorticalAreaTypeKind = CorticalAreaTypeKind::Foundation(FoundationSuperTypeKind::Power);

    type NeuronLayout = NeuronLayoutTypeVoxel;

    type CorticalIDGenerator = ();
}

impl<NM: NeuronModelFoundationPower> FoundationCorticalAreaSubType<NM> for CorticalAreaTypeFoundationPower<NM> {

}


// TODO make const

pub fn get_foundation_power_defaults<
    FIQ: FeagiIndexQuantization,
    NM: NeuronModelFoundationPower
>() -> CorticalAreaSpec<FIQ, NM> {
    CorticalAreaSpec::new_uniform(
        NeuronLayoutVoxel::new(
            DimensionalCorticalAreaDimensions::new_dimensions_from_usize(
                [1, 1, 1, 1]
            ).unwrap()),
        NeuronModelUniversalCorticalProperties {
            cortical_level_psp: CorticalAreaNeuronPotential::QUANT_ONE,
            is_psp_uniform: false,
            is_psp_using_neuron_potential: false,
        },
        NM::CorticalDataProperties::new(),
        NM::CorticalDataInternal::new(),
        NM::CorticalDataScratch::new(),
        NM::CorticalDataVariance::new(),
        NM::NeuronDataInternal::new(),
        NM::NeuronDataScratch::new(),
    )
}