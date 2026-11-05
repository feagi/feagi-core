use super::model_family_template::{manual_enum_impls, serde_bounds, serde_derive};
use crate::common::{basis_path, parse_optional_comma};
use heck::ToSnakeCase;
use indexmap::IndexMap;
use proc_macro2::{Ident, Literal};
use quote::{format_ident, quote, ToTokens};
use syn::parse::{Parse, ParseStream};
use syn::{braced, bracketed, Attribute, LitInt, LitStr, Token};

mod kw {
    syn::custom_keyword!(Family);
    syn::custom_keyword!(Error);
    syn::custom_keyword!(ModelFamily);
    syn::custom_keyword!(ModelFamilyError);
    syn::custom_keyword!(Identifier);
    syn::custom_keyword!(IdentifierError);
    syn::custom_keyword!(Types);
}

//region Cortical Type Family Template

/// A family of cortical area types, each belonging to exactly one super type and allowing one or
/// more models of a model family. Formatted as the following:
/// ```text
/// /// Optional family description
/// Family: CorticalAreaType,
/// Error: CorticalAreaTypeSelectionError,
/// ModelFamily: NeuronModel,
/// ModelFamilyError: NeuronModelSelectionError,
/// Identifier: CorticalIdentifier,
/// IdentifierError: CorticalIdentifierParseError,
///
/// Types: {
///     /// Optional super type description
///     Foundation = 1: {
///         /// Optional cortical type description
///         Power = 1: [FoundationPower],
///     }
///     Processing = 2: {
///         Interconnect = 1: [FeagiAdvanced, FeagiSimple],
///     }
/// }
/// ```
/// The number of a super type or cortical type is its key, unique among its siblings, from 1 to 255.
/// Each cortical type lists the models of the model family it allows. No paths are given. Expanding
/// this generates the family module, named the snake case of the family, which re-exports everything
/// public where it is expanded. That must include the base trait `{Family}`, with associated type
/// `NeuronLayout`, and a public module per super type named its snake case, holding a unit struct per
/// cortical type named as in the template, such as `foundation::Power`, implementing the base trait.
/// The module of the model family must also be in scope where this is expanded, as must
/// `CorticalAreaInit` and `FeagiIndexQuantization`. Every map keeps its template order
pub struct CorticalTypeFamilyTemplate {
    pub family_attributes: Vec<Attribute>,
    pub family_name: Ident,
    pub error_name: Ident,
    pub model_family_name: Ident,
    pub model_family_error_name: Ident,
    pub identifier_name: Ident,
    pub identifier_error_name: Ident,
    pub super_types: IndexMap<Ident, SuperTypeTemplate>,
}

/// A super type of the family, which only holds cortical types
pub struct SuperTypeTemplate {
    pub attributes: Vec<Attribute>,
    pub key: u8,
    pub cortical_types: IndexMap<Ident, CorticalTypeTemplate>,
}

/// A cortical type of a super type, with the models of the model family it allows
pub struct CorticalTypeTemplate {
    pub attributes: Vec<Attribute>,
    pub key: u8,
    pub models: Vec<Ident>,
}

/// A cortical type, by the names of its super type and itself
pub(super) struct CorticalTypeLeaf<'a> {
    pub(super) super_type: &'a Ident,
    pub(super) name: &'a Ident,
    pub(super) template: &'a CorticalTypeTemplate,
}

impl CorticalTypeFamilyTemplate {
    //region Names

    fn family_snake_case(&self) -> String {
        self.family_name.to_string().to_snake_case()
    }

    pub fn template_macro_name(&self) -> Ident {
        format_ident!("{}_family_template", self.family_snake_case())
    }

    /// The module of this family, generated where this is expanded
    pub fn family_module(&self) -> Ident {
        format_ident!("{}", self.family_snake_case())
    }

    /// The module of the model family, expected in scope wherever generated code is expanded
    pub fn model_family_module(&self) -> Ident {
        format_ident!("{}", self.model_family_name.to_string().to_snake_case())
    }

    pub(super) fn family_kind_name(&self) -> Ident {
        format_ident!("{}Kind", self.family_name)
    }

    fn family_enum_name(&self) -> Ident {
        format_ident!("{}Enum", self.family_name)
    }

    pub(super) fn super_type_kind_name(super_type: &Ident) -> Ident {
        format_ident!("{}SuperTypeKind", super_type)
    }

    fn super_type_enum_name(super_type: &Ident) -> Ident {
        format_ident!("{}SuperTypeEnum", super_type)
    }

    /// The enum of the models a cortical type allows
    fn model_enum_name(super_type: &Ident, cortical_type: &Ident) -> Ident {
        format_ident!("{}{}ModelEnum", super_type, cortical_type)
    }

    pub fn member_trait_name(&self) -> Ident {
        format_ident!("{}Member", self.family_name)
    }

    /// Implemented per cortical type and allowed model, generic over the members of that model. The
    /// tag argument is what keeps those impls from overlapping
    fn member_via_trait_name(&self) -> Ident {
        format_ident!("{}MemberVia", self.family_name)
    }

    /// Implemented for a cortical type at each member of each model it allows
    fn member_at_trait_name(&self) -> Ident {
        format_ident!("{}MemberAt", self.family_name)
    }

    fn uses_trait_name(model: &Ident) -> Ident {
        format_ident!("Uses{}", model)
    }

    pub fn with_macro_name(&self) -> Ident {
        format_ident!("with_{}", self.family_snake_case())
    }

    /// The method of every data enum returning the model family kind held
    fn model_kind_method(&self) -> Ident {
        format_ident!("{}_kind", self.model_family_name.to_string().to_snake_case())
    }

