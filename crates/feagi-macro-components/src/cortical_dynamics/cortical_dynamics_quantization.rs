use crate::common::{basis_path, parse_optional_comma};
use crate::model_family::model_family_template as model_names;
use heck::ToSnakeCase;
use indexmap::IndexMap;
use proc_macro2::Ident;
use quote::{format_ident, quote, ToTokens};
use syn::parse::{Parse, ParseStream};
use syn::{braced, Attribute, Token, Type};

mod kw {
    syn::custom_keyword!(NeuronLayout);
    syn::custom_keyword!(NeurogenesisDefaults);
    syn::custom_keyword!(NeurogenesisGenerator);
    syn::custom_keyword!(Quantizables);
    syn::custom_keyword!(QuantizationLevels);
    syn::custom_keyword!(DataSpecifications);
}

/// Name of the quantization level entry that sets `MembranePotentialQuantization::MembranePotentialQuant`
const MEMBRANE_POTENTIAL_QUANT: &str = "MembranePotentialQuant";

/// Data type of `NeuronModel` that the model's per neuron variance is derived from, whose
/// specification trait has a function to implement
const CORTICAL_DATA_VARIANCE: &str = "CorticalDataVariance";

/// Data type of `NeuronModel` derived from [`CORTICAL_DATA_VARIANCE`], so it is never serialized
/// nor has a default
const NEURON_DATA_VARIANCE: &str = "NeuronDataVariance";

/// Name of the neuron model generic of the generated data structs
const NEURON_MODEL_GENERIC: &str = "NM";

/// Name of the `FeagiIndexQuantization` generic of the generated cortical variance specification
const FEAGI_INDEX_QUANTIZATION_GENERIC: &str = "FIQ";

/// Name of the marker field every generated data struct holds
const PHANTOM_FIELD: &str = "_p";

/// The data types of `NeuronModel` in trait order, with the specification trait each of their
/// structs implements
pub const MODEL_DATA_STRUCTS: [(&str, &str); 7] = [
    ("CorticalDataProperties", "CorticalDataPropertiesSpecification"),
    ("CorticalDataInternal", "CorticalDataInternalSpecification"),
    ("CorticalDataScratch", "CorticalDataScratchSpecification"),
    (CORTICAL_DATA_VARIANCE, "CorticalDataVarianceSpecification"),
    ("NeuronDataInternal", "NeuronDataInternalSpecification"),
    ("NeuronDataScratch", "NeuronDataScratchSpecification"),
    (NEURON_DATA_VARIANCE, "NeuronDataVarianceSpecification"),
];

//region Cortical Dynamics Quantization Template

