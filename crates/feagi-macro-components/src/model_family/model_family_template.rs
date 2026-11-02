use crate::common::{basis_path, parse_optional_comma};
use heck::ToSnakeCase;
use indexmap::IndexMap;
use proc_macro2::Ident;
use quote::{format_ident, quote, ToTokens};
use syn::parse::{Parse, ParseStream};
use syn::{braced, Attribute, LitStr, Token};

mod kw {
    syn::custom_keyword!(Family);
    syn::custom_keyword!(Error);
    syn::custom_keyword!(Models);
}

/// Path serde is reached by from generated code, so the caller does not need to depend on serde
const SERDE_CRATE: &str = "::feagi_basis::__private::serde";

//region Model Family Template

/// A family of models: a base trait, a sub trait of it per model, and a zero sized marker struct
/// per quantization level of each model, implementing the model's trait. Formatted as the
/// following:
/// ```text
/// /// Optional family description
/// Family: NeuronModel,
/// Error: NeuronModelSelectionError,
///
/// Models: {
///     /// Optional model description
///     FoundationPower: {
///         /// Optional quantization level description
///         Float16,
///         Float32,
///     }
///     FeagiAdvanced: { DefaultQuant }
/// }
/// ```
/// No paths are given. Generated code instead expects a module named the snake case of the family
/// to be in scope where it is expanded, holding the base trait `{Family}` and the items generated
/// for the family, with a sub module per model named the snake case of the model, holding the
/// model trait `{Family}{Model}` and the marker struct `{Family}{Model}{Level}` of each of its
/// quantization levels. Marker structs must be unit structs. Every map keeps its template order
pub struct ModelFamilyTemplate {
    pub family_attributes: Vec<Attribute>,
    pub family_name: Ident,
    pub error_name: Ident,
    pub models: IndexMap<Ident, ModelTemplate>,
}

/// A model of a family, with the attributes of each of its quantization levels
pub struct ModelTemplate {
    pub attributes: Vec<Attribute>,
    pub quantization_levels: IndexMap<Ident, Vec<Attribute>>,
}

impl ModelFamilyTemplate {
    //region Names

    fn family_snake_case(&self) -> String {
        self.family_name.to_string().to_snake_case()
    }

    pub fn template_macro_name(&self) -> Ident {
        format_ident!("{}_family_template", self.family_snake_case())
    }

    /// The module expected in scope wherever generated code is expanded
    pub fn family_module(&self) -> Ident {
        format_ident!("{}", self.family_snake_case())
    }

    /// The sub module of the family module holding the items of `model`
    pub fn model_module(model: &Ident) -> Ident {
        format_ident!("{}", model.to_string().to_snake_case())
    }