    /// The marker struct of a cortical type, through the family module
    pub(super) fn cortical_type_path(&self, super_type: &Ident, cortical_type: &Ident) -> proc_macro2::TokenStream {
        let family_module = self.family_module();
        let super_type_module = format_ident!("{}", super_type.to_string().to_snake_case());
        quote!(#family_module::#super_type_module::#cortical_type)
    }

    pub(super) fn base_trait_path(&self) -> proc_macro2::TokenStream {
        let family_module = self.family_module();
        let family_name = &self.family_name;
        quote!(#family_module::#family_name)
    }

    fn model_container_path(&self) -> proc_macro2::TokenStream {
        let module = self.model_family_module();
        let container = format_ident!("{}Container", self.model_family_name);
        quote!(#module::#container)
    }

    fn model_member_path(&self) -> proc_macro2::TokenStream {
        let module = self.model_family_module();
        let member = format_ident!("{}Member", self.model_family_name);
        quote!(#module::#member)
    }

    fn model_model_member_path(&self, model: &Ident) -> proc_macro2::TokenStream {
        let module = self.model_family_module();
        let member = format_ident!("{}{}Member", self.model_family_name, model);
        quote!(#module::#member)
    }

    fn model_module_path(&self, model: &Ident) -> proc_macro2::TokenStream {
        let module = self.model_family_module();
        let model_module = format_ident!("{}", model.to_string().to_snake_case());
        quote!(#module::#model_module)
    }

    fn model_trait_path(&self, model: &Ident) -> proc_macro2::TokenStream {
        let model_module = self.model_module_path(model);
        let model_trait = format_ident!("{}{}", self.model_family_name, model);
        quote!(#model_module::#model_trait)
    }

    fn model_tag_path(&self, model: &Ident) -> proc_macro2::TokenStream {
        let model_module = self.model_module_path(model);
        let tag = format_ident!("{}{}Tag", self.model_family_name, model);
        quote!(#model_module::#tag)
    }

    fn model_tag_trait_path(&self) -> proc_macro2::TokenStream {
        let module = self.model_family_module();
        let tag_trait = format_ident!("{}Tag", self.model_family_name);
        quote!(#module::#tag_trait)
    }

    fn model_quant_enum_path(&self, model: &Ident) -> proc_macro2::TokenStream {
        let module = self.model_family_module();
        let quant_enum = format_ident!("{}QuantEnum", model);
        quote!(#module::#quant_enum)
    }

    fn model_kind_path(&self) -> proc_macro2::TokenStream {
        let module = self.model_family_module();
        let kind = format_ident!("{}Kind", self.model_family_name);
        quote!(#module::#kind)
    }

    fn model_error_path(&self) -> proc_macro2::TokenStream {
        let module = self.model_family_module();
        let error = &self.model_family_error_name;
        quote!(#module::#error)
    }

    //endregion

    /// Every cortical type of the family, in template order
    pub(super) fn cortical_types(&self) -> Vec<CorticalTypeLeaf<'_>> {
        self.super_types
            .iter()
            .flat_map(|(super_type, template)| {
                template.cortical_types.iter().map(move |(name, template)| CorticalTypeLeaf { super_type, name, template })
            })
            .collect()
    }

    /// Every model allowed by any cortical type, once each, in template order
    fn used_models(&self) -> Vec<&Ident> {
        let mut used: Vec<&Ident> = Vec::new();
        for leaf in self.cortical_types() {
            for model in &leaf.template.models {
                if !used.contains(&model) {
                    used.push(model);
                }
            }
        }
        used
    }

    /// The kind of a cortical type, as a value of the kind of the family
    pub(super) fn kind_value(&self, super_type: &Ident, cortical_type: &Ident) -> proc_macro2::TokenStream {
        let family_kind = self.family_kind_name();
        let super_type_kind = Self::super_type_kind_name(super_type);
        quote!(#family_kind::#super_type(#super_type_kind::#cortical_type))
    }

    /// The variant of a cortical type at a model, as a value or pattern of the data enum of the
    /// family, wrapping `inner`. Each enum is prefixed by `qualifier`
    fn enum_value(
        &self,
        super_type: &Ident,
        cortical_type: &Ident,
        model: &Ident,
        inner: &proc_macro2::TokenStream,
        qualifier: &proc_macro2::TokenStream,
    ) -> proc_macro2::TokenStream {
        let model_enum = Self::model_enum_name(super_type, cortical_type);
        let wrapped = quote!(#qualifier #model_enum::#model(#inner));
        self.enum_value_of_type(super_type, cortical_type, &wrapped, qualifier)
    }

    /// The variant of a cortical type, as a value or pattern of the data enum of the family,
    /// wrapping `inner`, which is held for the models of the cortical type
    fn enum_value_of_type(
        &self,
        super_type: &Ident,
        cortical_type: &Ident,
        inner: &proc_macro2::TokenStream,
        qualifier: &proc_macro2::TokenStream,
    ) -> proc_macro2::TokenStream {
        let family_enum = self.family_enum_name();
        let super_type_enum = Self::super_type_enum_name(super_type);
        quote!(#qualifier #family_enum::#super_type(#qualifier #super_type_enum::#cortical_type(#inner)))
    }

    /// Generate the template macro, which holds this template once validated. Its single input is
    /// the path of a macro, which it calls with the template, as in
    /// `{family}_family_template!(expand_cortical_type_family)`. Exported, so other crates may use
    /// it
    pub fn generate_rust_template_macro(&self) -> proc_macro2::TokenStream {
        let macro_name = self.template_macro_name();
        let doc = LitStr::new(
            &format!("The validated cortical type family template of the `{}` family. Calls the given macro with it", self.family_name),
            proc_macro2::Span::call_site(),
        );
        quote! {
            #[doc = #doc]
            #[macro_export]
            macro_rules! #macro_name {
                ($($callback:tt)+) => {
                    $($callback)+! {
                        #self
                    }
                };
            }
        }
    }

    /// Generate the family module, kinds, error, data enums, uses traits, member traits, and
    /// dispatch macro of the family
    pub fn generate_rust_family(&self) -> proc_macro2::TokenStream {
        let family_module = self.generate_family_module();
        let error = self.generate_error();
        let kinds = self.generate_kinds();
        let enums = self.generate_enums();
        let uses = self.generate_uses_traits();
        let member = self.generate_member_traits();
        let with_macro = self.generate_with_macro();
        quote! {
            #family_module
            #error
            #kinds
            #enums
            #uses
            #member
            #with_macro
        }
    }

    //region Family Module

    /// The module named the snake case of the family, re-exporting everything public where it is
    /// expanded
    fn generate_family_module(&self) -> proc_macro2::TokenStream {
        let family_module = self.family_module();
        let family_attributes = &self.family_attributes;
        quote! {
            #( #family_attributes )*
            pub mod #family_module {
                pub use super::*;
            }
        }
    }

    //endregion

    //region Error

    fn generate_error(&self) -> proc_macro2::TokenStream {
        let family_name = &self.family_name;
        let error = &self.error_name;
        let kind = self.family_kind_name();
        let model_error = self.model_error_path();
        let model_variant = &self.model_family_name;
        let doc = format!("Data of the `{family_name}` family was accessed as a cortical type or super type it does not hold");
        let incorrect_type = LitStr::new(
            &format!("Tried to access the {family_name} `{{}}` when it was actually `{{}}`"),
            proc_macro2::Span::call_site(),
        );
        let incorrect_super_type = LitStr::new(
            &format!("Tried to access a {family_name} of super type `{{}}` when it was actually `{{}}`"),
            proc_macro2::Span::call_site(),
        );
        quote! {
            #[doc = #doc]
            #[derive(Clone, Copy, PartialEq, Eq, Debug)]
            pub enum #error {
                /// Accessed as a cortical type other than the one held
                IncorrectCorticalType { expected: #kind, actual: #kind },
                /// Accessed as a super type other than the one held
                IncorrectSuperType { expected: &'static str, actual: &'static str },
                /// The cortical type held is at a different member of the model family
                #model_variant(#model_error),
            }

            impl ::core::convert::From<#model_error> for #error {
                fn from(value: #model_error) -> Self {
                    Self::#model_variant(value)
                }
            }

            impl ::core::fmt::Display for #error {
                fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                    match self {
                        Self::IncorrectCorticalType { expected, actual } => write!(f, #incorrect_type, expected, actual),
                        Self::IncorrectSuperType { expected, actual } => write!(f, #incorrect_super_type, expected, actual),
                        Self::#model_variant(inner) => ::core::fmt::Display::fmt(inner, f),
                    }
                }
            }

            impl ::core::error::Error for #error {
                fn source(&self) -> ::core::option::Option<&(dyn ::core::error::Error + 'static)> {
                    match self {
                        Self::#model_variant(inner) => ::core::option::Option::Some(inner),
                        _ => ::core::option::Option::None,
                    }
                }
            }
        }
    }

    //endregion

    //region Kinds

    /// A data less `{SuperType}SuperTypeKind` per super type, with a variant per cortical type, and
    /// `{Family}Kind`, with a variant per super type holding its kind
    fn generate_kinds(&self) -> proc_macro2::TokenStream {
        let family_name = &self.family_name;
        let family_kind = self.family_kind_name();
        let serde_derive = serde_derive();

        let super_type_kinds = self.super_types.iter().map(|(super_type, template)| {
            let kind = Self::super_type_kind_name(super_type);
            let doc = format!("Every cortical type of the `{super_type}` super type of the `{family_name}` family");
            let names: Vec<&Ident> = template.cortical_types.keys().collect();
            let name_strings = names.iter().map(|name| name.to_string());
            let attributes = template.cortical_types.values().map(|cortical_type| &cortical_type.attributes);
            let keys = template.cortical_types.values().map(|cortical_type| cortical_type.key);
            let count = names.len();
            quote! {
                #[doc = #doc]
                #[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
                #serde_derive
                pub enum #kind {
                    #(
                        #( #attributes )*
                        #names,
                    )*
                }

                impl #kind {
                    /// Every cortical type of the super type, in template order
                    pub const ALL: [Self; #count] = [#( Self::#names ),*];

                    /// The name of the cortical type
                    pub const fn name(&self) -> &'static str {
                        match self {
                            #( Self::#names => #name_strings, )*
                        }
                    }

                    /// The key of the cortical type, unique within the super type
                    pub const fn key(&self) -> u8 {
                        match self {
                            #( Self::#names => #keys, )*
                        }
                    }
                }

                impl ::core::fmt::Display for #kind {
                    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                        f.write_str(self.name())
                    }
                }
            }
        });

        let names: Vec<&Ident> = self.super_types.keys().collect();
        let name_strings: Vec<String> = names.iter().map(|name| name.to_string()).collect();
        let attributes = self.super_types.values().map(|super_type| &super_type.attributes);
        let held = names.iter().map(|name| Self::super_type_kind_name(name));
        let keys = self.super_types.values().map(|super_type| super_type.key);
        let leaves = self.cortical_types();
        let count = leaves.len();
        let all_kinds = leaves.iter().map(|leaf| self.kind_value(leaf.super_type, leaf.name));
        let doc = format!("Every cortical type of the `{family_name}` family, as a super type and a cortical type of it");

        quote! {
            #( #super_type_kinds )*

            #[doc = #doc]
            #[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
            #serde_derive
            pub enum #family_kind {
                #(
                    #( #attributes )*
                    #names(#held),
                )*
            }

            impl #family_kind {
                /// Every cortical type of the family, in template order
                pub const ALL: [Self; #count] = [#( #all_kinds ),*];

                /// The name of the super type
                pub const fn super_type_name(&self) -> &'static str {
                    match self {
                        #( Self::#names(_) => #name_strings, )*
                    }
                }

                /// The name of the cortical type
                pub const fn name(&self) -> &'static str {
                    match self {
                        #( Self::#names(inner) => inner.name(), )*
                    }
                }

                /// The key of the super type, unique within the family
                pub const fn super_type_key(&self) -> u8 {
                    match self {
                        #( Self::#names(_) => #keys, )*
                    }
                }

                /// The key of the cortical type, unique within its super type
                pub const fn key(&self) -> u8 {
                    match self {
                        #( Self::#names(inner) => inner.key(), )*
                    }
                }
            }

            impl ::core::fmt::Display for #family_kind {
                fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                    write!(f, "{}::{}", self.super_type_name(), self.name())
                }
            }
        }
    }

    //endregion

    //region Data Enums

    /// `{Family}Enum<C>`, a `{SuperType}SuperTypeEnum<C>` per super type, and a
    /// `{SuperType}{Type}ModelEnum<C>` per cortical type, holding the data of container `C` for the
    /// models that cortical type allows
    fn generate_enums(&self) -> proc_macro2::TokenStream {
        let model_enums = self.cortical_types().into_iter().map(|leaf| self.generate_model_enum(&leaf));
        let super_type_enums = self.super_types.iter().map(|(super_type, template)| self.generate_super_type_enum(super_type, template));
        let family_enum = self.generate_family_enum();
        quote! {
            #( #model_enums )*
            #( #super_type_enums )*
            #family_enum
        }
    }

    fn generate_model_enum(&self, leaf: &CorticalTypeLeaf<'_>) -> proc_macro2::TokenStream {
        let family_name = &self.family_name;
        let (super_type, cortical_type) = (leaf.super_type, leaf.name);
        let model_enum = Self::model_enum_name(super_type, cortical_type);
        let container = self.model_container_path();
        let model_kind = self.model_kind_path();
        let model_kind_method = self.model_kind_method();
        let model_error = self.model_error_path();
        let error = &self.error_name;
        let serde_derive = serde_derive();

        let models: Vec<&Ident> = leaf.template.models.iter().collect();
        let model_strings: Vec<String> = models.iter().map(|model| model.to_string()).collect();
        let held: Vec<proc_macro2::TokenStream> = models
            .iter()
            .map(|model| {
                let quant_enum = self.model_quant_enum_path(model);
                quote!(#quant_enum<C>)
            })
            .collect();

        let manual_impls = manual_enum_impls(&model_enum, &container, &models, &model_strings, &held);
        let serde_bounds = serde_bounds(&held);
        let doc = format!("Data of container `C`, for any model the `{super_type}::{cortical_type}` cortical type of the `{family_name}` family allows");

        quote! {
            #[doc = #doc]
            #serde_derive
            #serde_bounds
            pub enum #model_enum<C: #container> {
                #( #models(#held), )*
            }

            #manual_impls

            impl<C: #container> #model_enum<C> {
                /// The member of the model family held
                pub const fn #model_kind_method(&self) -> #model_kind {
                    match self {
                        #( Self::#models(inner) => inner.kind(), )*
                    }
                }
            }

            #(
                impl<C: #container> ::core::convert::From<#held> for #model_enum<C> {
                    fn from(value: #held) -> Self {
                        Self::#models(value)
                    }
                }

                impl<C: #container> ::core::convert::TryFrom<#model_enum<C>> for #held {
                    type Error = #error;

                    #[allow(unreachable_patterns)]
                    fn try_from(value: #model_enum<C>) -> ::core::result::Result<Self, Self::Error> {
                        match value {
                            #model_enum::#models(inner) => ::core::result::Result::Ok(inner),
                            other => ::core::result::Result::Err(#error::from(#model_error::IncorrectModel { expected: #model_strings, actual: other.#model_kind_method() })),
                        }
                    }
                }
            )*
        }
    }

    fn generate_super_type_enum(&self, super_type: &Ident, template: &SuperTypeTemplate) -> proc_macro2::TokenStream {
        let family_name = &self.family_name;
        let family_kind = self.family_kind_name();
        let kind = Self::super_type_kind_name(super_type);
        let enum_name = Self::super_type_enum_name(super_type);
        let container = self.model_container_path();
        let model_kind = self.model_kind_path();
        let model_kind_method = self.model_kind_method();
        let error = &self.error_name;
        let serde_derive = serde_derive();

        let names: Vec<&Ident> = template.cortical_types.keys().collect();
        let name_strings: Vec<String> = names.iter().map(|name| name.to_string()).collect();
        let attributes = template.cortical_types.values().map(|cortical_type| &cortical_type.attributes);
        let held: Vec<proc_macro2::TokenStream> = names
            .iter()
            .map(|name| {
                let model_enum = Self::model_enum_name(super_type, name);
                quote!(#model_enum<C>)
            })
            .collect();

        let manual_impls = manual_enum_impls(&enum_name, &container, &names, &name_strings, &held);
        let serde_bounds = serde_bounds(&held);
        let doc = format!("Data of container `C`, for any cortical type of the `{super_type}` super type of the `{family_name}` family, at any model it allows");

        quote! {
            #[doc = #doc]
            #serde_derive
            #serde_bounds
            pub enum #enum_name<C: #container> {
                #(
                    #( #attributes )*
                    #names(#held),
                )*
            }

            #manual_impls

            impl<C: #container> #enum_name<C> {
                /// The cortical type held
                pub const fn kind(&self) -> #kind {
                    match self {
                        #( Self::#names(_) => #kind::#names, )*
                    }
                }

                /// The member of the model family held
                pub const fn #model_kind_method(&self) -> #model_kind {
                    match self {
                        #( Self::#names(inner) => inner.#model_kind_method(), )*
                    }
                }
            }

            #(
                impl<C: #container> ::core::convert::From<#held> for #enum_name<C> {
                    fn from(value: #held) -> Self {
                        Self::#names(value)
                    }
                }

                impl<C: #container> ::core::convert::TryFrom<#enum_name<C>> for #held {
                    type Error = #error;

                    #[allow(unreachable_patterns)]
                    fn try_from(value: #enum_name<C>) -> ::core::result::Result<Self, Self::Error> {
                        match value {
                            #enum_name::#names(inner) => ::core::result::Result::Ok(inner),
                            other => ::core::result::Result::Err(#error::IncorrectCorticalType {
                                expected: #family_kind::#super_type(#kind::#names),
                                actual: #family_kind::#super_type(other.kind()),
                            }),
                        }
                    }
                }
            )*
        }
    }

    fn generate_family_enum(&self) -> proc_macro2::TokenStream {
        let family_name = &self.family_name;
        let family_kind = self.family_kind_name();
        let enum_name = self.family_enum_name();
        let container = self.model_container_path();
        let model_kind = self.model_kind_path();
        let model_kind_method = self.model_kind_method();
        let error = &self.error_name;
        let serde_derive = serde_derive();

        let names: Vec<&Ident> = self.super_types.keys().collect();
        let name_strings: Vec<String> = names.iter().map(|name| name.to_string()).collect();
        let attributes = self.super_types.values().map(|super_type| &super_type.attributes);
        let held: Vec<proc_macro2::TokenStream> = names
            .iter()
            .map(|name| {
                let super_type_enum = Self::super_type_enum_name(name);
                quote!(#super_type_enum<C>)
            })
            .collect();

        let manual_impls = manual_enum_impls(&enum_name, &container, &names, &name_strings, &held);
        let serde_bounds = serde_bounds(&held);
        let doc = format!("Data of container `C`, for any cortical type of the `{family_name}` family, at any model it allows");

        quote! {
            #[doc = #doc]
            #serde_derive
            #serde_bounds
            pub enum #enum_name<C: #container> {
                #(
                    #( #attributes )*
                    #names(#held),
                )*
            }

            #manual_impls

            impl<C: #container> #enum_name<C> {
                /// The cortical type held
                pub const fn kind(&self) -> #family_kind {
                    match self {
                        #( Self::#names(inner) => #family_kind::#names(inner.kind()), )*
                    }
                }

                /// The member of the model family held
                pub const fn #model_kind_method(&self) -> #model_kind {
                    match self {
                        #( Self::#names(inner) => inner.#model_kind_method(), )*
                    }
                }
            }

            #(
                impl<C: #container> ::core::convert::From<#held> for #enum_name<C> {
                    fn from(value: #held) -> Self {
                        Self::#names(value)
                    }
                }

                impl<C: #container> ::core::convert::TryFrom<#enum_name<C>> for #held {
                    type Error = #error;

                    #[allow(unreachable_patterns)]
                    fn try_from(value: #enum_name<C>) -> ::core::result::Result<Self, Self::Error> {
                        match value {
                            #enum_name::#names(inner) => ::core::result::Result::Ok(inner),
                            other => ::core::result::Result::Err(#error::IncorrectSuperType { expected: #name_strings, actual: other.kind().super_type_name() }),
                        }
                    }
                }
            )*
        }
    }

    //endregion

    //region Uses Traits

    /// A `Uses{Model}` trait per model any cortical type allows, for a cortical type to implement.
    /// Being a trait per model lets a cortical type implement several, each generic over the members
    /// of that model. Its supertrait requires the layout of the cortical type to be the model's
    fn generate_uses_traits(&self) -> proc_macro2::TokenStream {
        let family_name = &self.family_name;
        let base_trait = self.base_trait_path();
        let tag_trait = self.model_tag_trait_path();
        let basis = basis_path();
        let quantization_error = quote!(#basis::quantization::quantizable::DataValueQuantizationError);

        let traits = self.used_models().into_iter().map(|model| {
            let uses_trait = Self::uses_trait_name(model);
            let model_trait = self.model_trait_path(model);
            let tag = self.model_tag_path(model);
            let doc = format!("A cortical type of the `{family_name}` family allowing the `{model}` model");
            quote! {
                #[doc = #doc]
                pub trait #uses_trait: #base_trait<NeuronLayout = <#tag as #tag_trait>::LayoutTag> {
                    /// How a new cortical area of this type starts, at quantization `NM`.
                    /// Returns an error if a decimal default becomes infinite or NaN at `NM`
                    fn default_init<FIQ: FeagiIndexQuantization, NM: #model_trait>() -> ::core::result::Result<CorticalAreaInit<FIQ, NM>, #quantization_error>;
                }
            }
        });

        quote!(#( #traits )*)
    }

    //endregion

    //region Member

    /// `{Family}Member`, implemented for every cortical type, holding its kind.
    /// `{Family}MemberAt<M>`, implemented for every cortical type at every member of the models it
    /// allows, which converts data of that member to and from `{Family}Enum`, and the typed access of
    /// `{Family}Enum` through it. A cortical type allowing several models would need an overlapping
    /// impl per model for that, so each goes through `{Family}MemberVia<M, Tag>` instead, which
    /// differs by the tag of the model
    fn generate_member_traits(&self) -> proc_macro2::TokenStream {
        let family_name = &self.family_name;
        let member_trait = self.member_trait_name();
        let via_trait = self.member_via_trait_name();
        let at_trait = self.member_at_trait_name();
        let base_trait = self.base_trait_path();
        let model_member = self.model_member_path();
        let model_error = self.model_error_path();
        let container = self.model_container_path();
        let family_kind = self.family_kind_name();
        let enum_name = self.family_enum_name();
        let error = &self.error_name;
        let model_kind_method = self.model_kind_method();
        let basis = basis_path();
        let quantization_error = quote!(#basis::quantization::quantizable::DataValueQuantizationError);
        let unqualified = quote!();
        let data = quote!(data);

        let member_impls = self.cortical_types().into_iter().map(|leaf| {
            let cortical_type = self.cortical_type_path(leaf.super_type, leaf.name);
            let kind_value = self.kind_value(leaf.super_type, leaf.name);
            let model_enum = Self::model_enum_name(leaf.super_type, leaf.name);
            quote! {
                impl #member_trait for #cortical_type {
                    const KIND: #family_kind = #kind_value;
                    type Models<C: #container> = #model_enum<C>;
                }
            }
        });

        let leaves = self.cortical_types();
        let via_impls = leaves.iter().flat_map(|leaf| {
            leaf.template.models.iter().map(|model| {
                let cortical_type = self.cortical_type_path(leaf.super_type, leaf.name);
                let model_model_member = self.model_model_member_path(model);
                let tag = self.model_tag_path(model);
                let uses_trait = Self::uses_trait_name(model);
                let wrap = self.enum_value(leaf.super_type, leaf.name, model, &quote!(<M as #model_model_member>::into_quant_enum(data)), &unqualified);
                let pattern = self.enum_value(leaf.super_type, leaf.name, model, &data, &unqualified);
                let mismatch = quote! {
                    other if other.kind() == <Self as #member_trait>::KIND => ::core::result::Result::Err(#error::from(#model_error::IncorrectMember {
                        expected: <M as #model_member>::KIND,
                        actual: other.#model_kind_method(),
                    })),
                    other => ::core::result::Result::Err(#error::IncorrectCorticalType { expected: <Self as #member_trait>::KIND, actual: other.kind() }),
                };
                quote! {
                    impl<M: #model_model_member> #via_trait<M, #tag> for #cortical_type {
                        fn into_family_enum<C: #container>(data: C::Of<M>) -> #enum_name<C> {
                            #wrap
                        }

                        #[allow(unreachable_patterns)]
                        fn try_from_family_enum<C: #container>(value: #enum_name<C>) -> ::core::result::Result<C::Of<M>, #error> {
                            match value {
                                #pattern => <M as #model_model_member>::try_from_quant_enum(#data).map_err(#error::from),
                                #mismatch
                            }
                        }

                        #[allow(unreachable_patterns)]
                        fn try_ref_family_enum<C: #container>(value: &#enum_name<C>) -> ::core::result::Result<&C::Of<M>, #error> {
                            match value {
                                #pattern => <M as #model_model_member>::try_ref_quant_enum(#data).map_err(#error::from),
                                #mismatch
                            }
                        }

                        #[allow(unreachable_patterns)]
                        fn try_mut_family_enum<C: #container>(value: &mut #enum_name<C>) -> ::core::result::Result<&mut C::Of<M>, #error> {
                            match value {
                                #pattern => <M as #model_model_member>::try_mut_quant_enum(#data).map_err(#error::from),
                                #mismatch
                            }
                        }

                        fn default_init<FIQ: FeagiIndexQuantization>() -> ::core::result::Result<CorticalAreaInit<FIQ, M>, #quantization_error> {
                            <Self as #uses_trait>::default_init::<FIQ, M>()
                        }
                    }
                }
            })
        });

        let member_doc = format!("A cortical type of the `{family_name}` family, a unit struct implemented by hand");
        let via_doc = format!(
            "A cortical type of the `{family_name}` family at member `M` of the model named by `Tag`. Implemented for the models a cortical type allows. Use [`{at_trait}`] instead"
        );
        let at_doc = format!(
            "A cortical type of the `{family_name}` family at member `M` of a model it allows. Converts data of any container of the model family between itself and `{enum_name}`"
        );

        quote! {
            #[doc = #member_doc]
            pub trait #member_trait: #base_trait {
                /// The cortical type this is
                const KIND: #family_kind;

                /// The models this cortical type allows, holding the data of container `C`
                type Models<C: #container>;
            }

            #( #member_impls )*

            #[doc = #via_doc]
            pub trait #via_trait<M: #model_member, Tag>: #member_trait {
                fn into_family_enum<C: #container>(data: C::Of<M>) -> #enum_name<C>;

                fn try_from_family_enum<C: #container>(value: #enum_name<C>) -> ::core::result::Result<C::Of<M>, #error>;

                fn try_ref_family_enum<C: #container>(value: &#enum_name<C>) -> ::core::result::Result<&C::Of<M>, #error>;

                fn try_mut_family_enum<C: #container>(value: &mut #enum_name<C>) -> ::core::result::Result<&mut C::Of<M>, #error>;

                fn default_init<FIQ: FeagiIndexQuantization>() -> ::core::result::Result<CorticalAreaInit<FIQ, M>, #quantization_error>;
            }

            #( #via_impls )*

            #[doc = #at_doc]
            pub trait #at_trait<M: #model_member>: #member_trait {
                /// Wraps data of this cortical type at this member
                fn into_family_enum<C: #container>(data: C::Of<M>) -> #enum_name<C>;

                /// Unwraps the data held, if held for this cortical type and member
                fn try_from_family_enum<C: #container>(value: #enum_name<C>) -> ::core::result::Result<C::Of<M>, #error>;

                /// The data held, if held for this cortical type and member
                fn try_ref_family_enum<C: #container>(value: &#enum_name<C>) -> ::core::result::Result<&C::Of<M>, #error>;

                /// The data held, if held for this cortical type and member
                fn try_mut_family_enum<C: #container>(value: &mut #enum_name<C>) -> ::core::result::Result<&mut C::Of<M>, #error>;

                /// How a new cortical area of this type and member starts. Returns an error
                /// if a decimal default becomes infinite or NaN at this member
                fn default_init<FIQ: FeagiIndexQuantization>() -> ::core::result::Result<CorticalAreaInit<FIQ, M>, #quantization_error>;
            }

            impl<T, M> #at_trait<M> for T
            where
                M: #model_member,
                T: #via_trait<M, <M as #model_member>::Tag>,
            {
                fn into_family_enum<C: #container>(data: C::Of<M>) -> #enum_name<C> {
                    <T as #via_trait<M, <M as #model_member>::Tag>>::into_family_enum::<C>(data)
                }

                fn try_from_family_enum<C: #container>(value: #enum_name<C>) -> ::core::result::Result<C::Of<M>, #error> {
                    <T as #via_trait<M, <M as #model_member>::Tag>>::try_from_family_enum::<C>(value)
                }

                fn try_ref_family_enum<C: #container>(value: &#enum_name<C>) -> ::core::result::Result<&C::Of<M>, #error> {
                    <T as #via_trait<M, <M as #model_member>::Tag>>::try_ref_family_enum::<C>(value)
                }

                fn try_mut_family_enum<C: #container>(value: &mut #enum_name<C>) -> ::core::result::Result<&mut C::Of<M>, #error> {
                    <T as #via_trait<M, <M as #model_member>::Tag>>::try_mut_family_enum::<C>(value)
                }

                fn default_init<FIQ: FeagiIndexQuantization>() -> ::core::result::Result<CorticalAreaInit<FIQ, M>, #quantization_error> {
                    <T as #via_trait<M, <M as #model_member>::Tag>>::default_init::<FIQ>()
                }
            }

            impl<C: #container> #enum_name<C> {
                /// Wraps the data of cortical type `T` at member `M`
                pub fn from_data<T: #at_trait<M>, M: #model_member>(data: C::Of<M>) -> Self {
                    <T as #at_trait<M>>::into_family_enum::<C>(data)
                }

                /// The data held, if held for cortical type `T` at member `M`
                pub fn try_as<T: #at_trait<M>, M: #model_member>(&self) -> ::core::result::Result<&C::Of<M>, #error> {
                    <T as #at_trait<M>>::try_ref_family_enum::<C>(self)
                }

                /// The data held, if held for cortical type `T` at member `M`
                pub fn try_as_mut<T: #at_trait<M>, M: #model_member>(&mut self) -> ::core::result::Result<&mut C::Of<M>, #error> {
                    <T as #at_trait<M>>::try_mut_family_enum::<C>(self)
                }

                /// Unwraps the data held, if held for cortical type `T` at member `M`
                pub fn try_into_data<T: #at_trait<M>, M: #model_member>(self) -> ::core::result::Result<C::Of<M>, #error> {
                    <T as #at_trait<M>>::try_from_family_enum::<C>(self)
                }
            }
        }
    }

    //endregion

    //region Dispatch Macro

    /// `with_{family}!` over `{Family}Enum`. Its `<T, NM>` form dispatches further through the
    /// per model dispatch macro of the model family, `with_{model_family}_{model}!`
    fn generate_with_macro(&self) -> proc_macro2::TokenStream {
        let family_module = self.family_module();
        let qualified = quote!(#family_module::);
        let with_macro = self.with_macro_name();
        let enum_name = self.family_enum_name();
        let model_family_snake = self.model_family_name.to_string().to_snake_case();
        let inner = quote!(__cortical_type_inner);

        let leaves = self.cortical_types();
        let type_patterns: Vec<proc_macro2::TokenStream> = leaves
            .iter()
            .map(|leaf| self.enum_value_of_type(leaf.super_type, leaf.name, &inner, &qualified))
            .collect();

        let mut model_patterns = Vec::new();
        let mut model_macros = Vec::new();
        let mut cortical_types = Vec::new();
        let mut model_macro_names: Vec<String> = Vec::new();
        for leaf in &leaves {
            for model in &leaf.template.models {
                let model_macro = format_ident!("with_{}_{}", model_family_snake, model.to_string().to_snake_case());
                let macro_name = format!("`{model_macro}!`");
                if !model_macro_names.contains(&macro_name) {
                    model_macro_names.push(macro_name);
                }
                model_patterns.push(self.enum_value(leaf.super_type, leaf.name, model, &inner, &qualified));
                model_macros.push(model_macro);
                cortical_types.push(self.cortical_type_path(leaf.super_type, leaf.name));
            }
        }
        let model_macro_names = model_macro_names.join(", ");

        let doc = format!(
            "Runs the same code against whichever cortical type of `{enum_name}` is held.\n\n\
             `{with_macro}!(value, |inner| body)` binds the model enum of the cortical type held to \
             `inner`. `{with_macro}!(value, <T, NM> |data| body)` binds the data held to `data`, and \
             aliases the cortical type struct held as `T` and the member of its model as `NM` within \
             `body`. That form calls the model dispatch macros {model_macro_names}, which must be \
             in scope. Requires the `{family_module}` and model family modules in scope"
        );

        quote! {
            #[doc = #doc]
            #[macro_export]
            macro_rules! #with_macro {
                ($value:expr, |$inner:pat_param| $body:expr) => {
                    match $value {
                        #( #type_patterns => { let $inner = #inner; $body } )*
                    }
                };
                ($value:expr, <$cortical_type:ident, $member:ident> |$data:pat_param| $body:expr) => {
                    match $value {
                        #( #model_patterns => #model_macros!(#inner, <$member> |$data| {
                            #[allow(dead_code)]
                            type $cortical_type = #cortical_types;
                            $body
                        }), )*
                    }
                };
            }
        }
    }

    //endregion
}

