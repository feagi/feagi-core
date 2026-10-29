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
///     // Optional, the NeuronLayout the model is restricted to. Either a single layout as here,
///     // or a bounded generic, such as `NeuronLayout: NL: VoxelNeuronLayout`. Defaults to any,
///     // `NL: NeuronLayout`
///     NeuronLayout: NeuronLayoutVoxelDefault
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
///   generic layout, restricting) its `Layout`, holding an associated type per quantizable
/// - One `NeuronModel{Model}{Level}` marker struct per quantization level, implementing
///   `NeuronModel` and the model trait. For a generic layout, it is generic over the layout,
///   `NeuronModel{Model}{Level}<NL>`
/// - A `{DataType}{Model}<NM>` struct for every data type, such as
///   `CorticalDataProperties{Model}`, with its fields public, a `PhantomData` of the
///   quantization, a `new` constructor, and its specification trait implemented. Data types not
///   given hold only the `PhantomData`
///
/// Must be in scope where this is expanded: `NeuronModel`, `MembranePotentialQuantization`, the six
/// data specification traits (such as `CorticalDataPropertiesSpecification`), any types the data
/// fields name, and any traits or types the `NeuronLayout` names (`NeuronLayout` when it is left as default). The crate must
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
