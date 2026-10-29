use half::{bf16, f16};
use feagi_connectome_neural::neural_basis::cortical_area::cortical_dynamics_model::CorticalDynamicsModel;
use feagi_connectome_neural::neural_basis::cortical_area::cortical_dynamics_quantization::{CorticalDataInternalSpecification, CorticalDataPropertiesSpecification, CorticalDataScratchSpecification, CorticalDynamicsQuantization, CorticalFunctionSpecification, NeuronDataInternalSpecification, NeuronDataScratchSpecification};
use feagi_connectome_neural::neural_basis::cortical_area::neuron_layout::implementations::voxels_default::NeuronLayoutVoxelDefault;
use feagi_connectome_neural::neural_basis::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
use feagi_connectome_neural::neural_basis::quantization_levels::membrane_potential_quantization::MembranePotentialQuantization;
use feagi_connectome_proc::{define_cortical_dynamics_quantization_template, expand_cortical_dynamics_quantization};

pub struct CorticalDynamicsModelCorePower<FIQ, CDQ>(core::marker::PhantomData<(FIQ, CDQ)>)
where
    FIQ: FeagiIndexQuantization,
    CDQ: CorticalDynamicsQuantizationCorePower;

impl<FIQ, CDQ> CorticalDynamicsModel<FIQ, NeuronLayoutVoxelDefault<FIQ>, CDQ> for CorticalDynamicsModelCorePower<FIQ, CDQ>
where
    FIQ: FeagiIndexQuantization,
    CDQ: CorticalDynamicsQuantizationCorePower,
{

}


define_cortical_dynamics_quantization_template! {
    /// Quantization For the Core Power type
    CorePower: {

     }

     layout: NeuronLayoutVoxelDefault<FIQ>

     implementations: {
         Float16: {
             MembranePotentialQuant: f16,
         }

        BFloat16: {
             MembranePotentialQuant: bf16,
         }

         Float32: {
             MembranePotentialQuant: f32,
         }

         Float64: {
             MembranePotentialQuant: f64,
         }
     }
 }


cortical_dynamics_quantization_core_power_template!(expand_cortical_dynamics_quantization);

pub struct CorticalDataPropertiesCorePower<CDQ: CorticalDynamicsQuantizationCorePower> {
    _p: core::marker::PhantomData<CDQ>,
}

impl<CDQ: CorticalDynamicsQuantizationCorePower> CorticalDataPropertiesSpecification<CDQ> for CorticalDataPropertiesCorePower<CDQ> {}

pub struct CorticalDataInternalCorePower<CDQ: CorticalDynamicsQuantizationCorePower> {
    _p: core::marker::PhantomData<CDQ>,
}

impl<CDQ: CorticalDynamicsQuantizationCorePower> CorticalDataInternalSpecification<CDQ> for CorticalDataInternalCorePower<CDQ> {}


pub struct CorticalDataScratchCorePower<CDQ: CorticalDynamicsQuantizationCorePower> {
    _p: core::marker::PhantomData<CDQ>,
}

impl<CDQ: CorticalDynamicsQuantizationCorePower> CorticalDataScratchSpecification<CDQ> for CorticalDataScratchCorePower<CDQ> {}

pub struct CorticalFunctionContextCorePower<CDQ: CorticalDynamicsQuantizationCorePower> {
    _p: core::marker::PhantomData<CDQ>,
}

impl<CDQ: CorticalDynamicsQuantizationCorePower> CorticalFunctionSpecification<CDQ> for CorticalFunctionContextCorePower<CDQ> {}


pub struct NeuronDataInternalCorePower<CDQ: CorticalDynamicsQuantizationCorePower> {
    _p: core::marker::PhantomData<CDQ>,
}

impl<CDQ: CorticalDynamicsQuantizationCorePower> NeuronDataInternalSpecification<CDQ> for NeuronDataInternalCorePower<CDQ> {}


pub struct NeuronDataScratchCorePower<CDQ: CorticalDynamicsQuantizationCorePower> {
    _p: core::marker::PhantomData<CDQ>,
}

impl<CDQ: CorticalDynamicsQuantizationCorePower> NeuronDataScratchSpecification<CDQ> for NeuronDataScratchCorePower<CDQ> {}