impl Parse for CorticalTypeFamilyTemplate {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let family_attributes = input.call(Attribute::parse_outer)?;

        input.parse::<kw::Family>()?;
        input.parse::<Token![:]>()?;
        let family_name = input.parse::<Ident>()?;
        parse_optional_comma(input)?;

        input.parse::<kw::Error>()?;
        input.parse::<Token![:]>()?;
        let error_name = input.parse::<Ident>()?;
        parse_optional_comma(input)?;

        input.parse::<kw::ModelFamily>()?;
        input.parse::<Token![:]>()?;
        let model_family_name = input.parse::<Ident>()?;
        parse_optional_comma(input)?;

        input.parse::<kw::ModelFamilyError>()?;
        input.parse::<Token![:]>()?;
        let model_family_error_name = input.parse::<Ident>()?;
        parse_optional_comma(input)?;

        input.parse::<kw::Identifier>()?;
        input.parse::<Token![:]>()?;
        let identifier_name = input.parse::<Ident>()?;
        parse_optional_comma(input)?;

        input.parse::<kw::IdentifierError>()?;
        input.parse::<Token![:]>()?;
        let identifier_error_name = input.parse::<Ident>()?;
        parse_optional_comma(input)?;

        input.parse::<kw::Types>()?;
        input.parse::<Token![:]>()?;
        let super_types = parse_super_types(input)?;
        parse_optional_comma(input)?;

