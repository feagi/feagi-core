use proc_macro::TokenStream;
use proc_macro2::Ident;
use quote::{format_ident, quote};
use syn::parse::{Parse, ParseStream};
use syn::{parenthesized, parse_macro_input, Path, Token};
use feagi_macro_components::common::parse_optional_comma;

pub fn enum_leveled_builder(input: TokenStream) -> TokenStream {
    let enum_level_params = parse_macro_input!(input as EnumLevelParams);

    let mut out = proc_macro2::TokenStream::new();

    out.extend(enum_level_params.generate_family_trait());
    out.extend(enum_level_params.generate_leveled_enum());

    out.into()
}

struct ShortStructKeyNames {
    pub short: Ident,
    pub key: Ident
}

impl Parse for ShortStructKeyNames {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let fields_input;
        parenthesized!(fields_input in input);
        let short: Ident = fields_input.parse()?;
        parse_optional_comma(&fields_input)?;
        let key: Ident = fields_input.parse()?;
        Ok( Self { short, key })
    }
}

/// Std traits whose derive would also bound the family param itself (which is only a marker), so
/// they are implemented by hand on the enum instead
#[derive(Clone, Copy, PartialEq, Eq)]
enum ManualDerive {
    Clone,
    Copy,
    Debug,
    PartialEq,
    Eq,
    Hash,
}

impl ManualDerive {
    fn from_path(path: &Path) -> Option<Self> {
        Some(match path.segments.last()?.ident.to_string().as_str() {
            "Clone" => Self::Clone,
            "Copy" => Self::Copy,
            "Debug" => Self::Debug,
            "PartialEq" => Self::PartialEq,
            "Eq" => Self::Eq,
            "Hash" => Self::Hash,
            _ => return None,
        })
    }

    /// Fully qualified, since traits like `Debug` and `Hash` are not in the prelude
    fn trait_path(self) -> proc_macro2::TokenStream {
        match self {
            Self::Clone => quote!(::core::clone::Clone),
            Self::Copy => quote!(::core::marker::Copy),
            Self::Debug => quote!(::core::fmt::Debug),
            Self::PartialEq => quote!(::core::cmp::PartialEq),
            Self::Eq => quote!(::core::cmp::Eq),
            Self::Hash => quote!(::core::hash::Hash),
        }
    }
}

struct EnumLevelParams {
    pub derives: Vec<Path>,
    pub short_trait_name: Ident,
    pub base_trait_name: Ident,
    pub short_and_struct_key_names: Vec<ShortStructKeyNames>
}

impl EnumLevelParams {

