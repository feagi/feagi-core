//region Templates

//region Cortical Dynamics Quantization

use feagi_macro_components::cortical_dynamics::cortical_dynamics_quantization::CorticalDynamicsQuantizationTemplate;

/// Defines and validates the quantization template of a cortical dynamics model. Generates the
/// template macro `cortical_dynamics_quantization_{model}_template!`, whose single input is the
/// path of a macro to call with the validated template. Pass it
/// [`expand_cortical_dynamics_quantization`] to generate the model's quantization types
///
/// # Example
///
/// ```ignore
/// define_cortical_dynamics_quantization_template! {
///     /// Quantization for the Core Power model
///     CorePower: {
///         /// Optional description
///         T1: QuantizedDecimalUnwrappedTrait,
///     }
///
///     implementations: {
///         Float16: {
///             MembranePotentialQuant: f16,
///             T1: f32,
///         }
///
///         Float32: {
///             MembranePotentialQuant: f32,
///             T1: f32,
///         }
///     }
///
///     // Optional, one per data type of `CorticalDynamicsModel`: CorticalDataProperties,
///     // CorticalDataInternal, CorticalDataScratch, CorticalFunctionContext,
///     // NeuronDataInternal, NeuronDataScratch
///     CorticalDataProperties: {
///         t_1: T1,
///     }
///
///     NeuronDataInternal: {
///         n_1: T1,
///     }
/// }
///
/// cortical_dynamics_quantization_core_power_template!(expand_cortical_dynamics_quantization);
/// ```
#[proc_macro]
pub fn define_cortical_dynamics_quantization_template(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    match syn::parse::<CorticalDynamicsQuantizationTemplate>(input) {
        Err(e) => e.to_compile_error().into(),
        Ok(template) => template.generate_rust_template_macro().into(),
    }
}

/// Expands a cortical dynamics quantization template, as given by its template macro. Generates
/// the model's `CorticalDynamicsQuantization{Model}` trait, one
/// `CorticalDynamicsQuantization{Model}{Implementation}` marker struct per implementation, and
/// a `{DataType}{Model}` struct per data type given, such as `CorticalDataProperties{Model}`.
/// Data types given without fields hold only a `PhantomData` of the quantization. The model struct `CorticalDynamicsModel{Model}` must be in
/// scope where this is expanded
#[proc_macro]
pub fn expand_cortical_dynamics_quantization(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    match syn::parse::<CorticalDynamicsQuantizationTemplate>(input) {
        Err(e) => e.to_compile_error().into(),
        Ok(template) => template.generate_rust_quantization().into(),
    }
}

//endregion

//endregion
