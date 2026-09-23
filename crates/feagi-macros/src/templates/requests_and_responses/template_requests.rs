use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::{braced, parse_macro_input, LitStr, Token};

use crate::common::parse_optional_comma;
use crate::templates::requests_and_responses::template_requests_category::TemplateRequestCategory;

mod kw {
    syn::custom_keyword!(categories);
}

pub fn template_requests(input: TokenStream) -> TokenStream {
    let request = parse_macro_input!(input as TemplateRequest);

    // TODO validations

    TokenStream::from(request.expand_macro())
}

//region Template Request

/// All request / responses of a certain category
pub struct TemplateRequest {
    pub macro_name: syn::Ident,
    pub root_path: Vec<LitStr>,
    pub categories: Vec<TemplateRequestCategory>,
}

impl Parse for TemplateRequest {
    fn parse(input: ParseStream) -> syn::Result<Self> {

        let macro_name: syn::Ident = input.parse()?;
        input.parse::<Token![,]>()?;

        let root_lit: LitStr = input.parse()?;
        let root_path: Vec<LitStr> = root_lit
            .value()
            .split('/')
            .filter(|segment| !segment.is_empty())
            .map(|segment| LitStr::new(segment, root_lit.span()))
            .collect();

        input.parse::<Token![,]>()?;
        input.parse::<kw::categories>()?;

        let cat_input;
        braced!(cat_input in input);
        let mut categories: Vec<TemplateRequestCategory> = Vec::new();
        while !cat_input.is_empty() {
            categories.push(cat_input.parse::<TemplateRequestCategory>()?);
            parse_optional_comma(&cat_input)?;
        }

        Ok(Self {
            macro_name,
            root_path,
            categories,
        })
    }
}

impl TemplateRequest {
    /// Emits per-category `macro_rules!` helpers plus the root `macro_rules! #macro_name`.
    pub fn expand_macro(&self) -> proc_macro2::TokenStream {
        let category_macros = self.categories.iter().map(TemplateRequestCategory::expand_macro);
        let macro_name = &self.macro_name;
        let template_body = self.expand_template();

        let root_macro = quote! {
            macro_rules! #macro_name {
                () => {
                    #template_body
                };
            }
        };

        quote! {
            #(#category_macros)*
            #root_macro
        }
    }

    /// Emits `#macro_name, "v2", categories { template_request_category_*!(), ... }`.
    pub fn expand_template(&self) -> proc_macro2::TokenStream {
        let macro_name = &self.macro_name;
        let root_path = self.root_path_lit();
        let category_invocations = self.categories.iter().map(|category| {
            let category_macro_name = &category.category_macro_name;
            quote! {
                #category_macro_name!()
            }
        });

        quote! {
            #macro_name,
            #root_path,
            categories {
                #(#category_invocations,)*
            }
        }
    }

    fn root_path_lit(&self) -> LitStr {
        let span = self
            .root_path
            .first()
            .map(LitStr::span)
            .unwrap_or(Span::call_site());

        let path = self
            .root_path
            .iter()
            .map(LitStr::value)
            .collect::<Vec<_>>()
            .join("/");

        LitStr::new(&path, span)
    }
}

//endregion