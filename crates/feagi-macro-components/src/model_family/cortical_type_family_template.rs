use super::model_family_template::{manual_enum_impls, serde_bounds, serde_derive};
use crate::common::parse_optional_comma;
use heck::ToSnakeCase;
use indexmap::IndexMap;
use proc_macro2::Ident;
use quote::{format_ident, quote, ToTokens};
use syn::parse::{Parse, ParseStream};
use syn::{braced, Attribute, LitStr, Token};

mod kw {
    syn::custom_keyword!(Family);
    syn::custom_keyword!(Error);
    syn::custom_keyword!(ModelFamily);
    syn::custom_keyword!(ModelFamilyError);
    syn::custom_keyword!(Types);
}

//region Cortical Type Family Template

/// A family of cortical area types: a base trait generic over a member of a model family, and a
/// struct per cortical type generic over the members of the single model it uses, implementing
/// the base trait. Cortical types may be grouped into super types, to any depth. Formatted as the
/// following:
/// ```text
/// /// Optional family description
/// Family: CorticalAreaType,
/// Error: CorticalAreaTypeSelectionError,
/// ModelFamily: NeuronModel,
/// ModelFamilyError: NeuronModelSelectionError,
///
/// Types: {
///     /// Optional super type description
///     Foundation: {
///         /// Optional cortical type description
///         Power: FoundationPower,
///     }
///     Interconnect: FeagiAdvanced,
/// }
/// ```
/// Each cortical type names the model of the model family it uses. No paths are given. Generated
/// code instead expects the modules of both families in scope where it is expanded: the module of
/// the model family, as its own template expects, and a module named the snake case of this
/// family, holding the base trait `{Family}` and the items generated for this family, with nested
/// sub modules per super type and cortical type named their snake case. The module of a cortical
/// type holds its struct, `{Family}{Path}`, `{Path}` being the names of its super types and itself,
/// such as `cortical_area_type::foundation::power::CorticalAreaTypeFoundationPower<NM>`. Every map
/// keeps its template order
pub struct CorticalTypeFamilyTemplate {
    pub family_attributes: Vec<Attribute>,
    pub family_name: Ident,
    pub error_name: Ident,
    pub model_family_name: Ident,
    pub model_family_error_name: Ident,
    pub types: IndexMap<Ident, CorticalTypeEntry>,
}

/// A super type or cortical type of the family
pub struct CorticalTypeEntry {
    pub attributes: Vec<Attribute>,
    pub kind: CorticalTypeEntryKind,
}

pub enum CorticalTypeEntryKind {
    /// Groups further super types and cortical types
    SuperType(IndexMap<Ident, CorticalTypeEntry>),
    /// A cortical type, using the named model of the model family
    CorticalType(Ident),
}

/// A cortical type, by the names of its super types and itself, and the model it uses
struct CorticalTypeLeaf<'a> {
    path: Vec<&'a Ident>,
    model: &'a Ident,
}

impl CorticalTypeFamilyTemplate {
    //region Names

    fn family_snake_case(&self) -> String {
        self.family_name.to_string().to_snake_case()
    }

    pub fn template_macro_name(&self) -> Ident {
        format_ident!("{}_family_template", self.family_snake_case())
    }

    /// The module of this family, expected in scope wherever generated code is expanded
    pub fn family_module(&self) -> Ident {
        format_ident!("{}", self.family_snake_case())
    }

    /// The module of the model family, expected in scope wherever generated code is expanded
    pub fn model_family_module(&self) -> Ident {
        format_ident!("{}", self.model_family_name.to_string().to_snake_case())
    }

    /// The kind of the super type at `path`, or of the family for an empty path
    fn kind_name(&self, path: &[&Ident]) -> Ident {
        match path.is_empty() {
            true => format_ident!("{}Kind", self.family_name),
            false => format_ident!("{}SuperTypeKind", concat_path(path)),
        }
    }

    /// The data enum of the super type at `path`, or of the family for an empty path
    fn enum_name(&self, path: &[&Ident]) -> Ident {
        match path.is_empty() {
            true => format_ident!("{}Enum", self.family_name),
            false => format_ident!("{}SuperTypeEnum", concat_path(path)),
        }
    }

    pub fn member_trait_name(&self) -> Ident {
        format_ident!("{}Member", self.family_name)
    }

    pub fn with_macro_name(&self) -> Ident {
        format_ident!("with_{}", self.family_snake_case())
    }

    /// The method of every data enum returning the model family kind held
    fn model_kind_method(&self) -> Ident {
        format_ident!("{}_kind", self.model_family_name.to_string().to_snake_case())
    }