/// The quantization types of one cortical dynamics model. Formatted as the following:
/// ```text
/// /// Optional model description
/// ModelName
///
/// NeuronLayout: NeuronLayoutTagVoxel
///
/// NeurogenesisDefaults: ModelNameNeurogenesisDefaults<FIQ, Self>
///
/// NeurogenesisGenerator: ModelNameNeurogenesisGenerator<FIQ, Self>
///
/// Quantizables: {
///     /// Optional description
///     RefractoryPeriod: QDECIMAL,
/// }
///
/// QuantizationLevels: {
///     Float32: {
///         MembranePotentialQuant: f32,
///         RefractoryPeriod: f32,
///     }
/// }
///
/// DataSpecifications: {
///     CorticalDataProperties: {
///         /// Optional description
///         refractory_period: RefractoryPeriod,
///         some_count: u32,
///     }
/// }
/// ```
/// The required `NeuronLayout` entry is the single `NeuronLayoutTag` of the model, which every
/// quantization level sets and the model trait fixes. The required `NeurogenesisDefaults` and `NeurogenesisGenerator` are the types every quantization level sets
/// as `NeuronModel::NeurogenesisDefaults<FIQ>` and `NeuronModel::NeurogenesisGenerator<FIQ>`,
/// written in terms of the generic `FIQ` and `Self`, the quantization level. They must implement
/// `NeurogenesisScratchVarianceGenerator<FIQ, Self>` and `NeurogenesisByGenerator<FIQ, Self>`
/// respectively, for every level of the model. The optional `Quantizables` each become a wrapper struct of their kind (see [`QuantizableKind`]) and an
/// associated type of the model. Every quantization level must set `MembranePotentialQuant` and
/// each quantizable once. The optional `DataSpecifications` hold, in any order, the fields of the
/// model's data structs, one per data type of `NeuronModel` (see
/// [`MODEL_DATA_STRUCTS`]). A field typed by a quantizable is its wrapper at the level's
/// quantization, and `MembranePotentialQuant` the level's membrane potential type. Any other type
/// is kept as written. Entries keep their template order, except data structs, which keep trait
/// order. A model giving fields for neither `CorticalDataVariance` nor `NeuronDataVariance` has its
/// variance generated as nothing, otherwise `CorticalDataVarianceSpecification` must be implemented
/// by hand for `NM{Model}DataCorticalVariance`
pub struct CorticalDynamicsQuantizationTemplate {
    pub model_attributes: Vec<Attribute>,
    pub model_name: Ident,
    pub layout: Type,
    pub neurogenesis_defaults: Type,
    pub neurogenesis_generator: Type,
    pub quantizables: IndexMap<Ident, QuantizableTemplate>,
    pub quantization_levels: IndexMap<Ident, QuantizationLevelTemplate>,
    pub data_specifications: IndexMap<Ident, IndexMap<Ident, DataFieldTemplate>>,
}

/// The kind of a quantizable, deciding its wrapper and the bound of its associated type
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum QuantizableKind {
    /// `QDECIMAL`, wrapped by `create_wrapped_quantized_decimal!`
    Decimal,
    /// `QUINT`, wrapped by `create_wrapped_quantized_unsigned_integer!`
    UnsignedInteger,
    /// `QSINT`, wrapped by `create_wrapped_quantized_signed_integer!`
    SignedInteger,
}

impl QuantizableKind {
    const KEYWORDS: [(&'static str, QuantizableKind); 3] = [
        ("QDECIMAL", QuantizableKind::Decimal),
        ("QUINT", QuantizableKind::UnsignedInteger),
        ("QSINT", QuantizableKind::SignedInteger),
    ];

    pub fn keyword(self) -> &'static str {
        Self::KEYWORDS.iter().find(|(_, kind)| *kind == self).map(|(keyword, _)| *keyword).unwrap_or_default()
    }