    pub fn base_trait_path(&self) -> proc_macro2::TokenStream {
        let family_module = self.family_module();
        let family_name = &self.family_name;
        quote!(#family_module::#family_name)
    }

    pub fn model_trait_path(&self, model: &Ident) -> proc_macro2::TokenStream {
        let family_module = self.family_module();
        let model_module = Self::model_module(model);
        let model_trait = format_ident!("{}{}", self.family_name, model);
        quote!(#family_module::#model_module::#model_trait)
    }

    pub fn marker_path(&self, model: &Ident, level: &Ident) -> proc_macro2::TokenStream {
        let family_module = self.family_module();
        let model_module = Self::model_module(model);
        let marker = format_ident!("{}{}{}", self.family_name, model, level);
        quote!(#family_module::#model_module::#marker)
    }

    pub fn kind_name(&self) -> Ident {
        format_ident!("{}Kind", self.family_name)
    }

    pub fn quant_kind_name(model: &Ident) -> Ident {
        format_ident!("{}QuantKind", model)
    }

    pub fn container_trait_name(&self) -> Ident {
        format_ident!("{}Container", self.family_name)
    }

    pub fn enum_name(&self) -> Ident {
        format_ident!("{}Enum", self.family_name)
    }

    pub fn quant_enum_name(model: &Ident) -> Ident {
        format_ident!("{}QuantEnum", model)
    }

    pub fn member_trait_name(&self) -> Ident {
        format_ident!("{}Member", self.family_name)
    }

    pub fn model_member_trait_name(&self, model: &Ident) -> Ident {
        format_ident!("{}{}Member", self.family_name, model)
    }

    pub fn with_macro_name(&self) -> Ident {
        format_ident!("with_{}", self.family_snake_case())
    }

    pub fn with_kind_macro_name(&self) -> Ident {
        format_ident!("with_{}_kind", self.family_snake_case())
    }

    pub fn with_model_macro_name(&self, model: &Ident) -> Ident {
        format_ident!("with_{}_{}", self.family_snake_case(), model.to_string().to_snake_case())
    }

    /// Every (model, quantization level) of the family, in template order
    fn members(&self) -> Vec<(&Ident, &Ident)> {
        self.models
            .iter()
            .flat_map(|(model, template)| template.quantization_levels.keys().map(move |level| (model, level)))
            .collect()
    }

    //endregion

    /// Generate the template macro, which holds this template once validated. Its single input is
    /// the path of a macro, which it calls with the template, as in
    /// `{family}_family_template!(expand_model_family)`. Exported, so other crates may use it
    pub fn generate_rust_template_macro(&self) -> proc_macro2::TokenStream {
        let macro_name = self.template_macro_name();
        let doc = LitStr::new(
            &format!("The validated model family template of the `{}` family. Calls the given macro with it", self.family_name),
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

    /// Generate the kinds, error, container trait, enums, member traits, and dispatch macros of
    /// the family
    pub fn generate_rust_family(&self) -> proc_macro2::TokenStream {
        let kinds = self.generate_kinds();
        let error = self.generate_error();
        let container = self.generate_container_trait();
        let quant_enums = self.models.keys().map(|model| self.generate_quant_enum(model));
        let family_enum = self.generate_family_enum();
        let members = self.generate_member_traits();
        let with_macros = self.generate_with_macros();
        quote! {
            #kinds
            #error
            #container
            #( #quant_enums )*
            #family_enum
            #members
            #with_macros
        }
    }

    //region Kinds

    /// A data less `{Model}QuantKind` per model, with a variant per quantization level, and the
    /// family's `{Family}Kind`, with a variant per model holding its quantization kind
    fn generate_kinds(&self) -> proc_macro2::TokenStream {
        let family_name = &self.family_name;
        let kind = self.kind_name();
        let member_trait = self.member_trait_name();
        let error = &self.error_name;
        let serde_derive = serde_derive();

        let quant_kinds = self.models.iter().map(|(model, template)| {
            let quant_kind = Self::quant_kind_name(model);
            let doc = format!("The quantization levels of the `{model}` model of the `{family_name}` family");
            let levels: Vec<&Ident> = template.quantization_levels.keys().collect();
            let level_attributes = template.quantization_levels.values();
            let level_names = levels.iter().map(|level| level.to_string());
            let count = levels.len();
            quote! {
                #[doc = #doc]
                #[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
                #serde_derive
                pub enum #quant_kind {
                    #(
                        #( #level_attributes )*
                        #levels,
                    )*
                }

                impl #quant_kind {
                    /// Every quantization level of the model, in template order
                    pub const ALL: [Self; #count] = [#( Self::#levels ),*];

                    /// The name of the quantization level
                    pub const fn name(&self) -> &'static str {
                        match self {
                            #( Self::#levels => #level_names, )*
                        }
                    }
                }

                impl ::core::fmt::Display for #quant_kind {
                    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                        f.write_str(self.name())
                    }
                }
            }
        });

        let models: Vec<&Ident> = self.models.keys().collect();
        let model_attributes = self.models.values().map(|template| &template.attributes);
        let model_names = models.iter().map(|model| model.to_string());
        let model_quant_kinds: Vec<Ident> = models.iter().map(|model| Self::quant_kind_name(model)).collect();
        let members = self.members();
        let member_count = members.len();
        let all_kinds = members.iter().map(|(model, level)| {
            let quant_kind = Self::quant_kind_name(model);
            quote!(Self::#model(#quant_kind::#level))
        });
        let doc = format!("Every member of the `{family_name}` family, as a model and its quantization level, without any data");

        quote! {
            #( #quant_kinds )*

            #[doc = #doc]
            #[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
            #serde_derive
            pub enum #kind {
                #(
                    #( #model_attributes )*
                    #models(#model_quant_kinds),
                )*
            }

            impl #kind {
                /// Every member of the family, in template order
                pub const ALL: [Self; #member_count] = [#( #all_kinds ),*];

                /// The name of the model
                pub const fn model_name(&self) -> &'static str {
                    match self {
                        #( Self::#models(_) => #model_names, )*
                    }
                }

                /// The name of the quantization level of the model
                pub const fn quantization_name(&self) -> &'static str {
                    match self {
                        #( Self::#models(quantization) => quantization.name(), )*
                    }
                }