        if !input.is_empty() {
            return Err(input.error("unexpected input after the template, `Types` is the last section"));
        }

        Ok(Self {
            family_attributes,
            family_name,
            error_name,
            model_family_name,
            model_family_error_name,
            identifier_name,
            identifier_error_name,
            super_types,
        })
    }
}

/// Writes the template back out in the format it is parsed from
impl ToTokens for CorticalTypeFamilyTemplate {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let super_types = self.super_types.iter().map(|(name, super_type)| {
            let attributes = &super_type.attributes;
            let key = Literal::u8_unsuffixed(super_type.key);
            let cortical_types = super_type.cortical_types.iter().map(|(name, cortical_type)| {
                let attributes = &cortical_type.attributes;
                let key = Literal::u8_unsuffixed(cortical_type.key);
                let models = &cortical_type.models;
                quote! {
                    #( #attributes )*
                    #name = #key: [#( #models ),*],
                }
            });
            quote! {
                #( #attributes )*
                #name = #key: {
                    #( #cortical_types )*
                }
            }
        });

        let family_attributes = &self.family_attributes;
        let family_name = &self.family_name;
        let error_name = &self.error_name;
        let model_family_name = &self.model_family_name;
        let model_family_error_name = &self.model_family_error_name;
        let identifier_name = &self.identifier_name;
        let identifier_error_name = &self.identifier_error_name;
        tokens.extend(quote! {
            #( #family_attributes )*
            Family: #family_name,
            Error: #error_name,
            ModelFamily: #model_family_name,
            ModelFamilyError: #model_family_error_name,
            Identifier: #identifier_name,
            IdentifierError: #identifier_error_name,

            Types: {
                #( #super_types )*
            }
        });
    }
}

