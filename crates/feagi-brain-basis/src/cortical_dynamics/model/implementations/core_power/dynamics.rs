use crate::quantization_levels::membrane_potential_quantization::MembranePotentialQuantization;
use crate::cortical_dynamics::model::cortical_dynamics_quantization::CorticalDynamicsQuantization;
use half::{bf16, f16};
use crate::cortical_dynamics::model::cortical_dynamics_model::{CorticalAreaModelCorticalSpecification, CorticalAreaModelNeuronSpecification, CorticalDynamicsModel};
use crate::{define_cortical_dynamics_quantization_template, expand_cortical_dynamics_quantization};

pub struct CorticalDynamicsModelCorePower<CDQ>(core::marker::PhantomData<CDQ>)
where CDQ: CorticalDynamicsQuantizationCorePower;

impl<CDQ> CorticalDynamicsModel<CDQ> for CorticalDynamicsModelCorePower<CDQ>
where
    CDQ: CorticalDynamicsQuantizationCorePower
{
    type CorticalDataProperties = CorticalDataPropertiesCorePower<CDQ>;
    type CorticalDataInternal = CorticalDataInternalCorePower<CDQ>;
    type CorticalDataScratch = CorticalDataScratchCorePower<CDQ>;
    type CorticalFunctionContext = CorticalFunctionContextCorePower<CDQ>;
    type NeuronDataInternal = NeuronDataInternalCorePower<CDQ>;
    type NeuronDataScratch = NeuronDataScratchCorePower<CDQ>;
}


define_cortical_dynamics_quantization_template! {
    /// Quantization For the Core Power type
    CorePower: {

     }

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

impl<CDQ: CorticalDynamicsQuantizationCorePower> CorticalAreaModelCorticalSpecification<CDQ> for CorticalDataPropertiesCorePower<CDQ> {}

pub struct CorticalDataInternalCorePower<CDQ: CorticalDynamicsQuantizationCorePower> {
    _p: core::marker::PhantomData<CDQ>,
}

impl<CDQ: CorticalDynamicsQuantizationCorePower> CorticalAreaModelCorticalSpecification<CDQ> for CorticalDataInternalCorePower<CDQ> {}


pub struct CorticalDataScratchCorePower<CDQ: CorticalDynamicsQuantizationCorePower> {
    _p: core::marker::PhantomData<CDQ>,
}

impl<CDQ: CorticalDynamicsQuantizationCorePower> CorticalAreaModelCorticalSpecification<CDQ> for CorticalDataScratchCorePower<CDQ> {}

pub struct CorticalFunctionContextCorePower<CDQ: CorticalDynamicsQuantizationCorePower> {
    _p: core::marker::PhantomData<CDQ>,
}

impl<CDQ: CorticalDynamicsQuantizationCorePower> CorticalAreaModelCorticalSpecification<CDQ> for CorticalFunctionContextCorePower<CDQ> {}





pub struct NeuronDataInternalCorePower<CDQ: CorticalDynamicsQuantizationCorePower> {
    _p: core::marker::PhantomData<CDQ>,
}

impl<CDQ: CorticalDynamicsQuantizationCorePower> CorticalAreaModelNeuronSpecification<CDQ> for NeuronDataInternalCorePower<CDQ> {
    
}


pub struct NeuronDataScratchCorePower<CDQ: CorticalDynamicsQuantizationCorePower> {
    _p: core::marker::PhantomData<CDQ>,
}

impl<CDQ: CorticalDynamicsQuantizationCorePower> CorticalAreaModelNeuronSpecification<CDQ> for NeuronDataScratchCorePower<CDQ> {

}



/*

pub trait CorticalDynamicsQuantizationCorePower: CorticalDynamicsQuantization {

}

#[derive(Clone, Copy, PartialEq, Debug, Serialize, Deserialize)]
pub struct CorticalDynamicsQuantizationCorePowerFloat16;

unsafe impl Sync for CorticalDynamicsQuantizationCorePowerFloat16 {

}

unsafe impl Send for CorticalDynamicsQuantizationCorePowerFloat16 {

}

impl MembranePotentialQuantization for CorticalDynamicsQuantizationCorePowerFloat16 {
    type MembranePotentialQuant = f16;
}

impl CorticalDynamicsQuantization for CorticalDynamicsQuantizationCorePowerFloat16 {
    type Model = CorticalDynamicsModelCorePower<Self>;
}

impl CorticalDynamicsQuantizationCorePower for CorticalDynamicsQuantizationCorePowerFloat16 {
    type T1 = f32;
}




#[derive(Clone, Copy, PartialEq, Debug, Serialize, Deserialize)]
pub struct CorticalDynamicsQuantizationCorePowerBFloat16;

unsafe impl Sync for CorticalDynamicsQuantizationCorePowerBFloat16 {

}

unsafe impl Send for CorticalDynamicsQuantizationCorePowerBFloat16 {

}

impl MembranePotentialQuantization for CorticalDynamicsQuantizationCorePowerBFloat16 {
    type MembranePotentialQuant = bf16;
}

impl CorticalDynamicsQuantization for CorticalDynamicsQuantizationCorePowerBFloat16 {
    type Model = CorticalDynamicsModelCorePower<Self>;
}

impl CorticalDynamicsQuantizationCorePower for CorticalDynamicsQuantizationCorePowerBFloat16 {
    type T1 = f32;
}


 */