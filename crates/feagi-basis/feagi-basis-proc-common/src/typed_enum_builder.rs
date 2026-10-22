use proc_macro::TokenStream;
use proc_macro2::Ident;
use quote::{format_ident, quote, TokenStreamExt};
use syn::parse::{Parse, ParseStream};
use syn::{parenthesized, parse, parse_macro_input, DeriveInput, Token};

pub fn enum_leveled_builder(input: TokenStream) -> TokenStream {
    let enum_level_params = parse_macro_input!(input as EnumLevelParams);

    let mut out = proc_macro2::TokenStream::new();

    out.append(enum_level_params.generate_family_trait().into());
    out.append(enum_level_params.generate_leveled_enum().into());

    out.into()
}

mod kw {
    //syn::custom_keyword!(path);
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
        let key: Ident = fields_input.parse()?;
        Ok( Self { short, key })
    }
}



struct EnumLevelParams {
    pub derives: Vec<Ident>,
    pub short_trait_name: Ident,
    pub base_trait_name: Ident,
    pub short_and_struct_key_names: Vec<ShortStructKeyNames>
}

impl EnumLevelParams {

    /// generate the family trait code
    pub fn generate_family_trait(&self) -> TokenStream {
        let family_trait_name = format_ident!("{}Family", self.base_trait_name.to_string());
        let base_trait_name = self.base_trait_name.clone();
        let short_trait_name = self.short_trait_name.clone();
        quote! {
            pub trait #family_trait_name {
                type Of<#short_trait_name: #base_trait_name>
            }
        }.into()
    }

    pub fn generate_leveled_enum(&self) -> TokenStream {

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

        quote!{
            pub enum #leveled_enum_name<#short_family_trait_name: #family_trait_name> {
                #rows
            }
        }.into()
    }
}

impl Parse for EnumLevelParams {
    fn parse(input: ParseStream) -> syn::Result<Self> {
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
        };
        Ok( Self {
            derives: Vec::new(),
            short_trait_name,
            base_trait_name,
            short_and_struct_key_names
        })
    }
}