    /// The bound of the quantizable's associated type, the unwrapped quantization trait of its kind
    pub fn unwrapped_trait(self) -> proc_macro2::TokenStream {
        let basis = basis_path();
        match self {
            Self::Decimal => quote!(#basis::quantization::quantizable::QuantizedDecimalUnwrappedTrait),
            Self::UnsignedInteger => quote!(#basis::quantization::quantizable::QuantizedUnsignedIntegerUnwrappedTrait),
            Self::SignedInteger => quote!(#basis::quantization::quantizable::QuantizedSignedIntegerUnwrappedTrait),
        }
    }

    /// The `feagi_basis` macro creating the quantizable's wrapper struct
    pub fn wrapper_macro(self) -> proc_macro2::TokenStream {
        let basis = basis_path();
        match self {
            Self::Decimal => quote!(#basis::create_wrapped_quantized_decimal),
            Self::UnsignedInteger => quote!(#basis::create_wrapped_quantized_unsigned_integer),
            Self::SignedInteger => quote!(#basis::create_wrapped_quantized_signed_integer),
        }
    }
}

impl Parse for QuantizableKind {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let keyword = input.parse::<Ident>()?;
        Self::KEYWORDS.iter().find(|(name, _)| keyword == name).map(|(_, kind)| *kind).ok_or_else(|| {
            let expected: Vec<&str> = Self::KEYWORDS.iter().map(|(name, _)| *name).collect();
            syn::Error::new(keyword.span(), format!("`{keyword}` is not a quantizable kind, expected one of: {}", expected.join(", ")))
        })
    }
}

impl ToTokens for QuantizableKind {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        format_ident!("{}", self.keyword()).to_tokens(tokens);
    }
}

/// A quantizable of the model, which is both a wrapper struct and an associated type of the model
/// trait
pub struct QuantizableTemplate {
    pub attributes: Vec<Attribute>,
    pub kind: QuantizableKind,
}

/// One quantization level of the model, with the concrete type of each quantizable
pub struct QuantizationLevelTemplate {
    pub membrane_potential_quant: Type,
    pub quantizables: IndexMap<Ident, Type>,
}

/// A field of a model data struct
pub struct DataFieldTemplate {
    pub attributes: Vec<Attribute>,
    pub field_type: Type,
}

impl CorticalDynamicsQuantizationTemplate {
    /// `NM{Model}Trait`
    pub fn model_trait_name(&self) -> Ident {
        model_names::model_trait_name(&self.model_name)
    }

    /// `NM{Model}Tag`
    pub fn tag_name(&self) -> Ident {
        model_names::model_tag_name(&self.model_name)
    }

    /// `NM{Model}Quant{Level}`
    pub fn quantization_level_struct_name(&self, level: &Ident) -> Ident {
        model_names::quantization_level_struct_name(&self.model_name, level)
    }

    /// `NM{Model}Data{DataType}`, where `data_struct` is one of [`MODEL_DATA_STRUCTS`]
    pub fn data_struct_name(&self, data_struct: &Ident) -> Ident {
        model_names::model_data_struct_name(&self.model_name, data_struct)
    }

    pub fn template_macro_name(&self) -> Ident {
        format_ident!("cortical_dynamics_quantization_{}_template", self.model_name.to_string().to_snake_case())
    }

    /// The type a data field is generated with. A quantizable is its wrapper at the level's
    /// quantization, `Name<NM::Name>`, and `MembranePotentialQuant` is
    /// `NM::MembranePotentialQuant`. Anything else is kept as written
    fn resolve_field_type(&self, field_type: &Type) -> proc_macro2::TokenStream {
        let nm = format_ident!("{}", NEURON_MODEL_GENERIC);
        if let Some(name) = bare_ident(field_type) {
            if self.quantizables.contains_key(name) {
                return quote!(#name<#nm::#name>);
            }
            if name == MEMBRANE_POTENTIAL_QUANT {
                return quote!(#nm::#name);
            }
        }
        field_type.to_token_stream()
    }

    /// The expression converting data field `self.name` to the quantization of another model level
    /// in `requantize`. Decimals, `MembranePotentialQuant` included, error if they become infinite
    /// or NaN, integers are clamped, and anything else is cloned
    fn requantize_field(&self, name: &Ident, field_type: &Type) -> proc_macro2::TokenStream {
        let basis = basis_path();
        let kind = bare_ident(field_type).and_then(|type_name| {
            if type_name == MEMBRANE_POTENTIAL_QUANT {
                Some(QuantizableKind::Decimal)
            } else {
                self.quantizables.get(type_name).map(|quantizable| quantizable.kind)
            }
        });
        match kind {
            Some(QuantizableKind::Decimal) => {
                quote!(#basis::quantization::quantizable::QuantizedDecimalTrait::try_to_quantization(self.#name)?)
            }
            Some(QuantizableKind::UnsignedInteger) => {
                quote!(#basis::quantization::quantizable::QuantizedUnsignedIntegerTrait::to_quantization_clamped(self.#name))
            }
            Some(QuantizableKind::SignedInteger) => {
                quote!(#basis::quantization::quantizable::QuantizedSignedIntegerTrait::to_quantization_clamped(self.#name))
            }
            None => quote!(::core::clone::Clone::clone(&self.#name)),
        }
    }

    /// Generate the template macro, which holds this template once validated. Its single input is
    /// the path of a macro, which it calls with the template, as in
    /// `cortical_dynamics_quantization_{model}_template!(expand_cortical_dynamics_quantization)`
    pub fn generate_rust_template_macro(&self) -> proc_macro2::TokenStream {
        let macro_name = self.template_macro_name();
        let doc = syn::LitStr::new(
            &format!("The validated cortical dynamics quantization template of the `{}` model. Calls the given macro with it", self.model_name),
            proc_macro2::Span::call_site(),
        );
        quote! {
            #[doc = #doc]
            macro_rules! #macro_name {
                ($($callback:tt)+) => {
                    $($callback)+! {
                        #self
                    }
                };
            }
        }
    }

    /// Generate the model's quantizable wrappers, model trait, a marker struct implementing it per
    /// quantization level, every data struct of the model, and the model's tag
    pub fn generate_rust_quantization(&self) -> proc_macro2::TokenStream {
        let model_attributes = &self.model_attributes;
        let model_trait = self.model_trait_name();

        let wrappers = self.quantizables.iter().map(|(name, quantizable)| {
            let attributes = &quantizable.attributes;
            let wrapper_macro = quantizable.kind.wrapper_macro();
            quote! {
                #wrapper_macro!(
                    #( #attributes )*
                    pub #name
                );
            }
        });

        let associated_types = self.quantizables.iter().map(|(name, quantizable)| {
            let attributes = &quantizable.attributes;
            let bound = quantizable.kind.unwrapped_trait();
            quote! {
                #( #attributes )*
                type #name: #bound;
            }
        });

        // The single layout type of the model, which every level sets and the model trait fixes
        let layout = &self.layout;

        // Every data type of `NeuronModel` is set to the model's data struct of that type
        let model_data_names: Vec<Ident> = MODEL_DATA_STRUCTS.iter().map(|(name, _)| format_ident!("{}", name)).collect();
        let model_data_structs: Vec<Ident> = model_data_names.iter().map(|name| self.data_struct_name(name)).collect();

        let neurogenesis_defaults = &self.neurogenesis_defaults;
        let neurogenesis_generator = &self.neurogenesis_generator;
        let fiq = format_ident!("{}", FEAGI_INDEX_QUANTIZATION_GENERIC);
        let quantization_levels = self.quantization_levels.iter().map(|(name, level)| {
            let struct_name = self.quantization_level_struct_name(name);
            let membrane_potential_quant = &level.membrane_potential_quant;
            let quantizable_names = level.quantizables.keys();
            let quantizable_types = level.quantizables.values();
            let model_data_names = &model_data_names;
            let model_data_structs = &model_data_structs;
            quote! {
                #[derive(Clone, Copy, PartialEq, Debug, ::serde::Serialize, ::serde::Deserialize)]
                pub struct #struct_name;

                impl MembranePotentialQuantization for #struct_name {
                    type MembranePotentialQuant = #membrane_potential_quant;
                }

                impl NeuronModel for #struct_name {
                    type NeuronLayoutTag = #layout;
                    type NeurogenesisDefaults<#fiq: FeagiIndexQuantization> = #neurogenesis_defaults;
                    type NeurogenesisGenerator<#fiq: FeagiIndexQuantization> = #neurogenesis_generator;
                    #( type #model_data_names = #model_data_structs<Self>; )*
                }

                impl #model_trait for #struct_name {
                    #( type #quantizable_names = #quantizable_types; )*
                }
            }
        });

        let nm = format_ident!("{}", NEURON_MODEL_GENERIC);
        let basis = basis_path();
        let quantization_error = quote!(#basis::quantization::quantizable::DataValueQuantizationError);
        let fiq = format_ident!("{}", FEAGI_INDEX_QUANTIZATION_GENERIC);
        let no_fields = IndexMap::new();
        let has_variance = [CORTICAL_DATA_VARIANCE, NEURON_DATA_VARIANCE].iter().any(|name| self.data_specifications.contains_key(&format_ident!("{}", name)));
        let data_structs = MODEL_DATA_STRUCTS.iter().map(|(data_struct, specification)| {
            let data_struct = format_ident!("{}", data_struct);
            let specification = format_ident!("{}", specification);
            let struct_name = self.data_struct_name(&data_struct);
            let fields = self.data_specifications.get(&data_struct).unwrap_or(&no_fields);
            let field_attributes: Vec<&Vec<Attribute>> = fields.values().map(|field| &field.attributes).collect();
            let field_names: Vec<&Ident> = fields.keys().collect();
            let field_types: Vec<proc_macro2::TokenStream> = fields.values().map(|field| self.resolve_field_type(&field.field_type)).collect();
            let field_conversions = fields.iter().map(|(name, field)| self.requantize_field(name, &field.field_type));
            let quantization_error = &quantization_error;
            let (serde_derive, phantom_attribute) = match data_struct == NEURON_DATA_VARIANCE {
                true => (quote!(), quote!()),
                false => (quote!(, ::serde::Serialize, ::serde::Deserialize), quote!(#[serde(skip)])),
            };
            // The cortical variance specification has a function, which only a model without any
            // variance can have written for it
            let specification_impl = match data_struct == CORTICAL_DATA_VARIANCE {
                false => quote!(impl<#nm: #model_trait> #specification<#nm> for #struct_name<#nm> {}),
                true if has_variance => quote!(),
                true => {
                    let neuron_variance = self.data_struct_name(&format_ident!("{}", NEURON_DATA_VARIANCE));
                    quote! {
                        impl<#nm: #model_trait> #specification<#nm> for #struct_name<#nm> {
                            fn neuron_variance_at<#fiq: FeagiIndexQuantization>(
                                &self,
                                _layout: &NeuronLayoutOf<#nm, #fiq>,
                                _index: &#basis::neurons::CorticalAreaNeuronLocalIndex<#fiq::NeuronIndexQuant>,
                            ) -> #neuron_variance<#nm> {
                                #neuron_variance::new()
                            }
                        }
                    }
                }
            };
            quote! {
                #[derive(Clone, Debug, PartialEq #serde_derive)]
                pub struct #struct_name<#nm: #model_trait> {
                    #(
                        #( #field_attributes )*
                        pub #field_names: #field_types,
                    )*
                    #phantom_attribute
                    /// Public so that this struct may be written out with its fields named
                    pub _p: ::core::marker::PhantomData<#nm>,
                }

                impl<#nm: #model_trait> #struct_name<#nm> {
                    pub const fn new(#( #field_names: #field_types ),*) -> Self {
                        Self {
                            #( #field_names, )*
                            _p: ::core::marker::PhantomData,
                        }
                    }

                    /// Converts to the quantization of `To`. Returns an error if a decimal becomes
                    /// infinite or NaN
                    pub fn requantize<To: #model_trait>(&self) -> ::core::result::Result<#struct_name<To>, #quantization_error> {
                        ::core::result::Result::Ok(#struct_name::new(#( #field_conversions ),*))
                    }
                }

                #specification_impl
            }
        });

        // The tag names the model as a whole, a type distinct per model. It is the first
        // quantization level (parsing requires at least one), so its `NeuronModel::NeuronLayoutTag`
        // is the layout of the model, which every level shares
        let tag = self.quantization_levels.keys().next().map(|first_level| {
            let tag_name = self.tag_name();
            let first_level_struct = self.quantization_level_struct_name(first_level);
            let tag_doc = format!(
                "Names the `{}` model as a whole, across its quantization levels. Its `NeuronModel::NeuronLayoutTag` is the layout of the model",
                self.model_name
            );
            quote! {
                #[doc = #tag_doc]
                pub type #tag_name = #first_level_struct;
            }
        });

        quote! {
            #( #wrappers )*

            #( #model_attributes )*
            pub trait #model_trait: NeuronModel<NeuronLayoutTag = #layout, #( #model_data_names = #model_data_structs<Self>, )*> {
                #( #associated_types )*
            }

            #( #quantization_levels )*

            #( #data_structs )*

            #tag
        }
    }
}

impl Parse for CorticalDynamicsQuantizationTemplate {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let model_attributes = input.call(Attribute::parse_outer)?;
        let model_name = input.parse::<Ident>()?;
        parse_optional_comma(input)?;

        // Required, as every model has exactly one layout type
        input.parse::<kw::NeuronLayout>()?;
        input.parse::<Token![:]>()?;
        let layout = input.parse::<Type>()?;
        parse_optional_comma(input)?;

        // Required, as there is no default way to generate scratch and variance data
        input.parse::<kw::NeurogenesisDefaults>()?;
        input.parse::<Token![:]>()?;
        let neurogenesis_defaults = input.parse::<Type>()?;
        parse_optional_comma(input)?;

        // Required, as there is no default way to generate neurons
        input.parse::<kw::NeurogenesisGenerator>()?;
        input.parse::<Token![:]>()?;
        let neurogenesis_generator = input.parse::<Type>()?;
        parse_optional_comma(input)?;

        let quantizables = if input.peek(kw::Quantizables) {
            input.parse::<kw::Quantizables>()?;
            input.parse::<Token![:]>()?;
            let quantizables = parse_quantizables(input)?;
            parse_optional_comma(input)?;
            quantizables
        } else {
            IndexMap::new()
        };

        input.parse::<kw::QuantizationLevels>()?;
        input.parse::<Token![:]>()?;
        let quantization_levels = parse_quantization_levels(input, &quantizables)?;
        parse_optional_comma(input)?;

        let data_specifications = if input.peek(kw::DataSpecifications) {
            input.parse::<kw::DataSpecifications>()?;
            input.parse::<Token![:]>()?;
            let data_specifications = parse_data_specifications(input)?;
            parse_optional_comma(input)?;
            data_specifications
        } else {
            IndexMap::new()
        };

        if !input.is_empty() {
            return Err(input.error("unexpected input after the template, `DataSpecifications` is the last section"));
        }

        Ok(Self {
            model_attributes,
            model_name,
            layout,
            neurogenesis_defaults,
            neurogenesis_generator,
            quantizables,
            quantization_levels,
            data_specifications,
        })
    }
}

/// Writes the template back out in the format it is parsed from, with quantization levels setting
/// `MembranePotentialQuant` first and then the quantizables in model order
impl ToTokens for CorticalDynamicsQuantizationTemplate {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let model_attributes = &self.model_attributes;
        let model_name = &self.model_name;
        let layout = &self.layout;
        let neurogenesis_defaults = &self.neurogenesis_defaults;
        let neurogenesis_generator = &self.neurogenesis_generator;

        let quantizables = self.quantizables.iter().map(|(name, quantizable)| {
            let attributes = &quantizable.attributes;
            let kind = &quantizable.kind;
            quote! {
                #( #attributes )*
                #name: #kind,
            }
        });

        let quantization_levels = self.quantization_levels.iter().map(|(name, level)| {
            let membrane_potential_quant = &level.membrane_potential_quant;
            let quantizable_names = level.quantizables.keys();
            let quantizable_types = level.quantizables.values();
            quote! {
                #name: {
                    MembranePotentialQuant: #membrane_potential_quant,
                    #( #quantizable_names: #quantizable_types, )*
                }
            }
        });

        let data_specifications = self.data_specifications.iter().map(|(data_struct, fields)| {
            let fields = fields.iter().map(|(name, field)| {
                let attributes = &field.attributes;
                let field_type = &field.field_type;
                quote! {
                    #( #attributes )*
                    #name: #field_type,
                }
            });
            quote! {
                #data_struct: {
                    #( #fields )*
                }
            }
        });