/// `{ #[attr] SuperType = 1: { ... } ... }`, the commas being optional
fn parse_super_types(input: ParseStream) -> syn::Result<IndexMap<Ident, SuperTypeTemplate>> {
    let super_types_input;
    let brace = braced!(super_types_input in input);
    let mut super_types: IndexMap<Ident, SuperTypeTemplate> = IndexMap::new();
    while !super_types_input.is_empty() {
        let attributes = super_types_input.call(Attribute::parse_outer)?;
        let name = super_types_input.parse::<Ident>()?;
        let key_literal = parse_key(&super_types_input, &name)?;
        super_types_input.parse::<Token![:]>()?;
        if !super_types_input.peek(syn::token::Brace) {
            return Err(syn::Error::new(name.span(), format!("`{name}` must be a super type holding cortical types, as every cortical type belongs to exactly one super type")));
        }
        let cortical_types = parse_cortical_types(&super_types_input, &name)?;
        check_unique(&name, super_types.keys().any(|existing| *existing == name), super_types.values().any(|existing| existing.key == key_literal.0), &key_literal.1)?;
        super_types.insert(name, SuperTypeTemplate { attributes, key: key_literal.0, cortical_types });
        parse_optional_comma(&super_types_input)?;
    }
    if super_types.is_empty() {
        return Err(syn::Error::new(brace.span.join(), "a cortical type family needs at least one super type"));
    }
    Ok(super_types)
}

