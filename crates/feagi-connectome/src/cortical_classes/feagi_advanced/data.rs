use feagi_basis::create_wrapped_quantized_decimal;
use feagi_connectome_proc::{define_cortical_dynamics_quantization_template, expand_cortical_dynamics_quantization};
use crate::cortical_area::CorticalDynamicsModel;
use feagi_connectome_neural::neural_basis::cortical_area::cortical_dynamics_quantization::{CorticalDataInternalSpecification, CorticalDataPropertiesSpecification, CorticalDataScratchSpecification, CorticalDynamicsQuantization, CorticalFunctionSpecification, NeuronDataInternalSpecification, NeuronDataScratchSpecification};
use feagi_connectome_neural::neural_basis::cortical_area::neuron_layout::NeuronLayout;
use crate::cortical_area::neuron_layout::implementations::voxels_default::NeuronLayoutVoxelDefault;
use crate::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;

//region Wrappers

create_wrapped_quantized_decimal!(
    /// How long till the neuron can fire again
    pub RefractoryPeriod
);


//endregion

pub struct CorticalDynamicsModelFeagiAdvanced<FIQ, NL, CDQ>(core::marker::PhantomData<(FIQ, NL, CDQ)>)
where
    FIQ: FeagiIndexQuantization,
    NL: NeuronLayout<FIQ>,
    CDQ: CorticalDynamicsQuantizationFeagiAdvanced;

impl<FIQ, NL, CDQ> CorticalDynamicsModel<FIQ, NL, CDQ> for CorticalDynamicsModelFeagiAdvanced<FIQ, NL, CDQ>
where
    FIQ: FeagiIndexQuantization,
    NL: NeuronLayout<FIQ>,
    CDQ: CorticalDynamicsQuantizationFeagiAdvanced,
{

}

define_cortical_dynamics_quantization_template! {
    /// Quantization For the Core Power type
    FeagiAdvanced: {
        RefractoryPeriod: QuantizedDecimalUnwrappedTrait,
     }

     implementations: {
         DefaultQuant: {
            MembranePotentialQuant: f32,
            RefractoryPeriod: f32
         }
     }
 }

cortical_dynamics_quantization_feagi_advanced_template!(expand_cortical_dynamics_quantization);

pub struct CorticalDataPropertiesFeagiAdvanced<CDQ: CorticalDynamicsQuantizationFeagiAdvanced> {
    _p: core::marker::PhantomData<CDQ>,
}

impl<CDQ: CorticalDynamicsQuantizationFeagiAdvanced> CorticalDataPropertiesSpecification<CDQ> for CorticalDataPropertiesFeagiAdvanced<CDQ> {}

pub struct CorticalDataInternalFeagiAdvanced<CDQ: CorticalDynamicsQuantizationFeagiAdvanced> {
    _p: core::marker::PhantomData<CDQ>,
}

impl<CDQ: CorticalDynamicsQuantizationFeagiAdvanced> CorticalDataInternalSpecification<CDQ> for CorticalDataInternalFeagiAdvanced<CDQ> {}


pub struct CorticalDataScratchFeagiAdvanced<CDQ: CorticalDynamicsQuantizationFeagiAdvanced> {
    _p: core::marker::PhantomData<CDQ>,
}

impl<CDQ: CorticalDynamicsQuantizationFeagiAdvanced> CorticalDataScratchSpecification<CDQ> for CorticalDataScratchFeagiAdvanced<CDQ> {}

pub struct CorticalFunctionContextFeagiAdvanced<CDQ: CorticalDynamicsQuantizationFeagiAdvanced> {
    _p: core::marker::PhantomData<CDQ>,
}

impl<CDQ: CorticalDynamicsQuantizationFeagiAdvanced> CorticalFunctionSpecification<CDQ> for CorticalFunctionContextFeagiAdvanced<CDQ> {}


pub struct NeuronDataInternalFeagiAdvanced<CDQ: CorticalDynamicsQuantizationFeagiAdvanced> {
    _p: core::marker::PhantomData<CDQ>,
}

impl<CDQ: CorticalDynamicsQuantizationFeagiAdvanced> NeuronDataInternalSpecification<CDQ> for NeuronDataInternalFeagiAdvanced<CDQ> {}


pub struct NeuronDataScratchFeagiAdvanced<CDQ: CorticalDynamicsQuantizationFeagiAdvanced> {
    _p: core::marker::PhantomData<CDQ>,
}

impl<CDQ: CorticalDynamicsQuantizationFeagiAdvanced> NeuronDataScratchSpecification<CDQ> for NeuronDataScratchFeagiAdvanced<CDQ> {}
