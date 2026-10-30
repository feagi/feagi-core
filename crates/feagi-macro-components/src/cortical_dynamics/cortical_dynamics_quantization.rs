use crate::common::{basis_path, parse_optional_comma};
use heck::ToSnakeCase;
use indexmap::IndexMap;
use proc_macro2::Ident;
use quote::{format_ident, quote, ToTokens};
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{braced, Attribute, Token, Type, TypeParamBound};

mod kw {
    syn::custom_keyword!(NeuronLayout);
    syn::custom_keyword!(Quantizables);
    syn::custom_keyword!(QuantizationLevels);
    syn::custom_keyword!(DataSpecifications);
}

/// Name of the quantization level entry that sets `MembranePotentialQuantization::MembranePotentialQuant`
const MEMBRANE_POTENTIAL_QUANT: &str = "MembranePotentialQuant";

/// Name of the neuron model generic of the generated data structs
const NEURON_MODEL_GENERIC: &str = "NM";

/// Name of the marker field every generated data struct holds
const PHANTOM_FIELD: &str = "_p";

/// The data types of `NeuronModel` in trait order, with the specification trait each of their
/// structs implements
pub const MODEL_DATA_STRUCTS: [(&str, &str); 6] = [
    ("CorticalDataProperties", "CorticalDataPropertiesSpecification"),
    ("CorticalDataInternal", "CorticalDataInternalSpecification"),
    ("CorticalDataScratch", "CorticalDataScratchSpecification"),
    ("CorticalFunctionContext", "CorticalFunctionSpecification"),
    ("NeuronDataInternal", "NeuronDataInternalSpecification"),
    ("NeuronDataScratch", "NeuronDataScratchSpecification"),
];

//region Cortical Dynamics Quantization Template

/// The quantization types of one cortical dynamics model. Formatted as the following:
/// ```text
/// /// Optional model description
/// ModelName
///
/// NeuronLayout: NeuronLayoutVoxelDefaultType
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
/// The optional `NeuronLayout` entry sets the `NeuronLayoutType` the model is implemented for (see
/// [`ModelLayoutTemplate`]), defaulting to any, `NL: NeuronLayoutType`. The optional
/// `Quantizables` each become a wrapper struct of their kind (see [`QuantizableKind`]) and an
/// associated type of the model. Every quantization level must set `MembranePotentialQuant` and
/// each quantizable once. The optional `DataSpecifications` hold, in any order, the fields of the
/// model's data structs, one per data type of `NeuronModel` (see
/// [`MODEL_DATA_STRUCTS`]). A field typed by a quantizable is its wrapper at the level's
/// quantization, and `MembranePotentialQuant` the level's membrane potential type. Any other type
/// is kept as written. Entries keep their template order, except data structs, which keep trait
/// order
pub struct CorticalDynamicsQuantizationTemplate {
    pub model_attributes: Vec<Attribute>,
    pub model_name: Ident,
    pub layout: ModelLayoutTemplate,
    pub quantizables: IndexMap<Ident, QuantizableTemplate>,
    pub quantization_levels: IndexMap<Ident, QuantizationLevelTemplate>,
    pub data_specifications: IndexMap<Ident, IndexMap<Ident, DataFieldTemplate>>,
}

/// The `NeuronLayoutType` a model is implemented for, which decides the generics of its
/// quantization level structs `NeuronModel{Model}{Level}`
pub enum ModelLayoutTemplate {
    /// `NeuronLayout: NeuronLayoutVoxelDefaultType`, a single layout type, which the model trait
    /// `NeuronModel{Model}` fixes. The level structs are `NeuronModel{Model}{Level}`
    Fixed(Type),
    /// `NeuronLayout: NL: NeuronLayoutType`, any layout type within the bounds, which the model
    /// trait `NeuronModel{Model}` restricts to. The level structs are `NeuronModel{Model}{Level}<NL>`
    Generic {
        name: Ident,
        bounds: Punctuated<TypeParamBound, Token![+]>,
    },
}