/// `{ #[attr] CorticalType = 1: [Model, ...], ... }`, the commas between cortical types being optional
fn parse_cortical_types(input: ParseStream, super_type: &Ident) -> syn::Result<IndexMap<Ident, CorticalTypeTemplate>> {
    let cortical_types_input;
    let brace = braced!(cortical_types_input in input);
    let mut cortical_types: IndexMap<Ident, CorticalTypeTemplate> = IndexMap::new();
    while !cortical_types_input.is_empty() {
        let attributes = cortical_types_input.call(Attribute::parse_outer)?;
        let name = cortical_types_input.parse::<Ident>()?;
        let key_literal = parse_key(&cortical_types_input, &name)?;
        cortical_types_input.parse::<Token![:]>()?;
        if cortical_types_input.peek(syn::token::Brace) {
            return Err(syn::Error::new(name.span(), format!("`{name}` cannot be a super type inside of the super type `{super_type}`, as an ID only has room for a super type and a cortical type")));
        }
        let models = parse_models(&cortical_types_input, &name)?;
        check_unique(&name, cortical_types.keys().any(|existing| *existing == name), cortical_types.values().any(|existing| existing.key == key_literal.0), &key_literal.1)?;
        cortical_types.insert(name, CorticalTypeTemplate { attributes, key: key_literal.0, models });
        parse_optional_comma(&cortical_types_input)?;
    }
    if cortical_types.is_empty() {
        return Err(syn::Error::new(brace.span.join(), format!("super type `{super_type}` needs at least one cortical type")));
    }
    Ok(cortical_types)
}

