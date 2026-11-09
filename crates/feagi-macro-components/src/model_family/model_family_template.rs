use crate::common::parse_optional_comma;
use heck::ToSnakeCase;
use indexmap::IndexMap;
use proc_macro2::Ident;
use quote::{format_ident, quote, ToTokens};
use syn::parse::{Parse, ParseStream};
use syn::{braced, Attribute, LitStr, Token};

mod kw {
    syn::custom_keyword!(Family);
    syn::custom_keyword!(Models);
}

//region Neuron Model Names

// The model family only ever holds the neuron models, so the names of its items are fixed, all
// prefixed `NM`. Every generator naming an item of a neuron model or of the family goes through
// these

/// The trait of neuron model `model`, `NM{Model}Trait`, such as `NMFeagiAdvancedTrait`
pub fn model_trait_name(model: &Ident) -> Ident {
    format_ident!("NM{}Trait", model)
}

/// The tag of neuron model `model`, `NM{Model}Tag`, naming it as a whole across its quantization
/// levels
pub fn model_tag_name(model: &Ident) -> Ident {
    format_ident!("NM{}Tag", model)
}

/// The marker struct of quantization level `level` of neuron model `model`,
/// `NM{Model}Quant{Level}`, such as `NMFeagiAdvancedQuantDefaultQuant`
pub fn quantization_level_struct_name(model: &Ident, level: &Ident) -> Ident {
    format_ident!("NM{}Quant{}", model, level)
}

/// The data struct of neuron model `model` for the `NeuronModel` data type `data_type`,
/// `NM{Model}Data{DataType}`, where the data type drops its `Data`, such as
/// `NMFeagiAdvancedDataCorticalProperties` for `CorticalDataProperties`
pub fn model_data_struct_name(model: &Ident, data_type: &Ident) -> Ident {
    format_ident!("NM{}Data{}", model, data_type.to_string().replacen("Data", "", 1))
}

/// The module of neuron model `model`, its snake case, such as `feagi_advanced`
pub fn model_module_name(model: &Ident) -> Ident {
    format_ident!("{}", model.to_string().to_snake_case())
}

/// `NMKind`, every member of the family without data
pub fn kind_name() -> Ident {
    format_ident!("NMKind")
}

/// `NM{Model}QuantKind`, the quantization levels of `model` without data
pub fn quant_kind_name(model: &Ident) -> Ident {
    format_ident!("NM{}QuantKind", model)
}

/// `NMContainer`, written by hand, deciding the data held per member
pub fn container_trait_name() -> Ident {
    format_ident!("NMContainer")
}

/// `NMEnum`, the data of any member of the family
pub fn enum_name() -> Ident {
    format_ident!("NMEnum")
}

/// `NM{Model}QuantEnum`, the data of any quantization level of `model`
pub fn quant_enum_name(model: &Ident) -> Ident {
    format_ident!("NM{}QuantEnum", model)
}

/// `NMMember`, declared by hand, implemented here for every marker struct
pub fn member_trait_name() -> Ident {
    format_ident!("NMMember")
}

/// `NM{Model}Member`, a member of `model`
pub fn model_member_trait_name(model: &Ident) -> Ident {
    format_ident!("NM{}Member", model)
}

/// `NMAccessError`, written by hand, of accessing data as a member or model it does not hold
pub fn error_name() -> Ident {
    format_ident!("NMAccessError")
}

/// `with_nm`, dispatching over `NMEnum`
pub fn with_macro_name() -> Ident {
    format_ident!("with_nm")
}

/// `with_nm_{model}`, dispatching over `NM{Model}QuantEnum`
pub fn with_model_macro_name(model: &Ident) -> Ident {
    format_ident!("with_nm_{}", model.to_string().to_snake_case())
}

/// The public module of the family in the crate expanding it, which holds the hand written and
/// generated items of the family, and a module per model named [`model_module_name`]. The family is
/// only ever expanded in `feagi_connectome`, so it is fixed
fn family_root() -> proc_macro2::TokenStream {
    quote!(crate::connectome_basis::neuron_model)
}