                /// The marker struct of `M`, if this is the kind of `M`
                pub fn try_as_marker<M: #member_trait>(self) -> ::core::result::Result<M, #error> {
                    if self == M::KIND {
                        ::core::result::Result::Ok(M::MARKER)
                    } else {
                        ::core::result::Result::Err(#error::IncorrectMember { expected: M::KIND, actual: self })
                    }
                }
            }

            impl ::core::fmt::Display for #kind {
                fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                    write!(f, "{}::{}", self.model_name(), self.quantization_name())
                }
            }
        }
    }

    //endregion

    //region Error

    /// The error of accessing data of a family enum as a member or model it does not hold
    fn generate_error(&self) -> proc_macro2::TokenStream {
        let family_name = &self.family_name;
        let error = &self.error_name;
        let kind = self.kind_name();
        let doc = format!("Data of the `{family_name}` family was accessed as a member or model it does not hold");
        let incorrect_member = LitStr::new(
            &format!("Tried to access the {family_name} `{{}}` when it was actually `{{}}`"),
            proc_macro2::Span::call_site(),
        );
        let incorrect_model = LitStr::new(
            &format!("Tried to access a {family_name} of model `{{}}` when it was actually `{{}}`"),
            proc_macro2::Span::call_site(),
        );
        quote! {
            #[doc = #doc]
            #[derive(Clone, Copy, PartialEq, Eq, Debug)]
            pub enum #error {
                /// Accessed as a member of the family other than the one held
                IncorrectMember { expected: #kind, actual: #kind },
                /// Accessed as a model of the family other than the one held
                IncorrectModel { expected: &'static str, actual: #kind },
            }

            impl ::core::fmt::Display for #error {
                fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                    match self {
                        Self::IncorrectMember { expected, actual } => write!(f, #incorrect_member, expected, actual),
                        Self::IncorrectModel { expected, actual } => write!(f, #incorrect_model, expected, actual),
                    }
                }
            }

            impl ::core::error::Error for #error {}
        }
    }

    //endregion

    //region Container

    /// The `{Family}Container` trait, a family of data generic over the members of the family,
    /// which every enum of the family holds
    fn generate_container_trait(&self) -> proc_macro2::TokenStream {
        let family_name = &self.family_name;
        let container = self.container_trait_name();
        let base_trait = self.base_trait_path();
        let enum_name = self.enum_name();
        let doc = format!(
            "A kind of data held for any member of the `{family_name}` family, such as `{enum_name}<C>` holds. Meant to be implemented on a zero sized struct per kind of data"
        );
        quote! {
            #[doc = #doc]
            pub trait #container {
                /// The data held for member `M` of the family
                type Of<M: #base_trait>;
            }
        }
    }

    //endregion

    //region Enums

    /// The `{Model}QuantEnum<C>` of `model`, with a variant holding the data of container `C` per
    /// quantization level
    fn generate_quant_enum(&self, model: &Ident) -> proc_macro2::TokenStream {
        let family_name = &self.family_name;
        let template = &self.models[model];
        let quant_enum = Self::quant_enum_name(model);
        let quant_kind = Self::quant_kind_name(model);
        let kind = self.kind_name();
        let container = self.container_trait_name();
        let model_member = self.model_member_trait_name(model);
        let error = &self.error_name;

        let levels: Vec<&Ident> = template.quantization_levels.keys().collect();
        let level_attributes = template.quantization_levels.values();
        let level_names: Vec<String> = levels.iter().map(|level| level.to_string()).collect();
        let held: Vec<proc_macro2::TokenStream> = levels
            .iter()
            .map(|level| {
                let marker = self.marker_path(model, level);
                quote!(C::Of<#marker>)
            })
            .collect();

        let serde_derive = serde_derive();
        let serde_bounds = serde_bounds(&held);
        let manual_impls = manual_enum_impls(&quant_enum, &container, &levels, &level_names, &held);
        let doc = format!("Data of container `C`, for any quantization level of the `{model}` model of the `{family_name}` family");

        quote! {
            #[doc = #doc]
            #serde_derive
            #serde_bounds
            pub enum #quant_enum<C: #container> {
                #(
                    #( #level_attributes )*
                    #levels(#held),
                )*
            }

            #manual_impls

            impl<C: #container> #quant_enum<C> {
                /// The quantization level held
                pub const fn quant_kind(&self) -> #quant_kind {
                    match self {
                        #( Self::#levels(_) => #quant_kind::#levels, )*
                    }
                }

                /// The member of the family held
                pub const fn kind(&self) -> #kind {
                    #kind::#model(self.quant_kind())
                }

                /// Wraps the data of member `M`
                pub fn from_data<M: #model_member>(data: C::Of<M>) -> Self {
                    M::into_quant_enum(data)
                }

                /// The data held, if held for member `M`
                pub fn try_as<M: #model_member>(&self) -> ::core::result::Result<&C::Of<M>, #error> {
                    M::try_ref_quant_enum(self)
                }

                /// The data held, if held for member `M`
                pub fn try_as_mut<M: #model_member>(&mut self) -> ::core::result::Result<&mut C::Of<M>, #error> {
                    M::try_mut_quant_enum(self)
                }

                /// Unwraps the data held, if held for member `M`
                pub fn try_into_data<M: #model_member>(self) -> ::core::result::Result<C::Of<M>, #error> {
                    M::try_from_quant_enum(self)
                }

                /// The marker struct of `M`, if the data held is for member `M`
                pub fn try_as_marker<M: #model_member>(&self) -> ::core::result::Result<M, #error> {
                    self.kind().try_as_marker()
                }
            }
        }
    }

    /// The `{Family}Enum<C>`, with a variant holding the `{Model}QuantEnum<C>` per model, and the
    /// conversions between it and each of them
    fn generate_family_enum(&self) -> proc_macro2::TokenStream {
        let family_name = &self.family_name;
        let enum_name = self.enum_name();
        let kind = self.kind_name();
        let container = self.container_trait_name();
        let member_trait = self.member_trait_name();
        let error = &self.error_name;

        let models: Vec<&Ident> = self.models.keys().collect();
        let model_attributes = self.models.values().map(|template| &template.attributes);
        let model_names: Vec<String> = models.iter().map(|model| model.to_string()).collect();
        let quant_enums: Vec<Ident> = models.iter().map(|model| Self::quant_enum_name(model)).collect();
        let held: Vec<proc_macro2::TokenStream> = quant_enums.iter().map(|quant_enum| quote!(#quant_enum<C>)).collect();

        let serde_derive = serde_derive();
        let serde_bounds = serde_bounds(&held);
        let manual_impls = manual_enum_impls(&enum_name, &container, &models, &model_names, &held);
        let doc = format!("Data of container `C`, for any member of the `{family_name}` family");

        quote! {
            #[doc = #doc]
            #serde_derive
            #serde_bounds
            pub enum #enum_name<C: #container> {
                #(
                    #( #model_attributes )*
                    #models(#held),
                )*
            }

            #manual_impls

            impl<C: #container> #enum_name<C> {
                /// The member of the family held
                pub const fn kind(&self) -> #kind {
                    match self {
                        #( Self::#models(inner) => inner.kind(), )*
                    }
                }

                /// Wraps the data of member `M`
                pub fn from_data<M: #member_trait>(data: C::Of<M>) -> Self {
                    M::into_family_enum(data)
                }

                /// The data held, if held for member `M`
                pub fn try_as<M: #member_trait>(&self) -> ::core::result::Result<&C::Of<M>, #error> {
                    M::try_ref_family_enum(self)
                }

                /// The data held, if held for member `M`
                pub fn try_as_mut<M: #member_trait>(&mut self) -> ::core::result::Result<&mut C::Of<M>, #error> {
                    M::try_mut_family_enum(self)
                }

                /// Unwraps the data held, if held for member `M`
                pub fn try_into_data<M: #member_trait>(self) -> ::core::result::Result<C::Of<M>, #error> {
                    M::try_from_family_enum(self)
                }

                /// The marker struct of `M`, if the data held is for member `M`
                pub fn try_as_marker<M: #member_trait>(&self) -> ::core::result::Result<M, #error> {
                    self.kind().try_as_marker()
                }
            }

            #(
                impl<C: #container> ::core::convert::From<#held> for #enum_name<C> {
                    fn from(value: #held) -> Self {
                        Self::#models(value)
                    }
                }

                impl<C: #container> ::core::convert::TryFrom<#enum_name<C>> for #held {
                    type Error = #error;

                    #[allow(unreachable_patterns)]
                    fn try_from(value: #enum_name<C>) -> ::core::result::Result<Self, Self::Error> {
                        match value {
                            #enum_name::#models(inner) => ::core::result::Result::Ok(inner),
                            other => ::core::result::Result::Err(#error::IncorrectModel { expected: #model_names, actual: other.kind() }),
                        }
                    }
                }
            )*
        }
    }

    //endregion

    //region Members

    /// The `{Family}Member` trait and a `{Family}{Model}Member` trait per model, each implemented
    /// for every marker struct of the model, which convert data of the marker to and from the
    /// enums of the family
    fn generate_member_traits(&self) -> proc_macro2::TokenStream {
        let family_name = &self.family_name;
        let base_trait = self.base_trait_path();
        let member_trait = self.member_trait_name();
        let kind = self.kind_name();
        let container = self.container_trait_name();
        let enum_name = self.enum_name();
        let error = &self.error_name;
        let member_doc = format!(
            "A member of the `{family_name}` family, a quantization level of one of its models. Converts data of any container of the family between itself and `{enum_name}`"
        );

        let model_members = self.models.iter().map(|(model, template)| {
            let model_member = self.model_member_trait_name(model);
            let model_trait = self.model_trait_path(model);
            let quant_kind = Self::quant_kind_name(model);
            let quant_enum = Self::quant_enum_name(model);
            let model_member_doc = format!(
                "A member of the `{model}` model of the `{family_name}` family. Converts data of any container of the family between itself and `{quant_enum}`"
            );

            let impls = template.quantization_levels.keys().map(|level| {
                let marker = self.marker_path(model, level);
                quote! {
                    impl #model_member for #marker {
                        const QUANT_KIND: #quant_kind = #quant_kind::#level;

                        fn into_quant_enum<C: #container>(data: C::Of<Self>) -> #quant_enum<C> {
                            #quant_enum::#level(data)
                        }

                        #[allow(unreachable_patterns)]
                        fn try_from_quant_enum<C: #container>(value: #quant_enum<C>) -> ::core::result::Result<C::Of<Self>, #error> {
                            match value {
                                #quant_enum::#level(data) => ::core::result::Result::Ok(data),
                                other => ::core::result::Result::Err(#error::IncorrectMember { expected: <Self as #member_trait>::KIND, actual: other.kind() }),
                            }
                        }

                        #[allow(unreachable_patterns)]
                        fn try_ref_quant_enum<C: #container>(value: &#quant_enum<C>) -> ::core::result::Result<&C::Of<Self>, #error> {
                            match value {
                                #quant_enum::#level(data) => ::core::result::Result::Ok(data),
                                other => ::core::result::Result::Err(#error::IncorrectMember { expected: <Self as #member_trait>::KIND, actual: other.kind() }),
                            }
                        }

                        #[allow(unreachable_patterns)]
                        fn try_mut_quant_enum<C: #container>(value: &mut #quant_enum<C>) -> ::core::result::Result<&mut C::Of<Self>, #error> {
                            match value {
                                #quant_enum::#level(data) => ::core::result::Result::Ok(data),
                                other => ::core::result::Result::Err(#error::IncorrectMember { expected: <Self as #member_trait>::KIND, actual: other.kind() }),
                            }
                        }
                    }

                    impl #member_trait for #marker {
                        const KIND: #kind = #kind::#model(#quant_kind::#level);
                        const MARKER: Self = #marker;

                        fn into_family_enum<C: #container>(data: C::Of<Self>) -> #enum_name<C> {
                            #enum_name::#model(<Self as #model_member>::into_quant_enum(data))
                        }

                        #[allow(unreachable_patterns)]
                        fn try_from_family_enum<C: #container>(value: #enum_name<C>) -> ::core::result::Result<C::Of<Self>, #error> {
                            match value {
                                #enum_name::#model(inner) => <Self as #model_member>::try_from_quant_enum(inner),
                                other => ::core::result::Result::Err(#error::IncorrectMember { expected: <Self as #member_trait>::KIND, actual: other.kind() }),
                            }
                        }

                        #[allow(unreachable_patterns)]
                        fn try_ref_family_enum<C: #container>(value: &#enum_name<C>) -> ::core::result::Result<&C::Of<Self>, #error> {
                            match value {
                                #enum_name::#model(inner) => <Self as #model_member>::try_ref_quant_enum(inner),
                                other => ::core::result::Result::Err(#error::IncorrectMember { expected: <Self as #member_trait>::KIND, actual: other.kind() }),
                            }
                        }

                        #[allow(unreachable_patterns)]
                        fn try_mut_family_enum<C: #container>(value: &mut #enum_name<C>) -> ::core::result::Result<&mut C::Of<Self>, #error> {
                            match value {
                                #enum_name::#model(inner) => <Self as #model_member>::try_mut_quant_enum(inner),
                                other => ::core::result::Result::Err(#error::IncorrectMember { expected: <Self as #member_trait>::KIND, actual: other.kind() }),
                            }
                        }
                    }
                }
            });

            quote! {
                #[doc = #model_member_doc]
                pub trait #model_member: #member_trait + #model_trait {
                    /// The quantization level of the model this is
                    const QUANT_KIND: #quant_kind;

                    /// Wraps data of this member
                    fn into_quant_enum<C: #container>(data: C::Of<Self>) -> #quant_enum<C>;

                    /// Unwraps the data held, if held for this member
                    fn try_from_quant_enum<C: #container>(value: #quant_enum<C>) -> ::core::result::Result<C::Of<Self>, #error>;

                    /// The data held, if held for this member
                    fn try_ref_quant_enum<C: #container>(value: &#quant_enum<C>) -> ::core::result::Result<&C::Of<Self>, #error>;

                    /// The data held, if held for this member
                    fn try_mut_quant_enum<C: #container>(value: &mut #quant_enum<C>) -> ::core::result::Result<&mut C::Of<Self>, #error>;
                }

                #( #impls )*
            }
        });

        quote! {
            #[doc = #member_doc]
            pub trait #member_trait: #base_trait + ::core::marker::Sized {
                /// The member of the family this is
                const KIND: #kind;

                /// The marker struct itself
                const MARKER: Self;

                /// Wraps data of this member
                fn into_family_enum<C: #container>(data: C::Of<Self>) -> #enum_name<C>;

                /// Unwraps the data held, if held for this member
                fn try_from_family_enum<C: #container>(value: #enum_name<C>) -> ::core::result::Result<C::Of<Self>, #error>;

                /// The data held, if held for this member
                fn try_ref_family_enum<C: #container>(value: &#enum_name<C>) -> ::core::result::Result<&C::Of<Self>, #error>;

                /// The data held, if held for this member
                fn try_mut_family_enum<C: #container>(value: &mut #enum_name<C>) -> ::core::result::Result<&mut C::Of<Self>, #error>;
            }

            #( #model_members )*
        }
    }

    //endregion

    //region Dispatch Macros

    /// `with_{family}!` over `{Family}Enum`, `with_{family}_{model}!` over each `{Model}QuantEnum`,
    /// and `with_{family}_kind!` over `{Family}Kind`. Names in the expansion resolve where it is
    /// called, so the family module must be in scope there, holding the generated items
    fn generate_with_macros(&self) -> proc_macro2::TokenStream {
        let family_module = self.family_module();
        let enum_name = self.enum_name();
        let kind = self.kind_name();
        let members = self.members();
        let member_models: Vec<&Ident> = members.iter().map(|(model, _)| *model).collect();
        let member_levels: Vec<&Ident> = members.iter().map(|(_, level)| *level).collect();
        let member_quant_enums: Vec<Ident> = member_models.iter().map(|model| Self::quant_enum_name(model)).collect();
        let member_quant_kinds: Vec<Ident> = member_models.iter().map(|model| Self::quant_kind_name(model)).collect();
        let member_markers: Vec<proc_macro2::TokenStream> = members.iter().map(|(model, level)| self.marker_path(model, level)).collect();

        let with_macro = self.with_macro_name();
        let with_doc = format!(
            "Runs the same code against whichever member of `{enum_name}` is held.\n\n\
             `{with_macro}!(value, |inner| body)` binds the data held to `inner`. \
             `{with_macro}!(value, <NM> |inner| body)` additionally aliases the marker struct of \
             the member held as `NM` within `body`. Requires the `{family_module}` module in scope"
        );

        let with_kind_macro = self.with_kind_macro_name();
        let with_kind_doc = format!(
            "Runs the same code against whichever member `{kind}` is.\n\n\
             `{with_kind_macro}!(kind, <NM> body)` aliases the marker struct of the member as `NM` \
             within `body`. Requires the `{family_module}` module in scope"
        );

        let model_macros = self.models.iter().map(|(model, template)| {
            let with_model_macro = self.with_model_macro_name(model);
            let quant_enum = Self::quant_enum_name(model);
            let levels: Vec<&Ident> = template.quantization_levels.keys().collect();
            let markers: Vec<proc_macro2::TokenStream> = levels.iter().map(|level| self.marker_path(model, level)).collect();
            let doc = format!(
                "Runs the same code against whichever quantization level of `{quant_enum}` is held.\n\n\
                 `{with_model_macro}!(value, |inner| body)` binds the data held to `inner`. \
                 `{with_model_macro}!(value, <NM> |inner| body)` additionally aliases the marker \
                 struct of the level held as `NM` within `body`. Requires the `{family_module}` \
                 module in scope"
            );
            quote! {
                #[doc = #doc]
                #[macro_export]
                macro_rules! #with_model_macro {
                    ($value:expr, |$inner:pat_param| $body:expr) => {
                        match $value {
                            #( #family_module::#quant_enum::#levels($inner) => $body, )*
                        }
                    };
                    ($value:expr, <$level:ident> |$inner:pat_param| $body:expr) => {
                        match $value {
                            #( #family_module::#quant_enum::#levels($inner) => {
                                #[allow(dead_code)]
                                type $level = #markers;
                                $body
                            } )*
                        }
                    };
                }
            }
        });

        quote! {
            #[doc = #with_doc]
            #[macro_export]
            macro_rules! #with_macro {
                ($value:expr, |$inner:pat_param| $body:expr) => {
                    match $value {
                        #( #family_module::#enum_name::#member_models(#family_module::#member_quant_enums::#member_levels($inner)) => $body, )*
                    }
                };
                ($value:expr, <$member:ident> |$inner:pat_param| $body:expr) => {
                    match $value {
                        #( #family_module::#enum_name::#member_models(#family_module::#member_quant_enums::#member_levels($inner)) => {
                            #[allow(dead_code)]
                            type $member = #member_markers;
                            $body
                        } )*
                    }
                };
            }

            #[doc = #with_kind_doc]
            #[macro_export]
            macro_rules! #with_kind_macro {
                ($kind:expr, <$member:ident> $body:expr) => {
                    match $kind {
                        #( #family_module::#kind::#member_models(#family_module::#member_quant_kinds::#member_levels) => {
                            #[allow(dead_code)]
                            type $member = #member_markers;
                            $body
                        } )*
                    }
                };
            }

            #( #model_macros )*
        }
    }

    //endregion
}

