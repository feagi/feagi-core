use crate::basis::cortical_area::cortical_area_type::CorticalAreaType;
use crate::cortical_area_types::cortical_area_type::CorticalAreaTypeKind;
use feagi_connectome::cortical_area::neuron_layout::implementations::voxels::NeuronLayoutVoxelDefaultType;
use feagi_connectome::cortical_area::CorticalAreaGeneratorFromDefaults;
use feagi_connectome::neuron_models::neuron_model::feagi_advanced::NeuronModelFeagiAdvanced;
use feagi_connectome::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;

/// Connects cortical areas to each other
pub struct CorticalAreaTypeInterconnect<NM: NeuronModelFeagiAdvanced>(core::marker::PhantomData<NM>);

impl<NM: NeuronModelFeagiAdvanced> CorticalAreaType<NM> for CorticalAreaTypeInterconnect<NM> {

    const CORTICAL_AREA_TYPE: CorticalAreaTypeKind = CorticalAreaTypeKind::Interconnect;

    type NeuronLayout = NeuronLayoutVoxelDefaultType;

    type CorticalIDGenerator = ();

    type CorticalAreaGenerator<FIQ: FeagiIndexQuantization> = CorticalAreaGeneratorFromDefaults<FIQ, NM>;
}
