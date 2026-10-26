use crate::cortical_dynamics::model::cortical_dynamics_model::CorticalDynamicsModel;
use crate::quantization_levels::membrane_potential_quantization::MembranePotentialQuantization;

/// Can be used to rapidly generate quantization types
/*
 define_cortical_dynamics_quantization! {
     ///Quant Comment
     CorePower: {
         /// Comment
         T1: QuantizedDecimalUnwrappedTrait,
     }

     implementations: {
         Float16: {
             MembranePotentialQuant: f16,
             T1: f32
         }

         Float32: {
             MembranePotentialQuant: f32,
             T1: f32
         }
     }
 }
*/
#[macro_export]
macro_rules! define_cortical_dynamics_quantization {
    (
        $(#[$m_attr:meta])*
        $model:ident : {
            $( $(#[$attr:meta])* $assoc:ident : $bound:path ),* $(,)?
        } $(,)?

        implementations : {
            $(
                $variant:ident : {
                    MembranePotentialQuant : $mp_ty:ty
                    $(, $v_assoc:ident : $v_ty:ty )* $(,)?
                } $(,)?
            )*
        } $(,)?

        CorticalDataProperties : {
            $( $(#[$f_attr:meta])* $field:ident : $f_assoc:ident ),* $(,)?
        } $(,)?
    ) => {
        paste::paste! {
            $(#[$m_attr])*
            pub trait [<CorticalDynamicsQuantization $model>]: CorticalDynamicsQuantization {
                $(
                    $(#[$attr])*
                    type $assoc: $bound;
                )*
            }

            $(
                #[derive(Clone, Copy, PartialEq, Debug, Serialize, Deserialize)]
                pub struct [<CorticalDynamicsQuantization $model $variant>];

                unsafe impl Sync for [<CorticalDynamicsQuantization $model $variant>] {

                }

                unsafe impl Send for [<CorticalDynamicsQuantization $model $variant>] {

                }

                impl MembranePotentialQuantization for [<CorticalDynamicsQuantization $model $variant>] {
                    type MembranePotentialQuant = $mp_ty;
                }

                impl CorticalDynamicsQuantization for [<CorticalDynamicsQuantization $model $variant>] {
                    type Model = [<CorticalDynamicsModel $model>]<Self>;
                }

                impl [<CorticalDynamicsQuantization $model>] for [<CorticalDynamicsQuantization $model $variant>] {
                    $( type $v_assoc = $v_ty; )*
                }
            )*

            pub struct [<CorticalDataProperties $model>]<CDQ: [<CorticalDynamicsQuantization $model>]> {
                $(
                    $(#[$f_attr])*
                    pub $field: CDQ::$f_assoc,
                )*
            }
        }
    };
}


/// Describes the quantization level of a specific CorticalDynamicsModel data points
pub trait CorticalDynamicsQuantization: MembranePotentialQuantization {
    /// What model uses this quantization
    type Model: CorticalDynamicsModel<Self>;
    
    // This trait is to be expanded on per model
}