    /// generate the family trait code. Every derive is enforced as a bound on `Of`, so the enum
    /// can implement them for any family
    pub fn generate_family_trait(&self) -> proc_macro2::TokenStream {
        let family_trait_name = format_ident!("{}Family", self.base_trait_name.to_string());
        let base_trait_name = self.base_trait_name.clone();
        let short_trait_name = self.short_trait_name.clone();

        let bounds: Vec<proc_macro2::TokenStream> = self.derives.iter().map(|derive| {
            if let Some(manual) = ManualDerive::from_path(derive) {
                return manual.trait_path();
            }
            // Deserialize carries a lifetime, the bound must hold for any of them
            match derive.segments.last() {
                Some(last) if last.ident == "Deserialize" => quote!(for<'de> #derive<'de>),
                _ => quote!(#derive),
            }
        }).collect();
        let bounds = if bounds.is_empty() { quote!() } else { quote!(: #(#bounds)+*) };

        quote! {
            pub trait #family_trait_name {
                type Of<#short_trait_name: #base_trait_name> #bounds;
            }
        }
    }

    pub fn generate_leveled_enum(&self) -> proc_macro2::TokenStream {

        fn make_row(key: &Ident, short_name: &Ident, struct_name: &Ident) -> proc_macro2::TokenStream {
            quote!{#key(#short_name::Of<#struct_name>),
            }
        }

        let family_trait_name = format_ident!("{}Family", self.base_trait_name.to_string());
        let leveled_enum_name = format_ident!("{}Leveled", self.base_trait_name.to_string());
        let short_family_trait_name = format_ident!("{}F", self.short_trait_name.to_string());

        let mut rows = proc_macro2::TokenStream::new();
        for row in &self.short_and_struct_key_names {
            rows.extend(make_row(&row.short, &short_family_trait_name, &row.key));
        };

        // Derives like serde's only bound the field types, so they can be passed through as is
        let passthrough_derives: Vec<&Path> = self.derives.iter()
            .filter(|derive| ManualDerive::from_path(derive).is_none())
            .collect();
        let derive_attr = if passthrough_derives.is_empty() {
            quote!()
        } else {
            quote!(#[derive(#(#passthrough_derives),*)])
        };

        let variants: Vec<&Ident> = self.short_and_struct_key_names.iter().map(|row| &row.short).collect();
        let mut manual_impls = proc_macro2::TokenStream::new();
        for manual in self.derives.iter().filter_map(ManualDerive::from_path) {
            manual_impls.extend(make_manual_impl(manual, &leveled_enum_name, &family_trait_name, &short_family_trait_name, &variants));
        }

        quote!{
            #derive_attr
            pub enum #leveled_enum_name<#short_family_trait_name: #family_trait_name> {
                #rows
            }

            #manual_impls
        }
    }
}

fn make_manual_impl(
    manual: ManualDerive,
    enum_name: &Ident,
    family_trait_name: &Ident,
    family_param: &Ident,
    variants: &[&Ident],
) -> proc_macro2::TokenStream {
    let trait_path = manual.trait_path();
    let header = quote!(impl<#family_param: #family_trait_name> #trait_path for #enum_name<#family_param>);

    match manual {
        ManualDerive::Clone => quote! {
            #header {
                fn clone(&self) -> Self {
                    match self {
                        #(Self::#variants(inner) => Self::#variants(::core::clone::Clone::clone(inner)),)*
                    }
                }
            }
        },
        ManualDerive::Copy | ManualDerive::Eq => quote!(#header {}),
        ManualDerive::Debug => {
            let names = variants.iter().map(|v| v.to_string());
            quote! {
                #header {
                    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                        match self {
                            #(Self::#variants(inner) => f.debug_tuple(#names).field(inner).finish(),)*
                        }
                    }
                }
            }
        }
        ManualDerive::PartialEq => quote! {
            #header {
                fn eq(&self, other: &Self) -> bool {
                    #[allow(unreachable_patterns)]
                    match (self, other) {
                        #((Self::#variants(a), Self::#variants(b)) => ::core::cmp::PartialEq::eq(a, b),)*
                        _ => false,
                    }
                }
            }
        },
        ManualDerive::Hash => quote! {
            #header {
                fn hash<H: ::core::hash::Hasher>(&self, state: &mut H) {
                    ::core::hash::Hash::hash(&::core::mem::discriminant(self), state);
                    match self {
                        #(Self::#variants(inner) => ::core::hash::Hash::hash(inner, state),)*
                    }
                }
            }
        },
    }
}

impl Parse for EnumLevelParams {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let derives_input;
        parenthesized!(derives_input in input);
        let mut derives = Vec::new();
        while !derives_input.is_empty() {
            derives.push(derives_input.parse::<Path>()?);
            parse_optional_comma(&derives_input)?;
        }
        input.parse::<Token![,]>()?;

        let base_trait_name: Ident = input.parse()?;
        input.parse::<Token![,]>()?;
        let short_trait_name: Ident = input.parse()?;
        input.parse::<Token![,]>()?;

        let fields_input;
        syn::braced!(fields_input in input);
        let mut short_and_struct_key_names = Vec::new();
        while !fields_input.is_empty() {
            let short_key = ShortStructKeyNames::parse(&fields_input)?;
            short_and_struct_key_names.push(short_key);
            parse_optional_comma(&fields_input)?;
        };
        parse_optional_comma(input)?;
        Ok( Self {
            derives,
            short_trait_name,
            base_trait_name,
            short_and_struct_key_names
        })
    }
}
