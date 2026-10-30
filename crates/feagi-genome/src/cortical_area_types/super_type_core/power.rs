use crate::basis::cortical_area::cortical_area_type::{CoreCorticalAreaSubType, CorticalAreaType};
use crate::basis::invalid_cortical_enum_error::CorticalMatchSelectionError;
use crate::cortical_area_types::enums::CorticalAreaTypeEnum;
use crate::cortical_area_types::super_type_core::enums::CoreCorticalAreaSuperTypeEnum;
use feagi_connectome::cortical_area::neuron_layout::implementations::voxels::NeuronLayoutVoxelDefaultType;
use feagi_connectome::neuron_models::core_power::data::{CorePowerGenomePropertiesDataEnum, NeuronModelCorePower};
use feagi_connectome::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;

pub struct CorePowerCorticalClass<NM: NeuronModelCorePower>(core::marker::PhantomData<NM>);

impl<NM: NeuronModelCorePower> CorticalAreaType<NM> for CorePowerCorticalClass<NM> {
    
    const CORTICAL_AREA_TYPE: CorticalAreaTypeEnum = CorticalAreaTypeEnum::Core(CoreCorticalAreaSuperTypeEnum::Power);
    
    type NeuronLayout = NeuronLayoutVoxelDefaultType;

    type CorticalIDGenerator = ();
    type CorticalAreaGenerator = ();
}

impl<NM: NeuronModelCorePower> CoreCorticalAreaSubType<NM> for CorePowerCorticalClass<NM> {
    
}

/// The Genomic Data for a given Neuron Model of a Power Cortical Area
pub enum PowerTypeCorticalGenomicDataEnum<FIQ: FeagiIndexQuantization> {
    Power(CorePowerGenomePropertiesDataEnum<FIQ>)
}

impl<FIQ: FeagiIndexQuantization> From<CorePowerGenomePropertiesDataEnum<FIQ>> for PowerTypeCorticalGenomicDataEnum<FIQ>
{
    fn from(value: CorePowerGenomePropertiesDataEnum<FIQ>) -> Self {
        PowerTypeCorticalGenomicDataEnum::Power(value)
    }
}

impl<FIQ: FeagiIndexQuantization> TryFrom<PowerTypeCorticalGenomicDataEnum<FIQ>> for CorePowerGenomePropertiesDataEnum<FIQ>
{
    type Error = CorticalMatchSelectionError;

    fn try_from(value: PowerTypeCorticalGenomicDataEnum<FIQ>) -> Result<Self, Self::Error> {
        #[allow(unreachable_patterns)] // Only one variant exists for now
        match value {
            PowerTypeCorticalGenomicDataEnum::Power(t) => {
                Ok(t)
            }
            _ => {Err(CorticalMatchSelectionError::IncorrectCorticalModel {tried: "", actual: ""})}
        }
    }
}