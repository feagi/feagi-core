//! The neuron models, and the `NeuronModel` family holding every one of them at each of its
//! quantization levels. Everything here is reached publicly through
//! `connectome_basis::neuron_model`, with a module per model such as
//! `connectome_basis::neuron_model::feagi_advanced`.
//!
//! Every generated name is prefixed `NM`. Two macro steps generate the items: one per model, and
//! one for the family as a whole.
//!
//! # Per model: `define_cortical_dynamics_quantization_template!`
//!
//! Each model folder (such as `feagi_advanced/`) expands its template in `data.rs`. The template
//! gives the model's `NeuronLayout` (the single layout type of the model), its
//! `NeurogenesisDefaults` and `NeurogenesisGenerator`, its `Quantizables`, its
//! `QuantizationLevels`, and the fields of its `DataSpecifications`. Using `FeagiAdvanced`, with
//! the quantizable `Period` and the level `DefaultQuant`, as the example, it generates:
//!
//! ```text
//! Period<T>                                   // a wrapper struct per quantizable
//!
//! trait NMFeagiAdvancedTrait: NeuronModel<    // the model trait, fixing the layout and every data type
//!     NeuronLayoutTag = NeuronLayoutTagVoxel,
//!     CorticalDataProperties = NMFeagiAdvancedDataCorticalProperties<Self>, ..
//! > {
//!     type Period;                            // an associated type per quantizable
//! }
//!
//! struct NMFeagiAdvancedQuantDefaultQuant;    // a unit marker struct per quantization level,
//!                                             // implementing NeuronModel and the model trait
//!
//! type NMFeagiAdvancedTag = NMFeagiAdvancedQuantDefaultQuant;
//!                                             // names the model as a whole (the first level)
//!
//! struct NMFeagiAdvancedData{DataType}<NM>    // one per data type of NeuronModel, 7 in all:
//!                                             //   CorticalProperties, CorticalInternal,
//!                                             //   CorticalScratch, CorticalVariance,
//!                                             //   NeuronInternal, NeuronScratch, NeuronVariance
//!                                             // with public fields, `new`, and `requantize`
//! ```
//!
//! A field typed by a quantizable becomes its wrapper at the level's quantization, such as
//! `Period<NM::Period>`. A field typed `MembranePotentialQuant` becomes `NM::MembranePotentialQuant`.
//! If a model gives no variance fields, its `CorticalDataVarianceSpecification` is generated.
//! Otherwise, as for `FeagiAdvanced`, it is written by hand in `data.rs`.
//!
//! Each model's `mod.rs` re-exports its generated items, and re-exports its
//! `{Model}BurstEngineDynamics` as `BurstEngineDynamics`.
//!
//! # The family: `define_model_family_template!`
//!
//! [`family`] lists every model and its quantization levels. These items are written by hand
//! there:
//!
//! - `NMContainer`, a kind of data held per member, as `type Of<M: NeuronModel>`. It is implemented
//!   on a zero sized struct per kind of data, such as `NeurogenesisFromStorageOrByGeneratorContainer`
//! - `NMMember`, implemented by every marker struct, with its model `Tag`, its `KIND`, and
//!   conversions of its data to and from `NMEnum`
//! - `NMAccessError`, returned when data is accessed as a member or model it does not hold
//!
//! The expansion of the family template generates:
//!
//! ```text
//! enum NMFeagiAdvancedQuantKind { DefaultQuant }     // per model, a variant per level, no data
//! enum NMKind { FeagiAdvanced(NMFeagiAdvancedQuantKind), .. }
//!
//! enum NMFeagiAdvancedQuantEnum<C: NMContainer> {    // per model, the data of C per level
//!     DefaultQuant(C::Of<NMFeagiAdvancedQuantDefaultQuant>),
//! }
//! enum NMEnum<C: NMContainer> { FeagiAdvanced(NMFeagiAdvancedQuantEnum<C>), .. }
//!                                                    // kind, from_data, try_as, try_as_mut,
//!                                                    // try_into_data, From and TryFrom between them
//!
//! trait NMFeagiAdvancedMember: NMMember + NMFeagiAdvancedTrait
//!                                                    // per model, converts to and from its QuantEnum
//! impl NMMember + NM{Model}Member for every marker struct
//!
//! with_nm!(value, <NM, Model> |data| body)           // dispatch over NMEnum
//! with_nm_feagi_advanced!(value, <NM, Model> |data| body)
//!                                                    // dispatch over one model's QuantEnum
//! ```
//!
//! The dispatch macros are exported at the crate root, and their paths start at
//! `$crate::connectome_basis::neuron_model`, so callers need no imports. In `body`, `NM` aliases
//! the marker struct of the member held, and `Model` aliases its model module, so
//! `Model::BurstEngineDynamics` can be used. Both aliases are optional.
//!
//! # Using the family
//!
//! [`neurogenesis_dispatch`] puts the family to use. `NeurogenesisFromStorageOrByGeneratorEnum<FIQ>` is
//! `NMEnum<NeurogenesisFromStorageOrByGeneratorContainer<FIQ>>`, and it dispatches each area to the burst
//! engine with its concrete model through `with_nm!`.
//!
//! # Adding a model
//!
//! 1. Add a folder named the model's snake case, holding `data.rs` (with its quantization template),
//!    `neurogenesis.rs`, and `burst_dynamics.rs` (with `{Model}BurstEngineDynamics`), and a `mod.rs`
//!    re-exporting them as the other models do.
//! 2. Declare the folder below, and re-export it in `connectome_basis::neuron_model`.
//! 3. Add the model and its quantization levels to the template in [`family`].

