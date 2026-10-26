use crate::common::parse_optional_comma;
use heck::ToSnakeCase;
use indexmap::IndexMap;
use proc_macro2::Ident;
use quote::{format_ident, quote, ToTokens};
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{braced, Attribute, Token, Type, TypeParamBound};

mod kw {
    syn::custom_keyword!(implementations);
}

/// Name of the implementation entry that sets `MembranePotentialQuantization::MembranePotentialQuant`
const MEMBRANE_POTENTIAL_QUANT: &str = "MembranePotentialQuant";

/// The data types of `CorticalDynamicsModel`, in trait order. Each can be given a struct
pub const MODEL_DATA_STRUCTS: [&str; 6] = [
    "CorticalDataProperties",
    "CorticalDataInternal",
    "CorticalDataScratch",
    "CorticalFunctionContext",
    "NeuronDataInternal",
    "NeuronDataScratch",
];

//region Cortical Dynamics Quantization Template

/// The quantization types of one cortical dynamics model. Formatted as the following:
/// ```text
/// /// Optional model description
/// ModelName: {
///     /// Optional description
///     T1: SomeTrait + OtherTrait,
/// }
///
/// implementations: {
///     Float16: {
///         MembranePotentialQuant: f16,
///         T1: f32,
///     }
/// }
///
/// CorticalDataProperties: {
///     /// Optional description
///     field_name: T1,
/// }
///
/// NeuronDataInternal: {
///     field_name: T1,
/// }
/// ```
/// Every implementation must set `MembranePotentialQuant` and each associated type of the model
/// once. After the implementations come optional, in any order, the data structs of the model,
/// one per data type of `CorticalDynamicsModel` (see [`MODEL_DATA_STRUCTS`]). Every field must be
/// typed by one of the model's associated types. Entries keep their template order, except data
/// structs, which keep trait order
pub struct CorticalDynamicsQuantizationTemplate {
    pub model_attributes: Vec<Attribute>,
    pub model_name: Ident,
    pub associated_types: IndexMap<Ident, AssociatedTypeTemplate>,
    pub implementations: IndexMap<Ident, QuantizationImplementationTemplate>,
    pub data_structs: IndexMap<Ident, IndexMap<Ident, ModelDataFieldTemplate>>,
}

/// An associated type of the model trait, with its bounds
pub struct AssociatedTypeTemplate {
    pub attributes: Vec<Attribute>,
    pub bounds: Punctuated<TypeParamBound, Token![+]>,
}

/// One quantization of the model, with the concrete type of each associated type
pub struct QuantizationImplementationTemplate {
    pub membrane_potential_quant: Type,
    pub associated_types: IndexMap<Ident, Type>,
}

/// A field of a model data struct, typed by an associated type of the model
pub struct ModelDataFieldTemplate {
    pub attributes: Vec<Attribute>,
    pub associated_type: Ident,
}

impl CorticalDynamicsQuantizationTemplate {
    pub fn model_trait_name(&self) -> Ident {
        format_ident!("CorticalDynamicsQuantization{}", self.model_name)
    }

    pub fn model_struct_name(&self) -> Ident {
        format_ident!("CorticalDynamicsModel{}", self.model_name)
    }

    pub fn implementation_struct_name(&self, implementation: &Ident) -> Ident {
        format_ident!("CorticalDynamicsQuantization{}{}", self.model_name, implementation)
    }

    /// `data_struct` is one of [`MODEL_DATA_STRUCTS`]
    pub fn data_struct_name(&self, data_struct: &Ident) -> Ident {
        format_ident!("{}{}", data_struct, self.model_name)
    }

