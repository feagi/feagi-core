use syn::{braced, parse_macro_input, LitStr, Token};
use syn::parse::{Parse, ParseStream};
use proc_macro::TokenStream;
use crate::common::parse_optional_comma;
use crate::templates::requests_and_responses::template_requests_category::{TemplateRequestCategory};

mod kw {
    syn::custom_keyword!(categories);
}

pub fn template_requests(input: TokenStream) -> TokenStream {
    let request_category = parse_macro_input!(input as TemplateRequest);
    
    // TODO Generate new macro

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

//endregion