impl Default for ModelLayoutTemplate {
    /// Any layout type, `NL: NeuronLayoutType`
    fn default() -> Self {
        Self::Generic {
            name: format_ident!("NL"),
            bounds: syn::parse_quote!(NeuronLayoutType),
        }
    }
}

impl Parse for ModelLayoutTemplate {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        // `Name: Bounds` is a generic, while anything else (`Name<..>`, `path::Name`) is a type
        if input.peek(syn::Ident) && input.peek2(Token![:]) && !input.peek2(Token![::]) {
            let name = input.parse::<Ident>()?;
            if name == NEURON_MODEL_GENERIC {
                return Err(syn::Error::new(name.span(), format!("`{name}` is already the generic of the model data structs, the layout generic needs another name")));
            }
            input.parse::<Token![:]>()?;
            let bounds = Punctuated::<TypeParamBound, Token![+]>::parse_separated_nonempty(input)?;
            Ok(Self::Generic { name, bounds })
        } else {
            Ok(Self::Fixed(input.parse::<Type>()?))
        }
    }
}

impl ToTokens for ModelLayoutTemplate {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            Self::Fixed(layout) => layout.to_tokens(tokens),
            Self::Generic { name, bounds } => tokens.extend(quote! { #name: #bounds }),
        }
    }
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
    pub fn model_trait_name(&self) -> Ident {
        format_ident!("NeuronModel{}", self.model_name)
    }

    pub fn quantization_level_struct_name(&self, level: &Ident) -> Ident {
        format_ident!("NeuronModel{}{}", self.model_name, level)
    }

    /// `data_struct` is one of [`MODEL_DATA_STRUCTS`]
    pub fn data_struct_name(&self, data_struct: &Ident) -> Ident {
        format_ident!("{}{}", data_struct, self.model_name)
    }

    pub fn template_macro_name(&self) -> Ident {
        format_ident!("cortical_dynamics_quantization_{}_template", self.model_name.to_string().to_snake_case())
    }