    pub fn template_macro_name(&self) -> Ident {
        format_ident!("cortical_dynamics_quantization_{}_template", self.model_name.to_string().to_snake_case())
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

    /// Generate the model's quantization trait, a marker struct implementing it per
    /// implementation, and the model's data structs
    pub fn generate_rust_quantization(&self) -> proc_macro2::TokenStream {
        let model_attributes = &self.model_attributes;
        let model_trait = self.model_trait_name();
        let model_struct = self.model_struct_name();

        let associated_types = self.associated_types.iter().map(|(name, associated)| {
            let attributes = &associated.attributes;
            let bounds = &associated.bounds;
            quote! {
                #( #attributes )*
                type #name: #bounds;
            }
        });

        let implementations = self.implementations.iter().map(|(name, implementation)| {
            let struct_name = self.implementation_struct_name(name);
            let membrane_potential_quant = &implementation.membrane_potential_quant;
            let associated_names = implementation.associated_types.keys();
            let associated_types = implementation.associated_types.values();
            quote! {
                #[derive(Clone, Copy, PartialEq, Debug, ::serde::Serialize, ::serde::Deserialize)]
                pub struct #struct_name;

                unsafe impl ::core::marker::Sync for #struct_name {}

                unsafe impl ::core::marker::Send for #struct_name {}

                impl MembranePotentialQuantization for #struct_name {
                    type MembranePotentialQuant = #membrane_potential_quant;
                }

                impl CorticalDynamicsQuantization for #struct_name {
                    type Model = #model_struct<Self>;
                }

                impl #model_trait for #struct_name {
                    #( type #associated_names = #associated_types; )*
                }
            }
        });

        let data_structs = self.data_structs.iter().map(|(data_struct, fields)| {
            let struct_name = self.data_struct_name(data_struct);
            // A struct without fields holds the quantization as a marker, as it must use `CDQ`
            if fields.is_empty() {
                return quote! {
                    pub struct #struct_name<CDQ: #model_trait>(::core::marker::PhantomData<CDQ>);
                };
            }
            let fields = fields.iter().map(|(name, field)| {
                let attributes = &field.attributes;
                let associated_type = &field.associated_type;
                quote! {
                    #( #attributes )*
                    pub #name: CDQ::#associated_type,
                }
            });
            quote! {
                pub struct #struct_name<CDQ: #model_trait> {
                    #( #fields )*
                }
            }
        });

        quote! {
            #( #model_attributes )*
            pub trait #model_trait: CorticalDynamicsQuantization {
                #( #associated_types )*
            }

            #( #implementations )*

            #( #data_structs )*
        }
    }
}

impl Parse for CorticalDynamicsQuantizationTemplate {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let model_attributes = input.call(Attribute::parse_outer)?;
        let model_name = input.parse::<Ident>()?;
        input.parse::<Token![:]>()?;
        let associated_types = parse_associated_types(input)?;
        parse_optional_comma(input)?;

        input.parse::<kw::implementations>()?;
        input.parse::<Token![:]>()?;
        let implementations = parse_implementations(input, &associated_types)?;
        parse_optional_comma(input)?;

        let mut data_structs = IndexMap::new();
        while !input.is_empty() {
            let data_struct = input.parse::<Ident>()?;
            if !MODEL_DATA_STRUCTS.iter().any(|name| data_struct == name) {
                return Err(syn::Error::new(
                    data_struct.span(),
                    format!("`{data_struct}` is not a data type of the model, expected one of: {}", MODEL_DATA_STRUCTS.join(", ")),
                ));
            }
            if data_structs.contains_key(&data_struct) {
                return Err(syn::Error::new(data_struct.span(), format!("`{data_struct}` is defined more than once")));
            }
            input.parse::<Token![:]>()?;
            let fields = parse_data_struct_fields(input, &associated_types)?;
            data_structs.insert(data_struct, fields);
            parse_optional_comma(input)?;
        }
        data_structs.sort_by_cached_key(|data_struct, _| MODEL_DATA_STRUCTS.iter().position(|name| data_struct == name));

        Ok(Self {
            model_attributes,
            model_name,
            associated_types,
            implementations,
            data_structs,
        })
    }
}

/// Writes the template back out in the format it is parsed from, with implementations setting
/// `MembranePotentialQuant` first and then the associated types in model order
impl ToTokens for CorticalDynamicsQuantizationTemplate {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let model_attributes = &self.model_attributes;
        let model_name = &self.model_name;

        let associated_types = self.associated_types.iter().map(|(name, associated)| {
            let attributes = &associated.attributes;
            let bounds = &associated.bounds;
            quote! {
                #( #attributes )*
                #name: #bounds,
            }
        });

        let implementations = self.implementations.iter().map(|(name, implementation)| {
            let membrane_potential_quant = &implementation.membrane_potential_quant;
            let associated_names = implementation.associated_types.keys();
            let associated_types = implementation.associated_types.values();
            quote! {
                #name: {
                    MembranePotentialQuant: #membrane_potential_quant,
                    #( #associated_names: #associated_types, )*
                }
            }
        });

        let data_structs = self.data_structs.iter().map(|(data_struct, fields)| {
            let fields = fields.iter().map(|(name, field)| {
                let attributes = &field.attributes;
                let associated_type = &field.associated_type;
                quote! {
                    #( #attributes )*
                    #name: #associated_type,
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
            #model_name: {
                #( #associated_types )*
            }

            implementations: {
                #( #implementations )*
            }

            #( #data_structs )*
        });
    }
}