use feagi_connectome_neural::neural_basis::neuron_model::neurogenesis::from_storage::NeurogenesisFromStorage;
use feagi_connectome_neural::neural_basis::neuron_model::neurogenesis::from_storage_or_by_generator::NeurogenesisFromStorageOrByGenerator;
use feagi_connectome_neural::neural_basis::neuron_model::NeuronModel;
use feagi_connectome_neural::neural_basis::quantization_levels::FeagiIndexQuantization;
use models::feagi_advanced::quantizations::NMFeagiAdvancedQuantStandard32Bit;

//#[macro_use]
//pub mod family;
//pub mod neurogenesis_dispatch;


pub mod models;


// These are types that allow easy referencing to certain structs
//region Proxies

/// A struct of stored data for a neuron models data that can be used to generate that cortical area
/// as it was saved.
type ProxyNMNeuroGenesisFromStorage<FIQ: FeagiIndexQuantization, NM: NeuronModel>
    = NeurogenesisFromStorage<FIQ, NM>;

/// A struct for a given neuron model that allows creating it from minimal cortical level data.
/// This can be saved in the genome. Note implementation differs by Neuron Model
type ProxyNMNeuroGenesisFromGenerator<FIQ: FeagiIndexQuantization, NM: NeuronModel>
    = NM::NeurogenesisByGeneratorType<FIQ>;

/// An enum that that acts as a generator for a neuron model, that underneath can be of a storage
/// type or of the neuron models specific generator. Useful for exports where we may want
/// to optionally allow the use of storage, or not, to control export sizes
type ProxyNMNeurogenesisFromStorageOrByGenerator<FIQ: FeagiIndexQuantization, NM: NeuronModel>
    = NeurogenesisFromStorageOrByGenerator<FIQ, NM, NM::NeurogenesisByGeneratorType<FIQ>>;


//endregion

/// when going from some enums to others. `K` is the kind enum of the enum being unwrapped, such
/// as the `QuantKind` of a neuron model
#[derive(Debug, thiserror::Error)]
pub enum NeuronModelEnumConversionError<K: core::fmt::Debug> {
    #[error("Unable to unwrap given enum to neuron model and / or quantization! Expected {expected:?}, but the enum holds {actual:?}")]
    IncompatibleUnwrap {
        /// The kind the enum was asked to unwrap to
        expected: K,
        /// The kind the enum actually holds
        actual: K,
    }
}










// Replaced by neurogenesis, as the `NeuronInitializer` trait they implement no longer exists
//pub mod initializers;




//region Example: nested enums

// Building a cortical area's neurogenesis for one member of the family, nesting it into the
// family enums, and getting back down to the per model data fields of its generator. Checked to
// compile as an integration test of this crate, with these imports:
//



