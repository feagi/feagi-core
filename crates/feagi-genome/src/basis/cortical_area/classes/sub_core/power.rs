use feagi_connectome::cortical_area::neuron_layout::implementations::voxels::NeuronLayoutVoxelDefaultType;
use feagi_connectome::neuron_models::core_power::data::NeuronModelCorePower;
use crate::basis::cortical_area::classes::cortical_area_class_enum::CorticalAreaClassEnum;
use crate::basis::cortical_area::cortical_area_class::CorticalAreaClass;

pub struct CorePowerCorticalClass<NM: NeuronModelCorePower>(core::marker::PhantomData<NM>);

impl<NM: NeuronModelCorePower> CorticalAreaClass<NM> for CorePowerCorticalClass<NM> {
    
    const CORTICAL_AREA_CLASS: CorticalAreaClassEnum = CorticalAreaClassEnum::Interconnect;
    
    type NeuronLayout = NeuronLayoutVoxelDefaultType;

    type CorticalIDGenerator = ();
}