/// `= 1`, a key from 1 to 255. Zero is reserved, so that zeroed memory never reads as a valid key.
/// Returns the literal with the key, for errors about the key to point at
fn parse_key(input: ParseStream, name: &Ident) -> syn::Result<(u8, LitInt)> {
    input.parse::<Token![=]>()?;
    let literal = input.parse::<LitInt>()?;
    let key = literal
        .base10_parse::<u8>()
        .map_err(|_| syn::Error::new(literal.span(), format!("the key of `{name}` must be from 1 to 255")))?;
    if key == 0 {
        return Err(syn::Error::new(literal.span(), format!("the key of `{name}` must be from 1 to 255, as 0 is reserved")));
    }
    Ok((key, literal))
}

fn check_unique(name: &Ident, name_used: bool, key_used: bool, key: &LitInt) -> syn::Result<()> {
    if name_used {
        return Err(syn::Error::new(name.span(), format!("`{name}` is defined more than once")));
    }
    if key_used {
        return Err(syn::Error::new(key.span(), format!("the key of `{name}` is already used by another entry at this level")));
    }
    Ok(())
}

/// `[Model, ...]`
fn parse_models(input: ParseStream, cortical_type: &Ident) -> syn::Result<Vec<Ident>> {
    let models_input;
    let bracket = bracketed!(models_input in input);
    let parsed = models_input.parse_terminated(Ident::parse, Token![,])?;
    let mut models: Vec<Ident> = Vec::new();
    for model in parsed {
        if models.contains(&model) {
            return Err(syn::Error::new(model.span(), format!("cortical type `{cortical_type}` lists the model `{model}` more than once")));
        }
        models.push(model);
    }
    if models.is_empty() {
        return Err(syn::Error::new(bracket.span.join(), format!("cortical type `{cortical_type}` needs at least one model")));
    }
    Ok(models)
}

//endregion
