use crate::basis::cortical_area::cortical_area_type::CorticalAreaType;
use crate::cortical_area_types::enums::CorticalAreaTypeEnum;
use feagi_connectome::cortical_area::neuron_layout::implementations::voxels::NeuronLayoutVoxelDefaultType;
use feagi_connectome::neuron_models::feagi_advanced::data::{FeagiAdvancedGenomePropertiesDataEnum, NeuronModelFeagiAdvanced};
use feagi_connectome::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;

pub struct InterconnectCorticalClass<NM: NeuronModelFeagiAdvanced>(core::marker::PhantomData<NM>);

impl<NM: NeuronModelFeagiAdvanced> CorticalAreaType<NM> for InterconnectCorticalClass<NM> {

    const CORTICAL_AREA_TYPE: CorticalAreaTypeEnum = CorticalAreaTypeEnum::Interconnect;

    type NeuronLayout = NeuronLayoutVoxelDefaultType;

    type CorticalIDGenerator = ();
    type CorticalAreaGenerator = ();
}

/// The Genomic Data for a given Neuron Model of a InterConnect Cortical Area
pub enum InterConnectCorticalAreaEModelEnum<FIQ: FeagiIndexQuantization> {
    FeagiAdvanced(FeagiAdvancedGenomePropertiesDataEnum<FIQ>)
}