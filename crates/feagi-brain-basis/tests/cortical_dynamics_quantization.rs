use feagi_basis::prelude::QuantizedDecimalUnwrappedTrait;
use feagi_brain_basis::cortical_dynamics::model::cortical_dynamics_model::CorticalDynamicsModel;
use feagi_brain_basis::cortical_dynamics::model::cortical_dynamics_quantization::CorticalDynamicsQuantization;
use feagi_brain_basis::cortical_dynamics::model::implementations::core_power::dynamics::{
    CorticalDataPropertiesCorePower, CorticalDynamicsModelCorePower, CorticalDynamicsQuantizationCorePower,
    CorticalDynamicsQuantizationCorePowerBFloat16, CorticalDynamicsQuantizationCorePowerFloat16,
};
use feagi_brain_basis::quantization_levels::membrane_potential_quantization::MembranePotentialQuantization;
use feagi_brain_basis::{define_cortical_dynamics_quantization_template, expand_cortical_dynamics_quantization};
use half::{bf16, f16};

//region Test Model

/// A model defined outside of feagi_brain_basis, giving every data type
pub struct CorticalDynamicsModelEveryData<CDQ>(core::marker::PhantomData<CDQ>)
where
    CDQ: CorticalDynamicsQuantizationEveryData;

impl<CDQ> CorticalDynamicsModel<CDQ> for CorticalDynamicsModelEveryData<CDQ>
where
    CDQ: CorticalDynamicsQuantizationEveryData,
{
    type CorticalDataProperties = ();
    type CorticalDataInternal = ();
    type CorticalDataScratch = ();
    type CorticalFunctionContext = ();
    type NeuronDataInternal = ();
    type NeuronDataScratch = ();
}

define_cortical_dynamics_quantization_template! {
    EveryData: {
        T1: QuantizedDecimalUnwrappedTrait,
        T2: QuantizedDecimalUnwrappedTrait,
    }

    implementations: {
        Float32: {
            T2: f64,
            MembranePotentialQuant: f32,
            T1: f32,
        }
    }

    // Out of trait order on purpose
    NeuronDataScratch: {
        scratch: T2,
    }

    CorticalDataProperties: {
        /// A described field
        t_1: T1,
    }

    CorticalDataInternal: {
        t_1: T1,
        t_2: T2,
    }

    CorticalDataScratch: {}

    CorticalFunctionContext: {
        context: T2,
    }

    NeuronDataInternal: {
        neuron: T1,
    }
}

cortical_dynamics_quantization_every_data_template!(expand_cortical_dynamics_quantization);

//endregion

/// Compiles only if `Actual` and `Expected` are the same type
fn assert_same_type<Actual, Expected>()
where
    Actual: SameAs<Expected>,
{
}

trait SameAs<T> {}

impl<T> SameAs<T> for T {}

#[test]
fn implementations_set_their_types() {
    type F16 = CorticalDynamicsQuantizationCorePowerFloat16;
    assert_same_type::<<F16 as MembranePotentialQuantization>::MembranePotentialQuant, f16>();
    assert_same_type::<<F16 as CorticalDynamicsQuantization>::Model, CorticalDynamicsModelCorePower<F16>>();
    assert_same_type::<<F16 as CorticalDynamicsQuantizationCorePower>::T1, f32>();
    assert_same_type::<<F16 as CorticalDynamicsQuantizationCorePower>::T2, f32>();

    type BF16 = CorticalDynamicsQuantizationCorePowerBFloat16;
    assert_same_type::<<BF16 as MembranePotentialQuantization>::MembranePotentialQuant, bf16>();
}

#[test]
fn cortical_data_properties_use_the_implementation_types() {
    let properties = CorticalDataPropertiesCorePower::<CorticalDynamicsQuantizationCorePowerFloat16> { t_1: 1.5 };
    assert_eq!(properties.t_1, 1.5f32);
}

#[test]
fn every_data_type_gets_a_struct() {
    type F32 = CorticalDynamicsQuantizationEveryDataFloat32;
    assert_same_type::<<F32 as MembranePotentialQuantization>::MembranePotentialQuant, f32>();
    assert_same_type::<<F32 as CorticalDynamicsQuantizationEveryData>::T2, f64>();

    let properties = CorticalDataPropertiesEveryData::<F32> { t_1: 1.0 };
    let internal = CorticalDataInternalEveryData::<F32> { t_1: 1.0, t_2: 2.0 };
    let _scratch = CorticalDataScratchEveryData::<F32>(core::marker::PhantomData);
    let context = CorticalFunctionContextEveryData::<F32> { context: 3.0 };
    let neuron_internal = NeuronDataInternalEveryData::<F32> { neuron: 4.0 };
    let neuron_scratch = NeuronDataScratchEveryData::<F32> { scratch: 5.0 };

    assert_eq!(properties.t_1, 1.0f32);
    assert_eq!(internal.t_2, 2.0f64);
    assert_eq!(context.context, 3.0f64);
    assert_eq!(neuron_internal.neuron, 4.0f32);
    assert_eq!(neuron_scratch.scratch, 5.0f64);
}