/// Serialize and Deserialize derives, with serde reached through `feagi_basis`
pub(crate) fn serde_derive() -> proc_macro2::TokenStream {
    let basis = basis_path();
    quote! {
        #[derive(#basis::__private::serde::Serialize, #basis::__private::serde::Deserialize)]
        #[serde(crate = #SERDE_CRATE)]
    }
}

/// Bounds a serde derive by every held type, so an enum is serializable exactly when the data it
/// may hold is, regardless of its container
pub(crate) fn serde_bounds(held: &[proc_macro2::TokenStream]) -> proc_macro2::TokenStream {
    let serialize = held.iter().map(|held| format!("{held}: {SERDE_CRATE}::Serialize")).collect::<Vec<_>>().join(", ");
    let deserialize = held.iter().map(|held| format!("{held}: {SERDE_CRATE}::Deserialize<'de>")).collect::<Vec<_>>().join(", ");
    quote!(#[serde(bound(serialize = #serialize, deserialize = #deserialize))])
}

/// Clone, Debug, and PartialEq of an enum generic over container `C`, each bound by every held
/// type implementing it, as derives would instead bound `C` itself. `container` is the trait `C`
/// is bound by
pub(crate) fn manual_enum_impls(
    enum_name: &Ident,
    container: &impl ToTokens,
    variants: &[&Ident],
    variant_names: &[String],
    held: &[proc_macro2::TokenStream],
) -> proc_macro2::TokenStream {
    quote! {
        impl<C: #container> ::core::clone::Clone for #enum_name<C>
        where
            #( #held: ::core::clone::Clone, )*
        {
            fn clone(&self) -> Self {
                match self {
                    #( Self::#variants(inner) => Self::#variants(::core::clone::Clone::clone(inner)), )*
                }
            }
        }

        impl<C: #container> ::core::fmt::Debug for #enum_name<C>
        where
            #( #held: ::core::fmt::Debug, )*
        {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                match self {
                    #( Self::#variants(inner) => f.debug_tuple(#variant_names).field(inner).finish(), )*
                }
            }
        }

        impl<C: #container> ::core::cmp::PartialEq for #enum_name<C>
        where
            #( #held: ::core::cmp::PartialEq, )*
        {
            #[allow(unreachable_patterns)]
            fn eq(&self, other: &Self) -> bool {
                match (self, other) {
                    #( (Self::#variants(a), Self::#variants(b)) => ::core::cmp::PartialEq::eq(a, b), )*
                    _ => false,
                }
            }
        }
    }
}

