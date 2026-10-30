//region Templates

//region Cortical Dynamics Quantization

use feagi_macro_components::cortical_dynamics::cortical_dynamics_quantization::CorticalDynamicsQuantizationTemplate;

/// Defines and validates the quantization template of a cortical dynamics model. Generates the
/// template macro `cortical_dynamics_quantization_{model}_template!`, whose single input is the
/// path of a macro to call with the validated template. Pass it
/// [`expand_cortical_dynamics_quantization`] to generate the model's types
///
/// # Example
///
/// ```ignore
/// define_cortical_dynamics_quantization_template! {
///     /// Quantization for the Feagi Advanced model
///     FeagiAdvanced
///
///     // Optional, the NeuronLayoutType the model is restricted to. Either a single layout type as
///     // here, or a bounded generic, such as `NeuronLayout: NL: NeuronLayoutType`. Defaults to any,
///     // `NL: NeuronLayoutType`
///     NeuronLayout: NeuronLayoutVoxelDefaultType
///
///     // Optional, each one of QDECIMAL, QUINT or QSINT
///     Quantizables: {
///         /// How long till the neuron can fire again
///         RefractoryPeriod: QDECIMAL,
///     }
///
///     // Each sets MembranePotentialQuant and every quantizable
///     QuantizationLevels: {
///         Float32: {
///             MembranePotentialQuant: f32,
///             RefractoryPeriod: f32,
///         }
///     }
///
///     // Optional, as are each of its data types: CorticalDataProperties, CorticalDataInternal,
///     // CorticalDataScratch, CorticalFunctionContext, NeuronDataInternal, NeuronDataScratch
///     DataSpecifications: {
///         CorticalDataProperties: {
///             /// Becomes `RefractoryPeriod<NM::RefractoryPeriod>`
///             refractory_period: RefractoryPeriod,
///             /// Kept as written
///             some_count: u32,
///         }
///     }
/// }
///
/// cortical_dynamics_quantization_feagi_advanced_template!(expand_cortical_dynamics_quantization);
/// ```
#[proc_macro]
pub fn define_cortical_dynamics_quantization_template(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    match syn::parse::<CorticalDynamicsQuantizationTemplate>(input) {
        Err(e) => e.to_compile_error().into(),
        Ok(template) => template.generate_rust_template_macro().into(),
    }
}

/// Expands a cortical dynamics quantization template, as given by its template macro. Generates:
/// - A wrapper struct per quantizable, through `feagi_basis`'s `create_wrapped_quantized_*!`
/// - The model's `NeuronModel{Model}` trait, a sub trait of `NeuronModel` fixing (or, for a
///   generic layout, restricting) its `LayoutType` and fixing each of its data types to the
///   model's data struct of that type, holding an associated type per quantizable
/// - One `NeuronModel{Model}{Level}` marker struct per quantization level, implementing
///   `NeuronModel` and the model trait. For a generic layout, it is generic over the layout,
///   `NeuronModel{Model}{Level}<NL>`
/// - A `{DataType}{Model}<NM>` struct for every data type, such as
///   `CorticalDataProperties{Model}`, with its fields public, a `PhantomData` of the
///   quantization, a const `new` constructor, a `requantize` to any other quantization level of
///   the model, and its specification trait implemented. `requantize` errors if a decimal becomes
///   infinite or NaN, clamps integers, and clones any other field. Data types not given hold only
///   the `PhantomData`
/// - For each data container, a `{Model}{Container}<FIQ, NM>` alias and a
///   `{Model}{Container}Enum<FIQ>` enum with a variant per quantization level:
///   `{Model}FullStorageData<FIQ, NM, SB>` and `{Model}FullStorageDataQuantEnum<FIQ, SB>`,
///   `{Model}GenomePropertiesData` and `{Model}GenomePropertiesDataEnum`, and
///   `{Model}NPUInterfaceData` and `{Model}NPUInterfaceDataEnum`. For a generic layout, the enums
///   are generic over the layout as well
/// - The model's `{Model}Defaults` trait, a sub trait of `CorticalUniversalDefaults` with its
///   `Reference` a level of the model, holding a const of every data type except the cortical
///   properties at the `Reference` quantization. Its default `assemble` takes the cortical
///   properties and layout at any level of the model, and returns a `CorticalModelSpawnData` of
///   them and every default requantized to that level. Meant to be implemented on zero sized
///   structs, one per set of defaults
///
/// Must be in scope where this is expanded: `NeuronModel`, `NeuronLayoutOf`, `MembranePotentialQuantization`, the six
/// data specification traits (such as `CorticalDataPropertiesSpecification`), `FeagiIndexQuantization`,
/// the data containers `CorticalModelFullStorageData`, `CorticalModelGenomePropertiesData`,
/// `CorticalModelNPUInterfaceData` and `CorticalModelSpawnData`, `CorticalUniversalDefaults`, any types the data
/// fields name, and any traits or types the `NeuronLayout` names (`NeuronLayoutType` when it is left as default). The crate must
/// depend on `serde` and `half`, which the wrapper structs use
#[proc_macro]
pub fn expand_cortical_dynamics_quantization(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    match syn::parse::<CorticalDynamicsQuantizationTemplate>(input) {
        Err(e) => e.to_compile_error().into(),
        Ok(template) => template.generate_rust_quantization().into(),
    }
}

//endregion

//endregion