        tokens.extend(quote! {
            #( #model_attributes )*
            #model_name

            NeuronLayout: #layout,

            NeurogenesisDefaults: #neurogenesis_defaults,

            NeurogenesisGenerator: #neurogenesis_generator,

            Quantizables: {
                #( #quantizables )*
            }

            QuantizationLevels: {
                #( #quantization_levels )*
            }

            DataSpecifications: {
                #( #data_specifications )*
            }
        });
    }
}

/// The name of a type that is a single identifier, such as a quantizable or `MembranePotentialQuant`
fn bare_ident(field_type: &Type) -> Option<&Ident> {
    match field_type {
        Type::Path(path) if path.qself.is_none() => path.path.get_ident(),
        _ => None,
    }
}

/// `{ #[attr] Name: QDECIMAL, ... }`
fn parse_quantizables(input: ParseStream) -> syn::Result<IndexMap<Ident, QuantizableTemplate>> {
    let field_input;
    braced!(field_input in input);
    let mut quantizables = IndexMap::new();
    while !field_input.is_empty() {
        let attributes = field_input.call(Attribute::parse_outer)?;
        let name = field_input.parse::<Ident>()?;
        if name == MEMBRANE_POTENTIAL_QUANT {
            return Err(syn::Error::new(name.span(), format!("`{MEMBRANE_POTENTIAL_QUANT}` is set by every quantization level, it cannot be a quantizable")));
        }
        field_input.parse::<Token![:]>()?;
        let kind = field_input.parse::<QuantizableKind>()?;
        if quantizables.contains_key(&name) {
            return Err(syn::Error::new(name.span(), format!("quantizable `{name}` is defined more than once")));
        }
        quantizables.insert(name, QuantizableTemplate { attributes, kind });
        if field_input.is_empty() {
            break;
        }
        field_input.parse::<Token![,]>()?;
    }
    Ok(quantizables)
}