    /// The type a data field is generated with. A quantizable is its wrapper at the level's
    /// quantization, `Name<NM::Name>`, and `MembranePotentialQuant` is
    /// `NM::MembranePotentialQuant`. Anything else is kept as written
    fn resolve_field_type(&self, field_type: &Type) -> proc_macro2::TokenStream {
        let nm = format_ident!("{}", NEURON_MODEL_GENERIC);
        if let Type::Path(path) = field_type {
            if path.qself.is_none() {
                if let Some(name) = path.path.get_ident() {
                    if self.quantizables.contains_key(name) {
                        return quote!(#name<#nm::#name>);
                    }
                    if name == MEMBRANE_POTENTIAL_QUANT {
                        return quote!(#nm::#name);
                    }
                }
            }
        }
        field_type.to_token_stream()
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
    /// quantization level, and every data struct of the model
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

        // The layout type restriction of the model trait, and the generics of the level structs
        // with the layout type they set. A fixed layout type is set by every level, a generic one
        // is a generic of every level
        let (layout_restriction, level_generics, level_struct_body, layout_definition) = match &self.layout {
            ModelLayoutTemplate::Fixed(layout) => (
                quote!(LayoutType = #layout),
                quote!(),
                quote!(;),
                quote!(#layout),
            ),
            ModelLayoutTemplate::Generic { name, bounds } => (
                quote!(LayoutType: #bounds),
                quote!(<#name: #bounds>),
                quote!((::core::marker::PhantomData<#name>);),
                quote!(#name),
            ),
        };
        let level_generic_names = match &self.layout {
            ModelLayoutTemplate::Fixed(_) => quote!(),
            ModelLayoutTemplate::Generic { name, .. } => quote!(<#name>),
        };

        // Every data type of `NeuronModel` is set to the model's data struct of that type
        let model_data_names: Vec<Ident> = MODEL_DATA_STRUCTS.iter().map(|(name, _)| format_ident!("{}", name)).collect();
        let model_data_structs: Vec<Ident> = model_data_names.iter().map(|name| self.data_struct_name(name)).collect();

        let quantization_levels = self.quantization_levels.iter().map(|(name, level)| {
            let struct_name = self.quantization_level_struct_name(name);
            let membrane_potential_quant = &level.membrane_potential_quant;
            let quantizable_names = level.quantizables.keys();
            let quantizable_types = level.quantizables.values();
            let model_data_names = &model_data_names;
            let model_data_structs = &model_data_structs;
            let level_generics = &level_generics;
            let level_generic_names = &level_generic_names;
            let level_struct_body = &level_struct_body;
            let layout_definition = &layout_definition;
            quote! {
                #[derive(Clone, Copy, PartialEq, Debug, ::serde::Serialize, ::serde::Deserialize)]
                #[serde(bound = "")] // A generic layout type is only ever held as PhantomData
                pub struct #struct_name #level_generics #level_struct_body

                impl #level_generics MembranePotentialQuantization for #struct_name #level_generic_names {
                    type MembranePotentialQuant = #membrane_potential_quant;
                }

                impl #level_generics NeuronModel for #struct_name #level_generic_names {
                    type LayoutType = #layout_definition;
                    #( type #model_data_names = #model_data_structs<Self>; )*
                }

                impl #level_generics #model_trait for #struct_name #level_generic_names {
                    #( type #quantizable_names = #quantizable_types; )*
                }
            }
        });

        let nm = format_ident!("{}", NEURON_MODEL_GENERIC);
        let no_fields = IndexMap::new();
        let data_structs = MODEL_DATA_STRUCTS.iter().map(|(data_struct, specification)| {
            let data_struct = format_ident!("{}", data_struct);
            let specification = format_ident!("{}", specification);
            let struct_name = self.data_struct_name(&data_struct);
            let fields = self.data_specifications.get(&data_struct).unwrap_or(&no_fields);
            let field_attributes: Vec<&Vec<Attribute>> = fields.values().map(|field| &field.attributes).collect();
            let field_names: Vec<&Ident> = fields.keys().collect();
            let field_types: Vec<proc_macro2::TokenStream> = fields.values().map(|field| self.resolve_field_type(&field.field_type)).collect();
            quote! {
                #[derive(Clone, Debug, PartialEq, ::serde::Serialize, ::serde::Deserialize)]
                pub struct #struct_name<#nm: #model_trait> {
                    #(
                        #( #field_attributes )*
                        pub #field_names: #field_types,
                    )*
                    #[serde(skip)]
                    _p: ::core::marker::PhantomData<#nm>,
                }

                impl<#nm: #model_trait> #struct_name<#nm> {
                    pub fn new(#( #field_names: #field_types ),*) -> Self {
                        Self {
                            #( #field_names, )*
                            _p: ::core::marker::PhantomData,
                        }
                    }
                }

                impl<#nm: #model_trait> #specification<#nm> for #struct_name<#nm> {}
            }
        });

        quote! {
            #( #wrappers )*

            #( #model_attributes )*
            pub trait #model_trait: NeuronModel<#layout_restriction> {
                #( #associated_types )*
            }

            #( #quantization_levels )*

            #( #data_structs )*
        }
    }
}

impl Parse for CorticalDynamicsQuantizationTemplate {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let model_attributes = input.call(Attribute::parse_outer)?;
        let model_name = input.parse::<Ident>()?;
        parse_optional_comma(input)?;

        let layout = if input.peek(kw::NeuronLayout) {
            input.parse::<kw::NeuronLayout>()?;
            input.parse::<Token![:]>()?;
            let layout = input.parse::<ModelLayoutTemplate>()?;
            parse_optional_comma(input)?;
            layout
        } else {
            ModelLayoutTemplate::default()
        };

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
    braced!(levels_input in input);
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