impl Parse for ModelFamilyTemplate {
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

        input.parse::<kw::Models>()?;
        input.parse::<Token![:]>()?;
        let models = parse_models(input)?;
        parse_optional_comma(input)?;

        if !input.is_empty() {
            return Err(input.error("unexpected input after the template, `Models` is the last section"));
        }

        Ok(Self {
            family_attributes,
            family_name,
            error_name,
            models,
        })
    }
}

/// Writes the template back out in the format it is parsed from
impl ToTokens for ModelFamilyTemplate {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let family_attributes = &self.family_attributes;
        let family_name = &self.family_name;
        let error_name = &self.error_name;

        let models = self.models.iter().map(|(model, template)| {
            let attributes = &template.attributes;
            let levels = template.quantization_levels.keys();
            let level_attributes = template.quantization_levels.values();
            quote! {
                #( #attributes )*
                #model: {
                    #(
                        #( #level_attributes )*
                        #levels,
                    )*
                }
            }
        });

        tokens.extend(quote! {
            #( #family_attributes )*
            Family: #family_name,
            Error: #error_name,

            Models: {
                #( #models )*
            }
        });
    }
}

/// `{ #[attr] Model: { ... } ... }`, the comma between models being optional
fn parse_models(input: ParseStream) -> syn::Result<IndexMap<Ident, ModelTemplate>> {
    let models_input;
    let brace = braced!(models_input in input);
    let mut models = IndexMap::new();
    while !models_input.is_empty() {
        let attributes = models_input.call(Attribute::parse_outer)?;
        let model = models_input.parse::<Ident>()?;
        models_input.parse::<Token![:]>()?;
        let quantization_levels = parse_quantization_levels(&models_input, &model)?;
        if models.contains_key(&model) {
            return Err(syn::Error::new(model.span(), format!("model `{model}` is defined more than once")));
        }
        models.insert(model, ModelTemplate { attributes, quantization_levels });
        parse_optional_comma(&models_input)?;
    }
    if models.is_empty() {
        return Err(syn::Error::new(brace.span.join(), "a model family needs at least one model"));
    }
    Ok(models)
}

/// `{ #[attr] Level, ... }`
fn parse_quantization_levels(input: ParseStream, model: &Ident) -> syn::Result<IndexMap<Ident, Vec<Attribute>>> {
    let levels_input;
    let brace = braced!(levels_input in input);
    let mut levels = IndexMap::new();
    while !levels_input.is_empty() {
        let attributes = levels_input.call(Attribute::parse_outer)?;
        let level = levels_input.parse::<Ident>()?;
        if levels.contains_key(&level) {
            return Err(syn::Error::new(level.span(), format!("quantization level `{level}` of model `{model}` is defined more than once")));
        }
        levels.insert(level, attributes);
        if levels_input.is_empty() {
            break;
        }
        levels_input.parse::<Token![,]>()?;
    }
    if levels.is_empty() {
        return Err(syn::Error::new(brace.span.join(), format!("model `{model}` needs at least one quantization level")));
    }
    Ok(levels)
}

//endregion