/// `{ Name: { MembranePotentialQuant: Type, T1: Type, ... } ... }`, the comma between levels
/// being optional
fn parse_quantization_levels(
    input: ParseStream,
    quantizables: &IndexMap<Ident, QuantizableTemplate>,
) -> syn::Result<IndexMap<Ident, QuantizationLevelTemplate>> {
    let levels_input;
    let brace = braced!(levels_input in input);
    let mut levels = IndexMap::new();
    while !levels_input.is_empty() {
        let level_name = levels_input.parse::<Ident>()?;
        levels_input.parse::<Token![:]>()?;
        let level = parse_quantization_level(&levels_input, &level_name, quantizables)?;
        if levels.contains_key(&level_name) {
            return Err(syn::Error::new(level_name.span(), format!("quantization level `{level_name}` is defined more than once")));
        }
        levels.insert(level_name, level);
        parse_optional_comma(&levels_input)?;
    }
    if levels.is_empty() {
        return Err(syn::Error::new(brace.span.join(), "a model needs at least one quantization level"));
    }
    Ok(levels)
}

/// `{ MembranePotentialQuant: Type, T1: Type, ... }`, in any order. Concrete types come out in
/// the model's quantizable order
fn parse_quantization_level(
    input: ParseStream,
    level_name: &Ident,
    quantizables: &IndexMap<Ident, QuantizableTemplate>,
) -> syn::Result<QuantizationLevelTemplate> {
    let field_input;
    let brace = braced!(field_input in input);
    let mut membrane_potential_quant = None;
    let mut given_types = IndexMap::new();
    while !field_input.is_empty() {
        let name = field_input.parse::<Ident>()?;
        field_input.parse::<Token![:]>()?;
        let concrete_type = field_input.parse::<Type>()?;
        if name == MEMBRANE_POTENTIAL_QUANT {
            if membrane_potential_quant.is_some() {
                return Err(syn::Error::new(name.span(), format!("`{MEMBRANE_POTENTIAL_QUANT}` is set more than once")));
            }
            membrane_potential_quant = Some(concrete_type);
        } else if !quantizables.contains_key(&name) {
            return Err(syn::Error::new(name.span(), format!("`{name}` is not a quantizable of the model")));
        } else if given_types.contains_key(&name) {
            return Err(syn::Error::new(name.span(), format!("`{name}` is set more than once")));
        } else {
            given_types.insert(name, concrete_type);
        }
        if field_input.is_empty() {
            break;
        }
        field_input.parse::<Token![,]>()?;
    }

    let missing_error = |missing: &str| {
        syn::Error::new(brace.span.join(), format!("quantization level `{level_name}` does not set `{missing}`"))
    };
    let membrane_potential_quant = membrane_potential_quant.ok_or_else(|| missing_error(MEMBRANE_POTENTIAL_QUANT))?;
    let mut ordered_types = IndexMap::new();
    for name in quantizables.keys() {
        let concrete_type = given_types.swap_remove(name).ok_or_else(|| missing_error(&name.to_string()))?;
        ordered_types.insert(name.clone(), concrete_type);
    }

    Ok(QuantizationLevelTemplate {
        membrane_potential_quant,
        quantizables: ordered_types,
    })
}

