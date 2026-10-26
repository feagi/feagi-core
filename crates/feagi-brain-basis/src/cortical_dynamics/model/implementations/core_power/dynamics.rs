use half::{bf16, f16};
use feagi_basis::prelude::QuantizedDecimalUnwrappedTrait;
use crate::cortical_dynamics::model::cortical_dynamics_model::CorticalDynamicsModel;
use crate::{define_cortical_dynamics_quantization_template, expand_cortical_dynamics_quantization};

pub struct CorticalDynamicsModelCorePower<CDQ>(core::marker::PhantomData<CDQ>)
where CDQ: CorticalDynamicsQuantizationCorePower;

impl<CDQ> CorticalDynamicsModel<CDQ> for CorticalDynamicsModelCorePower<CDQ>
where
    CDQ: CorticalDynamicsQuantizationCorePower
{
    type CorticalDataProperties = ();
    type CorticalDataInternal = ();
    type CorticalDataScratch = ();
    type CorticalFunctionContext = ();
    type NeuronDataInternal = ();
    type NeuronDataScratch = ();
}


define_cortical_dynamics_quantization_template! {
    /// Quantization For the Core Power type
    CorePower: {
        T1: QuantizedDecimalUnwrappedTrait,
        T2: QuantizedDecimalUnwrappedTrait,
     }

     implementations: {
         Float16: {
             MembranePotentialQuant: f16,
            T1: f32,
            T2: f32,
         }

        BFloat16: {
             MembranePotentialQuant: bf16,
            T1: f32,
            T2: f32,
         }

         Float32: {
             MembranePotentialQuant: f32,
            T1: f32,
            T2: f32,
         }

         Float64: {
             MembranePotentialQuant: f64,
            T1: f32,
            T2: f32,
         }
     }

    CorticalDataProperties: {
        t_1: T1
    }
 }

cortical_dynamics_quantization_core_power_template!(expand_cortical_dynamics_quantization);



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