    /// The struct of a cortical type, `{family}::{path...}::{Family}{Path}`, without generics
    fn cortical_type_path(&self, path: &[&Ident]) -> proc_macro2::TokenStream {
        let family_module = self.family_module();
        let modules = path.iter().map(|name| format_ident!("{}", name.to_string().to_snake_case()));
        let struct_name = format_ident!("{}{}", self.family_name, concat_path(path));
        quote!(#family_module #( ::#modules )* ::#struct_name)
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
    fn cortical_types(&self) -> Vec<CorticalTypeLeaf<'_>> {
        fn collect<'a>(entries: &'a IndexMap<Ident, CorticalTypeEntry>, path: &mut Vec<&'a Ident>, out: &mut Vec<CorticalTypeLeaf<'a>>) {
            for (name, entry) in entries {
                path.push(name);
                match &entry.kind {
                    CorticalTypeEntryKind::SuperType(children) => collect(children, path, out),
                    CorticalTypeEntryKind::CorticalType(model) => out.push(CorticalTypeLeaf { path: path.clone(), model }),
                }
                path.pop();
            }
        }
        let mut out = Vec::new();
        collect(&self.types, &mut Vec::new(), &mut out);
        out
    }

    /// The kind of the cortical type at `relative` below the super type at `base`, as a value of
    /// the kind of `base`. Each kind is prefixed by `qualifier`
    fn kind_value(&self, base: &[&Ident], relative: &[&Ident], qualifier: &proc_macro2::TokenStream) -> proc_macro2::TokenStream {
        let kind = self.kind_name(base);
        let (first, rest) = relative.split_first().expect("a cortical type has a name");
        if rest.is_empty() {
            return quote!(#qualifier #kind::#first);
        }
        let mut deeper = base.to_vec();
        deeper.push(first);
        let inner = self.kind_value(&deeper, rest, qualifier);
        quote!(#qualifier #kind::#first(#inner))
    }

    /// The variant of the cortical type at `relative` below the super type at `base`, as a value
    /// or pattern of the data enum of `base`, wrapping `inner`. Each enum is prefixed by `qualifier`
    fn enum_value(
        &self,
        base: &[&Ident],
        relative: &[&Ident],
        inner: &proc_macro2::TokenStream,
        qualifier: &proc_macro2::TokenStream,
    ) -> proc_macro2::TokenStream {
        let enum_name = self.enum_name(base);
        let (first, rest) = relative.split_first().expect("a cortical type has a name");
        if rest.is_empty() {
            return quote!(#qualifier #enum_name::#first(#inner));
        }
        let mut deeper = base.to_vec();
        deeper.push(first);
        let wrapped = self.enum_value(&deeper, rest, inner, qualifier);
        quote!(#qualifier #enum_name::#first(#wrapped))
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

    /// Generate the kinds, error, data enums, member trait, and dispatch macro of the family
    pub fn generate_rust_family(&self) -> proc_macro2::TokenStream {
        let error = self.generate_error();
        let groups = self.generate_super_type(&[], &self.types);
        let member = self.generate_member_trait();
        let with_macro = self.generate_with_macro();
        quote! {
            #error
            #groups
            #member
            #with_macro
        }
    }

    //region Error

    fn generate_error(&self) -> proc_macro2::TokenStream {
        let family_name = &self.family_name;
        let error = &self.error_name;
        let kind = self.kind_name(&[]);
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
                /// The cortical type held is of a different member of its model
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

    //region Super Types

    /// The kind and data enum of the super type at `path` (the family for an empty path), holding
    /// `entries`, and those of every super type below it
    fn generate_super_type(&self, path: &[&Ident], entries: &IndexMap<Ident, CorticalTypeEntry>) -> proc_macro2::TokenStream {
        let family_name = &self.family_name;
        let kind = self.kind_name(path);
        let enum_name = self.enum_name(path);
        let container = self.model_container_path();
        let model_kind = self.model_kind_path();
        let model_kind_method = self.model_kind_method();
        let error = &self.error_name;
        let serde_derive = serde_derive();
        let unqualified = quote!();
        let described = match path.is_empty() {
            true => format!("the `{family_name}` family"),
            false => format!("the `{}` super type of the `{family_name}` family", concat_path(path)),
        };

        let names: Vec<&Ident> = entries.keys().collect();
        let name_strings: Vec<String> = names.iter().map(|name| name.to_string()).collect();
        let attributes: Vec<&Vec<Attribute>> = entries.values().map(|entry| &entry.attributes).collect();

        // Per entry, its kind variant, data held, and how kinds of data held are found
        let mut kind_variants = Vec::new();
        let mut held = Vec::new();
        let mut kind_arms = Vec::new();
        let mut model_kind_arms = Vec::new();
        let mut display_arms = Vec::new();
        let mut children = Vec::new();
        let mut super_type_conversions = Vec::new();
        for (name, entry) in entries {
            let name_string = name.to_string();
            match &entry.kind {
                CorticalTypeEntryKind::CorticalType(model) => {
                    let quant_enum = self.model_quant_enum_path(model);
                    kind_variants.push(quote!(#name));
                    held.push(quote!(#quant_enum<C>));
                    kind_arms.push(quote!(Self::#name(_) => #kind::#name));
                    model_kind_arms.push(quote!(Self::#name(inner) => inner.kind()));
                    display_arms.push(quote!(Self::#name => f.write_str(#name_string)));
                }
                CorticalTypeEntryKind::SuperType(child_entries) => {
                    let mut child_path = path.to_vec();
                    child_path.push(name);
                    let child_kind = self.kind_name(&child_path);
                    let child_enum = self.enum_name(&child_path);
                    let child_held = quote!(#child_enum<C>);
                    kind_variants.push(quote!(#name(#child_kind)));
                    kind_arms.push(quote!(Self::#name(inner) => #kind::#name(inner.kind())));
                    model_kind_arms.push(quote!(Self::#name(inner) => inner.#model_kind_method()));
                    display_arms.push(quote!(Self::#name(inner) => write!(f, "{}::{}", #name_string, inner)));
                    super_type_conversions.push(quote! {
                        impl<C: #container> ::core::convert::From<#child_held> for #enum_name<C> {
                            fn from(value: #child_held) -> Self {
                                Self::#name(value)
                            }
                        }

                        impl<C: #container> ::core::convert::TryFrom<#enum_name<C>> for #child_held {
                            type Error = #error;

                            #[allow(unreachable_patterns)]
                            fn try_from(value: #enum_name<C>) -> ::core::result::Result<Self, Self::Error> {
                                match value {
                                    #enum_name::#name(inner) => ::core::result::Result::Ok(inner),
                                    other => ::core::result::Result::Err(#error::IncorrectSuperType { expected: #name_string, actual: other.kind().name() }),
                                }
                            }
                        }
                    });
                    held.push(child_held);
                    children.push(self.generate_super_type(&child_path, child_entries));
                }
            }
        }

        // Every cortical type below, as a kind of this
        let leaves: Vec<CorticalTypeLeaf> = self.cortical_types().into_iter().filter(|leaf| leaf.path.starts_with(path)).collect();
        let leaf_count = leaves.len();
        let all_kinds = leaves.iter().map(|leaf| self.kind_value(path, &leaf.path[path.len()..], &unqualified));

        let manual_impls = manual_enum_impls(&enum_name, &container, &names, &name_strings, &held);
        let serde_bounds = serde_bounds(&held);
        let kind_doc = format!("Every cortical type of {described}, without any data");
        let enum_doc = format!("Data of container `C`, for any cortical type of {described} at any member of its model");

        quote! {
            #[doc = #kind_doc]
            #[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
            #serde_derive
            pub enum #kind {
                #(
                    #( #attributes )*
                    #kind_variants,
                )*
            }

            impl #kind {
                /// Every cortical type below, in template order
                pub const ALL: [Self; #leaf_count] = [#( #all_kinds ),*];

                /// The name of the cortical type or super type directly below
                pub const fn name(&self) -> &'static str {
                    match self {
                        #( Self::#names { .. } => #name_strings, )*
                    }
                }
            }

            impl ::core::fmt::Display for #kind {
                fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                    match self {
                        #( #display_arms, )*
                    }
                }
            }

            #[doc = #enum_doc]
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
                        #( #kind_arms, )*
                    }
                }

                /// The member of the model family held
                pub const fn #model_kind_method(&self) -> #model_kind {
                    match self {
                        #( #model_kind_arms, )*
                    }
                }
            }

            #( #super_type_conversions )*

            #( #children )*
        }
    }

    //endregion

    //region Member

    /// The `{Family}Member` trait, implemented for every cortical type at every member of its
    /// model, which converts data of a cortical type to and from `{Family}Enum`, and the typed
    /// access of `{Family}Enum` through it
    fn generate_member_trait(&self) -> proc_macro2::TokenStream {
        let family_name = &self.family_name;
        let member_trait = self.member_trait_name();
        let model_family_name = &self.model_family_name;
        let model_member = self.model_member_path();
        let container = self.model_container_path();
        let kind = self.kind_name(&[]);
        let enum_name = self.enum_name(&[]);
        let error = &self.error_name;
        let unqualified = quote!();
        let data = quote!(data);
        let doc = format!(
            "A cortical type of the `{family_name}` family at a member of its model. Converts data of any container of the model family between itself and `{enum_name}`"
        );

        let impls = self.cortical_types().into_iter().map(|leaf| {
            let cortical_type = self.cortical_type_path(&leaf.path);
            let model_model_member = self.model_model_member_path(leaf.model);
            let kind_value = self.kind_value(&[], &leaf.path, &unqualified);
            let wrap = self.enum_value(&[], &leaf.path, &quote!(<M as #model_model_member>::into_quant_enum(data)), &unqualified);
            let pattern = self.enum_value(&[], &leaf.path, &data, &unqualified);
            let mismatch = quote!(::core::result::Result::Err(#error::IncorrectCorticalType { expected: <Self as #member_trait>::KIND, actual: other.kind() }));
            quote! {
                impl<M: #model_model_member> #member_trait for #cortical_type<M> {
                    type #model_family_name = M;
                    const KIND: #kind = #kind_value;

                    fn into_family_enum<C: #container>(data: C::Of<M>) -> #enum_name<C> {
                        #wrap
                    }

                    #[allow(unreachable_patterns)]
                    fn try_from_family_enum<C: #container>(value: #enum_name<C>) -> ::core::result::Result<C::Of<M>, #error> {
                        match value {
                            #pattern => <M as #model_model_member>::try_from_quant_enum(#data).map_err(#error::from),
                            other => #mismatch,
                        }
                    }

                    #[allow(unreachable_patterns)]
                    fn try_ref_family_enum<C: #container>(value: &#enum_name<C>) -> ::core::result::Result<&C::Of<M>, #error> {
                        match value {
                            #pattern => <M as #model_model_member>::try_ref_quant_enum(#data).map_err(#error::from),
                            other => #mismatch,
                        }
                    }

                    #[allow(unreachable_patterns)]
                    fn try_mut_family_enum<C: #container>(value: &mut #enum_name<C>) -> ::core::result::Result<&mut C::Of<M>, #error> {
                        match value {
                            #pattern => <M as #model_model_member>::try_mut_quant_enum(#data).map_err(#error::from),
                            other => #mismatch,
                        }
                    }
                }
            }
        });

        let of_member = quote!(C::Of<<T as #member_trait>::#model_family_name>);
        quote! {
            #[doc = #doc]
            pub trait #member_trait {
                /// The member of the model family the cortical type is at
                type #model_family_name: #model_member;

                /// The cortical type this is
                const KIND: #kind;

                /// Wraps data of this cortical type
                fn into_family_enum<C: #container>(data: C::Of<Self::#model_family_name>) -> #enum_name<C>;

                /// Unwraps the data held, if held for this cortical type and member
                fn try_from_family_enum<C: #container>(value: #enum_name<C>) -> ::core::result::Result<C::Of<Self::#model_family_name>, #error>;

                /// The data held, if held for this cortical type and member
                fn try_ref_family_enum<C: #container>(value: &#enum_name<C>) -> ::core::result::Result<&C::Of<Self::#model_family_name>, #error>;

                /// The data held, if held for this cortical type and member
                fn try_mut_family_enum<C: #container>(value: &mut #enum_name<C>) -> ::core::result::Result<&mut C::Of<Self::#model_family_name>, #error>;
            }

            #( #impls )*

            impl<C: #container> #enum_name<C> {
                /// Wraps the data of cortical type `T`
                pub fn from_data<T: #member_trait>(data: #of_member) -> Self {
                    T::into_family_enum(data)
                }

                /// The data held, if held for cortical type `T`
                pub fn try_as<T: #member_trait>(&self) -> ::core::result::Result<&#of_member, #error> {
                    T::try_ref_family_enum(self)
                }

                /// The data held, if held for cortical type `T`
                pub fn try_as_mut<T: #member_trait>(&mut self) -> ::core::result::Result<&mut #of_member, #error> {
                    T::try_mut_family_enum(self)
                }

                /// Unwraps the data held, if held for cortical type `T`
                pub fn try_into_data<T: #member_trait>(self) -> ::core::result::Result<#of_member, #error> {
                    T::try_from_family_enum(self)
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
        let enum_name = self.enum_name(&[]);
        let model_family_snake = self.model_family_name.to_string().to_snake_case();
        let inner = quote!(__cortical_type_inner);

        let leaves = self.cortical_types();
        let patterns: Vec<proc_macro2::TokenStream> = leaves.iter().map(|leaf| self.enum_value(&[], &leaf.path, &inner, &qualified)).collect();
        let cortical_types: Vec<proc_macro2::TokenStream> = leaves.iter().map(|leaf| self.cortical_type_path(&leaf.path)).collect();
        let model_macros: Vec<Ident> = leaves
            .iter()
            .map(|leaf| format_ident!("with_{}_{}", model_family_snake, leaf.model.to_string().to_snake_case()))
            .collect();
        let model_macro_names = model_macros.iter().map(|name| format!("`{name}!`")).collect::<Vec<_>>().join(", ");

        let doc = format!(
            "Runs the same code against whichever cortical type of `{enum_name}` is held.\n\n\
             `{with_macro}!(value, |inner| body)` binds the model data enum held to `inner`. \
             `{with_macro}!(value, <T, NM> |data| body)` binds the data held to `data`, and aliases \
             the cortical type struct held as `T` and the member of its model as `NM` within \
             `body`. That form calls the model dispatch macros {model_macro_names}, which must be \
             in scope. Requires the `{family_module}` and model family modules in scope"
        );

        quote! {
            #[doc = #doc]
            #[macro_export]
            macro_rules! #with_macro {
                ($value:expr, |$inner:pat_param| $body:expr) => {
                    match $value {
                        #( #patterns => { let $inner = #inner; $body } )*
                    }
                };
                ($value:expr, <$cortical_type:ident, $member:ident> |$data:pat_param| $body:expr) => {
                    match $value {
                        #( #patterns => #model_macros!(#inner, <$member> |$data| {
                            #[allow(dead_code)]
                            type $cortical_type = #cortical_types<$member>;
                            $body
                        }), )*
                    }
                };
            }
        }
    }

    //endregion
}

/// The names of a path concatenated, such as `FoundationPower`
fn concat_path(path: &[&Ident]) -> String {
    path.iter().map(|name| name.to_string()).collect()
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

        input.parse::<kw::Types>()?;
        input.parse::<Token![:]>()?;
        let types = parse_entries(input, None)?;
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
            types,
        })
    }
}

/// Writes the template back out in the format it is parsed from
impl ToTokens for CorticalTypeFamilyTemplate {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        fn entries_tokens(entries: &IndexMap<Ident, CorticalTypeEntry>) -> proc_macro2::TokenStream {
            let entries = entries.iter().map(|(name, entry)| {
                let attributes = &entry.attributes;
                let value = match &entry.kind {
                    CorticalTypeEntryKind::SuperType(children) => entries_tokens(children),
                    CorticalTypeEntryKind::CorticalType(model) => quote!(#model,),
                };
                quote! {
                    #( #attributes )*
                    #name: #value
                }
            });
            quote!({ #( #entries )* })
        }

        let family_attributes = &self.family_attributes;
        let family_name = &self.family_name;
        let error_name = &self.error_name;
        let model_family_name = &self.model_family_name;
        let model_family_error_name = &self.model_family_error_name;
        let types = entries_tokens(&self.types);
        tokens.extend(quote! {
            #( #family_attributes )*
            Family: #family_name,
            Error: #error_name,
            ModelFamily: #model_family_name,
            ModelFamilyError: #model_family_error_name,

            Types: #types
        });
    }
}

/// `{ #[attr] SuperType: { ... } #[attr] CorticalType: Model, ... }`, the commas being optional.
/// `super_type` is the super type being parsed, if not the family
fn parse_entries(input: ParseStream, super_type: Option<&Ident>) -> syn::Result<IndexMap<Ident, CorticalTypeEntry>> {
    let entries_input;
    let brace = braced!(entries_input in input);
    let mut entries = IndexMap::new();
    while !entries_input.is_empty() {
        let attributes = entries_input.call(Attribute::parse_outer)?;
        let name = entries_input.parse::<Ident>()?;
        entries_input.parse::<Token![:]>()?;
        let kind = if entries_input.peek(syn::token::Brace) {
            CorticalTypeEntryKind::SuperType(parse_entries(&entries_input, Some(&name))?)
        } else {
            CorticalTypeEntryKind::CorticalType(entries_input.parse::<Ident>()?)
        };
        if entries.contains_key(&name) {
            return Err(syn::Error::new(name.span(), format!("`{name}` is defined more than once")));
        }
        entries.insert(name, CorticalTypeEntry { attributes, kind });
        parse_optional_comma(&entries_input)?;
    }
    if entries.is_empty() {
        let message = match super_type {
            Some(super_type) => format!("super type `{super_type}` needs at least one cortical type or super type"),
            None => String::from("a cortical type family needs at least one cortical type or super type"),
        };
        return Err(syn::Error::new(brace.span.join(), message));
    }
    Ok(entries)
}

//endregion