/// `{ CorticalDataProperties: { ... } NeuronDataInternal: { ... } ... }`, in any order, the colon
/// after each data type and the comma between them being optional. Comes out in trait order
fn parse_data_specifications(input: ParseStream) -> syn::Result<IndexMap<Ident, IndexMap<Ident, DataFieldTemplate>>> {
    let specifications_input;
    braced!(specifications_input in input);
    let mut data_specifications = IndexMap::new();
    while !specifications_input.is_empty() {
        let data_struct = specifications_input.parse::<Ident>()?;
        if !MODEL_DATA_STRUCTS.iter().any(|(name, _)| data_struct == name) {
            let expected: Vec<&str> = MODEL_DATA_STRUCTS.iter().map(|(name, _)| *name).collect();
            return Err(syn::Error::new(
                data_struct.span(),
                format!("`{data_struct}` is not a data type of the model, expected one of: {}", expected.join(", ")),
            ));
        }
        if data_specifications.contains_key(&data_struct) {
            return Err(syn::Error::new(data_struct.span(), format!("`{data_struct}` is defined more than once")));
        }
        if specifications_input.peek(Token![:]) {
            specifications_input.parse::<Token![:]>()?;
        }
        let fields = parse_data_fields(&specifications_input)?;
        data_specifications.insert(data_struct, fields);
        parse_optional_comma(&specifications_input)?;
    }
    data_specifications.sort_by_cached_key(|data_struct, _| MODEL_DATA_STRUCTS.iter().position(|(name, _)| data_struct == name));
    Ok(data_specifications)
}

/// `{ #[attr] field_name: Type, ... }`
fn parse_data_fields(input: ParseStream) -> syn::Result<IndexMap<Ident, DataFieldTemplate>> {
    let field_input;
    braced!(field_input in input);
    let mut fields = IndexMap::new();
    while !field_input.is_empty() {
        let attributes = field_input.call(Attribute::parse_outer)?;
        let name = field_input.parse::<Ident>()?;
        if name == PHANTOM_FIELD {
            return Err(syn::Error::new(name.span(), format!("`{PHANTOM_FIELD}` is the marker field of every data struct, the field needs another name")));
        }
        field_input.parse::<Token![:]>()?;
        let field_type = field_input.parse::<Type>()?;
        if fields.contains_key(&name) {
            return Err(syn::Error::new(name.span(), format!("field `{name}` is defined more than once")));
        }
        fields.insert(name, DataFieldTemplate { attributes, field_type });
        if field_input.is_empty() {
            break;
        }
        field_input.parse::<Token![,]>()?;
    }
    Ok(fields)
}

//endregion