/// `{ #[attr] T1: Bound + Bound, ... }`
fn parse_associated_types(input: ParseStream) -> syn::Result<IndexMap<Ident, AssociatedTypeTemplate>> {
    let field_input;
    braced!(field_input in input);
    let mut associated_types = IndexMap::new();
    while !field_input.is_empty() {
        let attributes = field_input.call(Attribute::parse_outer)?;
        let name = field_input.parse::<Ident>()?;
        if name == MEMBRANE_POTENTIAL_QUANT {
            return Err(syn::Error::new(name.span(), format!("`{MEMBRANE_POTENTIAL_QUANT}` is set by every implementation, it cannot be an associated type")));
        }
        field_input.parse::<Token![:]>()?;
        let bounds = Punctuated::<TypeParamBound, Token![+]>::parse_separated_nonempty(&field_input)?;
        if associated_types.contains_key(&name) {
            return Err(syn::Error::new(name.span(), format!("associated type `{name}` is defined more than once")));
        }
        associated_types.insert(name, AssociatedTypeTemplate { attributes, bounds });
        if field_input.is_empty() {
            break;
        }
        field_input.parse::<Token![,]>()?;
    }
    Ok(associated_types)
}

/// `{ Name: { MembranePotentialQuant: Type, T1: Type, ... } ... }`, the comma between
/// implementations being optional
fn parse_implementations(
    input: ParseStream,
    associated_types: &IndexMap<Ident, AssociatedTypeTemplate>,
) -> syn::Result<IndexMap<Ident, QuantizationImplementationTemplate>> {
    let implementations_input;
    braced!(implementations_input in input);
    let mut implementations = IndexMap::new();
    while !implementations_input.is_empty() {
        let implementation_name = implementations_input.parse::<Ident>()?;
        implementations_input.parse::<Token![:]>()?;
        let implementation = parse_implementation(&implementations_input, &implementation_name, associated_types)?;
        if implementations.contains_key(&implementation_name) {
            return Err(syn::Error::new(implementation_name.span(), format!("implementation `{implementation_name}` is defined more than once")));
        }
        implementations.insert(implementation_name, implementation);
        parse_optional_comma(&implementations_input)?;
    }
    Ok(implementations)
}

/// `{ MembranePotentialQuant: Type, T1: Type, ... }`, in any order. Concrete types come out in
/// the model's associated type order
fn parse_implementation(
    input: ParseStream,
    implementation_name: &Ident,
    associated_types: &IndexMap<Ident, AssociatedTypeTemplate>,
) -> syn::Result<QuantizationImplementationTemplate> {
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
        } else if !associated_types.contains_key(&name) {
            return Err(syn::Error::new(name.span(), format!("`{name}` is not an associated type of the model")));
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
        syn::Error::new(brace.span.join(), format!("implementation `{implementation_name}` does not set `{missing}`"))
    };
    let membrane_potential_quant = membrane_potential_quant.ok_or_else(|| missing_error(MEMBRANE_POTENTIAL_QUANT))?;
    let mut ordered_types = IndexMap::new();
    for name in associated_types.keys() {
        let concrete_type = given_types.swap_remove(name).ok_or_else(|| missing_error(&name.to_string()))?;
        ordered_types.insert(name.clone(), concrete_type);
    }

    Ok(QuantizationImplementationTemplate {
        membrane_potential_quant,
        associated_types: ordered_types,
    })
}

/// `{ #[attr] field_name: T1, ... }`
fn parse_data_struct_fields(
    input: ParseStream,
    associated_types: &IndexMap<Ident, AssociatedTypeTemplate>,
) -> syn::Result<IndexMap<Ident, ModelDataFieldTemplate>> {
    let field_input;
    braced!(field_input in input);
    let mut fields = IndexMap::new();
    while !field_input.is_empty() {
        let attributes = field_input.call(Attribute::parse_outer)?;
        let name = field_input.parse::<Ident>()?;
        field_input.parse::<Token![:]>()?;
        let associated_type = field_input.parse::<Ident>()?;
        if !associated_types.contains_key(&associated_type) {
            return Err(syn::Error::new(associated_type.span(), format!("`{associated_type}` is not an associated type of the model")));
        }
        if fields.contains_key(&name) {
            return Err(syn::Error::new(name.span(), format!("field `{name}` is defined more than once")));
        }
        fields.insert(name, ModelDataFieldTemplate { attributes, associated_type });
        if field_input.is_empty() {
            break;
        }
        field_input.parse::<Token![,]>()?;
    }
    Ok(fields)
}

//endregion

//region Generated Code Paths

