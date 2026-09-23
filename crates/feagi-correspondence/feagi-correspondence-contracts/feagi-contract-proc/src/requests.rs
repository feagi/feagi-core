use proc_macro::TokenStream;
use quote::{format_ident, quote};
use std::collections::HashSet;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::{braced, bracketed, parse_macro_input, Expr, ExprLit, ExprUnary, Ident, Lit, LitByteStr, LitInt, LitStr, Result, Token, UnOp};
use crate::struct_build::StructBuilderParameters;

mod kw {
    syn::custom_keyword!(request_response);

    syn::custom_keyword!(read);
    syn::custom_keyword!(create);
    syn::custom_keyword!(edit);
    syn::custom_keyword!(delete);
    syn::custom_keyword!(patch);

    syn::custom_keyword!(description);
    syn::custom_keyword!(request_parameters);
    syn::custom_keyword!(request);
    syn::custom_keyword!(response);
}

struct RootRequest {
    macro_name: Ident,
    root_path: Vec<LitStr>,
    categories: Vec<RequestCategories>
}

impl Parse for RootRequest {
    fn parse(input: ParseStream) -> Result<Self> {
        let macro_name: Ident = input.parse()?;
        input.parse::<Token![,]>()?;

        let root_path: Vec<LitStr>; // TODO the incoming lit string is a string separated "/" The string itself should only be alphanumeric characters, with "/" being what seperates elements.
        input.parse::<Token![,]>()?;

        input.parse::<kw::request_response>()?;

        let cat_input;
        braced!(cat_input in input);
        let mut categories: Vec<RequestCategories> = Vec::new();
        while !cat_input.is_empty() {
            categories.push(cat_input.parse::<RequestCategories>()?);
            let _ = cat_input.parse::<Token![,]>();
        }

        Ok( Self {
            macro_name, root_path, categories
        })

    }
}


struct RequestCategories {
    category_name: LitStr,
    read: RequestWithoutReqBody,
    create: RequestWithReqBody,
    edit: RequestWithReqBody,
    delete: RequestWithReqBody,
    patch: RequestWithReqBody,
}


/// A request to read / get state. Does not get a response body (but can have request parameters)
struct RequestWithoutReqBody {
    request_path: Vec<PathElement>,
    description: LitStr,
    request_parameters: URLEncodableStructBuilderParameters,
    response: StructBuilderParameters
}

/// Any other request that isnt a Read Request
struct RequestWithReqBody {
    request_path: Vec<PathElement>,
    description: LitStr,
    request_parameters: URLEncodableStructBuilderParameters,
    request: StructBuilderParameters,
    response: StructBuilderParameters
}

type URLEncodableStructBuilderParameters = StructBuilderParameters;

/// An element of a path that may be statically defined or represent a parameter
enum PathElement {
    /// A set path element name
    Static(LitStr),
    /// A parameter in the path itself
    Parameter(LitStr)
}