/*
fn example() {
    use crate::connectome_basis::neuron_model::{
        NMEnum, NMFeagiAdvancedQuantEnum, NMMember, NeurogenesisFromStorageOrByGeneratorEnum,
        NeurogenesisFromStorageOrByGeneratorOf, NeurogenesisGeneratorUniformDefault,
        NeurogenesisMethod, NeuronLayoutData, NeuronLayoutVoxel, NeuronModelUniversalCorticalProperties,
    };
    use crate::connectome_basis::quantization_levels::FeagiIndexQuantizationStandard;
    use feagi_basis::neurons::{CorticalAreaNeuronPotential, DimensionalCorticalAreaDimensions};
    use feagi_basis::prelude::*;
    use models::feagi_advanced::{
        NMFeagiAdvancedDataCorticalInternal, NMFeagiAdvancedDataCorticalProperties, NMFeagiAdvancedDataCorticalVariance,
        NMFeagiAdvancedDataNeuronInternal, Period,
    };

    // 1. The generator of a new area, holding every per model data struct of FeagiAdvanced
    let layout =
        NeuronLayoutVoxel::<FeagiIndexQuantizationStandard>::new(DimensionalCorticalAreaDimensions::new_dimensions_from_usize([2, 2, 1, 1]).unwrap());
    let generator = NeurogenesisGeneratorUniformDefault::<FeagiIndexQuantizationStandard, NMFeagiAdvancedQuantStandard32Bit>::new(
        layout,
        NeuronModelUniversalCorticalProperties {
            cortical_level_psp: CorticalAreaNeuronPotential::QUANT_ONE,
            is_psp_uniform: false,
            is_psp_using_neuron_potential: false,
        },
        // refractory_period, firing_threshold, leak_coefficient, leak_variability,
        // postsynaptic_current, postsynaptic_current_max, degeneration
        NMFeagiAdvancedDataCorticalProperties::new(Period::QUANT_ONE, 10.0, 0.5, 0.0, 1.0, 5.0, 0.0),
        NMFeagiAdvancedDataCorticalInternal::new(),
        // firing_threshold_increment_x, _y, _z
        NMFeagiAdvancedDataCorticalVariance::new(0.0, 0.0, 0.0),
        // refractory_countdown, which every neuron starts with
        NMFeagiAdvancedDataNeuronInternal::new(Period::QUANT_ZERO),
    );

    // 2. Nest it, innermost first. The container here is NeurogenesisFromStorageOrByGeneratorContainer, so
    //    the data held for a member is a NeurogenesisFromStorageOrByGeneratorOf: from storage, or by generator
    let neurogenesis: NeurogenesisFromStorageOrByGeneratorOf<FeagiIndexQuantizationStandard, NMFeagiAdvancedQuantStandard32Bit> =
        NeurogenesisFromStorageOrByGeneratorOf::ByGenerator(generator);
    let per_level = NMFeagiAdvancedQuantEnum::DefaultQuant(neurogenesis); // which quantization level
    let per_model: NeurogenesisFromStorageOrByGeneratorEnum<FeagiIndexQuantizationStandard> = NMEnum::FeagiAdvanced(per_level); // which model

    // The same three steps in one, by the member's marker struct
    let same: NeurogenesisFromStorageOrByGeneratorEnum<FeagiIndexQuantizationStandard> =
        NMEnum::from_data::<NMFeagiAdvancedQuantStandard32Bit>(NeurogenesisFromStorageOrByGeneratorOf::ByGenerator(NeurogenesisGeneratorUniformDefault::new(
            layout,
            NeuronModelUniversalCorticalProperties {
                cortical_level_psp: CorticalAreaNeuronPotential::QUANT_ONE,
                is_psp_uniform: false,
                is_psp_using_neuron_potential: false,
            },
            NMFeagiAdvancedDataCorticalProperties::new(Period::QUANT_ONE, 10.0, 0.5, 0.0, 1.0, 5.0, 0.0),
            NMFeagiAdvancedDataCorticalInternal::new(),
            NMFeagiAdvancedDataCorticalVariance::new(0.0, 0.0, 0.0),
            NMFeagiAdvancedDataNeuronInternal::new(Period::QUANT_ZERO),
        )));
    assert_eq!(per_model.kind(), same.kind());
    assert_eq!(per_model.kind(), <NMFeagiAdvancedQuantStandard32Bit as NMMember>::KIND);

    // 3. Back down to the per model data fields, by matching every layer
    match &per_model {
        NMEnum::FeagiAdvanced(NMFeagiAdvancedQuantEnum::DefaultQuant(NeurogenesisFromStorageOrByGeneratorOf::ByGenerator(generator))) => {
            let _threshold: f32 = generator.cortical_properties.firing_threshold;
            let _countdown: Period<f32> = generator.neuron_internal.refractory_countdown;
        }
        _ => unreachable!(),
    }

    //    or by the member, when it is known, which errors if another member is held
    let neurogenesis: &NeurogenesisFromStorageOrByGeneratorOf<FeagiIndexQuantizationStandard, NMFeagiAdvancedQuantStandard32Bit> =
        per_model.try_as::<NMFeagiAdvancedQuantStandard32Bit>().unwrap();
    if let NeurogenesisFromStorageOrByGeneratorOf::ByGenerator(generator) = neurogenesis {
        let _leak = generator.cortical_properties.leak_coefficient;
    }

    //    or generically over whichever member is held, through the NeurogenesisMethod trait
    let neuron_count = with_nm!(&per_model, |neurogenesis| neurogenesis.get_total_number_neurons_needed().quant_to_usize());
    assert_eq!(neuron_count, 4);
}


 */
//endregion