/// [`family_root`] from within an exported macro, which may be called from any crate
fn family_root_in_macro() -> proc_macro2::TokenStream {
    quote!($crate::connectome_basis::neuron_model)
}

//endregion

//region Model Family Template

/// The family of neuron models: a base trait, a sub trait of it per model, and a zero sized marker
/// struct per quantization level of each model, implementing the model's trait. Formatted as the
/// following:
/// ```text
/// /// Optional family description
/// Family: NeuronModel,
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
/// Expanded in the module defining the hand written items of the family: the base trait `{Family}`
/// in scope, and `NMContainer`, `NMMember`, and `NMAccessError` (see [`container_trait_name`]).
/// That module must be re-exported as [`family_root`], next to a module per model named its snake
/// case, holding the model trait `NM{Model}Trait`, the tag `NM{Model}Tag` (a type distinct per model
/// implementing the base trait), the marker struct `NM{Model}Quant{Level}` of each of its
/// quantization levels (see [`model_trait_name`]), and `BurstEngineDynamics`. Marker structs must
/// be unit structs. Every map keeps its template order
pub struct ModelFamilyTemplate {
    pub family_attributes: Vec<Attribute>,
    pub family_name: Ident,
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

    fn model_trait_path(model: &Ident) -> proc_macro2::TokenStream {
        let root = family_root();
        let model_module = model_module_name(model);
        let model_trait = model_trait_name(model);
        quote!(#root::#model_module::#model_trait)
    }

    fn tag_path(model: &Ident) -> proc_macro2::TokenStream {
        let root = family_root();
        let model_module = model_module_name(model);
        let tag = model_tag_name(model);
        quote!(#root::#model_module::#tag)
    }

    /// The marker struct of `level` of `model`, from [`family_root`] or, `in_macro`,
    /// [`family_root_in_macro`]
    fn marker_path(model: &Ident, level: &Ident, in_macro: bool) -> proc_macro2::TokenStream {
        let root = if in_macro { family_root_in_macro() } else { family_root() };
        let model_module = model_module_name(model);
        let marker = quantization_level_struct_name(model, level);
        quote!(#root::#model_module::#marker)
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

    /// Generate the kinds, enums, member trait implementations, and dispatch macros of the family
    pub fn generate_rust_family(&self) -> proc_macro2::TokenStream {
        let kinds = self.generate_kinds();
        let quant_enums = self.models.keys().map(|model| self.generate_quant_enum(model));
        let family_enum = self.generate_family_enum();
        let members = self.generate_member_traits();
        let with_macros = self.generate_with_macros();
        quote! {
            #kinds
            #( #quant_enums )*
            #family_enum
            #members
            #with_macros
        }
    }

    //region Kinds

    /// A data less `NM{Model}QuantKind` per model, with a variant per quantization level, and
    /// `NMKind`, with a variant per model holding its quantization kind
    fn generate_kinds(&self) -> proc_macro2::TokenStream {
        let family_name = &self.family_name;
        let kind = kind_name();
        let serde_derive = serde_derive();

        let quant_kinds = self.models.iter().map(|(model, template)| {
            let quant_kind = quant_kind_name(model);
            let doc = format!("The quantization levels of the `{model}` model of the `{family_name}` family");
            let levels: Vec<&Ident> = template.quantization_levels.keys().collect();
            let level_attributes = template.quantization_levels.values();
            let level_names = levels.iter().map(|level| level.to_string());
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
        let model_quant_kinds: Vec<Ident> = models.iter().map(|model| quant_kind_name(model)).collect();
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
            }

            impl ::core::fmt::Display for #kind {
                fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                    write!(f, "{}::{}", self.model_name(), self.quantization_name())
                }
            }
        }
    }

    //endregion

    //region Enums

    /// The `NM{Model}QuantEnum<C>` of `model`, with a variant holding the data of container `C` per
    /// quantization level
    fn generate_quant_enum(&self, model: &Ident) -> proc_macro2::TokenStream {
        let family_name = &self.family_name;
        let template = &self.models[model];
        let quant_enum = quant_enum_name(model);
        let quant_kind = quant_kind_name(model);
        let kind = kind_name();
        let container = container_trait_name();
        let model_member = model_member_trait_name(model);
        let error = error_name();

        let levels: Vec<&Ident> = template.quantization_levels.keys().collect();
        let level_attributes = template.quantization_levels.values();
        let level_names: Vec<String> = levels.iter().map(|level| level.to_string()).collect();
        let held: Vec<proc_macro2::TokenStream> = levels
            .iter()
            .map(|level| {
                let marker = Self::marker_path(model, level, false);
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
            }
        }
    }

    /// `NMEnum<C>`, with a variant holding the `NM{Model}QuantEnum<C>` per model, and the
    /// conversions between it and each of them
    fn generate_family_enum(&self) -> proc_macro2::TokenStream {
        let family_name = &self.family_name;
        let enum_name = enum_name();
        let kind = kind_name();
        let container = container_trait_name();
        let member_trait = member_trait_name();
        let error = error_name();

        let models: Vec<&Ident> = self.models.keys().collect();
        let model_attributes = self.models.values().map(|template| &template.attributes);
        let model_names: Vec<String> = models.iter().map(|model| model.to_string()).collect();
        let quant_enums: Vec<Ident> = models.iter().map(|model| quant_enum_name(model)).collect();
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

    /// A `NM{Model}Member` trait per model, implemented for every marker struct of the model along
    /// with the hand written `NMMember`, which convert data of the marker to and from the enums of
    /// the family
    fn generate_member_traits(&self) -> proc_macro2::TokenStream {
        let family_name = &self.family_name;
        let member_trait = member_trait_name();
        let kind = kind_name();
        let container = container_trait_name();
        let enum_name = enum_name();
        let error = error_name();

        let model_members = self.models.iter().map(|(model, template)| {
            let model_member = model_member_trait_name(model);
            let model_trait = Self::model_trait_path(model);
            let tag = Self::tag_path(model);
            let quant_kind = quant_kind_name(model);
            let quant_enum = quant_enum_name(model);
            let model_member_doc = format!(
                "A member of the `{model}` model of the `{family_name}` family. Converts data of any container of the family between itself and `{quant_enum}`"
            );

            let impls = template.quantization_levels.keys().map(|level| {
                let marker = Self::marker_path(model, level, false);
                quote! {
                    impl #model_member for #marker {
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
                        type Tag = #tag;
                        const KIND: #kind = #kind::#model(#quant_kind::#level);

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

        quote!(#( #model_members )*)
    }

    //endregion

    //region Dispatch Macros

    /// `with_nm!` over `NMEnum`, and `with_nm_{model}!` over each `NM{Model}QuantEnum`. Every path
    /// in the expansion starts at `$crate`, so they work from any crate without imports
    fn generate_with_macros(&self) -> proc_macro2::TokenStream {
        let root = family_root_in_macro();
        let enum_name = enum_name();
        let members = self.members();
        let member_models: Vec<&Ident> = members.iter().map(|(model, _)| *model).collect();
        let member_levels: Vec<&Ident> = members.iter().map(|(_, level)| *level).collect();
        let member_quant_enums: Vec<Ident> = member_models.iter().map(|model| quant_enum_name(model)).collect();
        let member_markers: Vec<proc_macro2::TokenStream> = members.iter().map(|(model, level)| Self::marker_path(model, level, true)).collect();
        let member_model_modules: Vec<Ident> = member_models.iter().map(|model| model_module_name(model)).collect();

        let with_macro = with_macro_name();
        let with_doc = format!(
            "Runs the same code against whichever member of `{enum_name}` is held.\n\n\
             `{with_macro}!(value, |inner| body)` binds the data held to `inner`. \
             `{with_macro}!(value, <NM> |inner| body)` additionally aliases the marker struct of \
             the member held as `NM` within `body`. `{with_macro}!(value, <NM, Model> |inner| body)` \
             further aliases the model module of the member held as `Model` within `body`"
        );

        let model_macros = self.models.iter().map(|(model, template)| {
            let with_model_macro = with_model_macro_name(model);
            let quant_enum = quant_enum_name(model);
            let levels: Vec<&Ident> = template.quantization_levels.keys().collect();
            let markers: Vec<proc_macro2::TokenStream> = levels.iter().map(|level| Self::marker_path(model, level, true)).collect();
            let model_module = model_module_name(model);
            let doc = format!(
                "Runs the same code against whichever quantization level of `{quant_enum}` is held.\n\n\
                 `{with_model_macro}!(value, |inner| body)` binds the data held to `inner`. \
                 `{with_model_macro}!(value, <NM> |inner| body)` additionally aliases the marker \
                 struct of the level held as `NM` within `body`. `{with_model_macro}!(value, \
                 <NM, Model> |inner| body)` further aliases the `{model_module}` model module as \
                 `Model` within `body`"
            );
            quote! {
                #[doc = #doc]
                #[macro_export]
                macro_rules! #with_model_macro {
                    ($value:expr, |$inner:pat_param| $body:expr) => {
                        match $value {
                            #( #root::#quant_enum::#levels($inner) => $body, )*
                        }
                    };
                    ($value:expr, <$level:ident> |$inner:pat_param| $body:expr) => {
                        match $value {
                            #( #root::#quant_enum::#levels($inner) => {
                                #[allow(dead_code)]
                                type $level = #markers;
                                $body
                            } )*
                        }
                    };
                    ($value:expr, <$level:ident, $model:ident> |$inner:pat_param| $body:expr) => {
                        match $value {
                            #( #root::#quant_enum::#levels($inner) => {
                                #[allow(dead_code)]
                                type $level = #markers;
                                #[allow(unused_imports)]
                                use #root::#model_module as $model;
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
                        #( #root::#enum_name::#member_models(#root::#member_quant_enums::#member_levels($inner)) => $body, )*
                    }
                };
                ($value:expr, <$member:ident> |$inner:pat_param| $body:expr) => {
                    match $value {
                        #( #root::#enum_name::#member_models(#root::#member_quant_enums::#member_levels($inner)) => {
                            #[allow(dead_code)]
                            type $member = #member_markers;
                            $body
                        } )*
                    }
                };
                ($value:expr, <$member:ident, $model:ident> |$inner:pat_param| $body:expr) => {
                    match $value {
                        #( #root::#enum_name::#member_models(#root::#member_quant_enums::#member_levels($inner)) => {
                            #[allow(dead_code)]
                            type $member = #member_markers;
                            #[allow(unused_imports)]
                            use #root::#member_model_modules as $model;
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

/// Serialize and Deserialize derives. The crate expanding them must depend on serde
pub(crate) fn serde_derive() -> proc_macro2::TokenStream {
    quote! {
        #[derive(::serde::Serialize, ::serde::Deserialize)]
    }
}

/// Bounds a serde derive by every held type, so an enum is serializable exactly when the data it
/// may hold is, regardless of its container
pub(crate) fn serde_bounds(held: &[proc_macro2::TokenStream]) -> proc_macro2::TokenStream {
    let serialize = held.iter().map(|held| format!("{held}: ::serde::Serialize")).collect::<Vec<_>>().join(", ");
    let deserialize = held.iter().map(|held| format!("{held}: ::serde::Deserialize<'de>")).collect::<Vec<_>>().join(", ");
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
            models,
        })
    }
}

/// Writes the template back out in the format it is parsed from
impl ToTokens for ModelFamilyTemplate {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let family_attributes = &self.family_attributes;
        let family_name = &self.family_name;

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
