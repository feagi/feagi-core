use proc_macro::TokenStream;
use quote::{format_ident, quote, IdentFragment};
use std::collections::HashSet;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::{braced, bracketed, parse_macro_input, Expr, ExprLit, ExprUnary, Ident, Lit, LitByteStr, LitInt, LitStr, Result, Token, UnOp};
use crate::struct_build::StructBuilderParameters;
use crate::common::parse_optional_comma;

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

//region Request

struct RootRequest {
    macro_name: Ident,
    root_path: Vec<LitStr>,
    categories: Vec<RequestCategories>
}

impl Parse for RootRequest {
    fn parse(input: ParseStream) -> Result<Self> {
        let macro_name: Ident = input.parse()?;
        input.parse::<Token![,]>()?;

        let root_lit: LitStr = input.parse()?;
        let root_path: Vec<LitStr> = root_lit
            .value()
            .split('/')
            .filter(|segment| !segment.is_empty())
            .map(|segment| LitStr::new(segment, root_lit.span()))
            .collect();

        input.parse::<kw::request_response>()?;

        let cat_input;
        braced!(cat_input in input);
        let mut categories: Vec<RequestCategories> = Vec::new();
        while !cat_input.is_empty() {
            categories.push(cat_input.parse::<RequestCategories>()?);
            _ = parse_optional_comma(&cat_input)?;
        }

        Ok( Self {
            macro_name,
            root_path,
            categories
        })

    }
}

//region Request Categories

struct RequestCategories {
    category_name: LitStr,
    read: RequestWithoutReqBody,
    create: RequestWithReqBody,
    edit: RequestWithReqBody,
    delete: RequestWithReqBody,
    patch: RequestWithReqBody,
}

impl Parse for RequestCategories {
    fn parse(input: ParseStream) -> Result<Self> {

        let category_name: LitStr = input.parse()?;
        input.parse::<Token![:]>()?;

        let category_body;
        braced!(category_body in input);

        input.parse::<kw::read>()?;
        input.parse::<Token![:]>()?;
        let read = ;
        input.parse::<Token![,]>()?;

        input.parse::<kw::create>()?;
        input.parse::<Token![:]>()?;
        let create = ;
        input.parse::<Token![,]>()?;

        input.parse::<kw::edit>()?;
        input.parse::<Token![:]>()?;
        let edit = ;
        input.parse::<Token![,]>()?;

        input.parse::<kw::delete>()?;
        input.parse::<Token![:]>()?;
        let delete = ;
        input.parse::<Token![,]>()?;

        input.parse::<kw::patch>()?;
        input.parse::<Token![:]>()?;
        let patch = ;
        parse_optional_comma(&category_body)?;
    }
}

//region Request

/// A request to read / get state. Does not get a response body (but can have request parameters)
struct RequestWithoutReqBody {
    request_path: Vec<PathElement>,
    description: LitStr,
    request_parameters: URLEncodableStructBuilderParameters,
    response: StructBuilderParameters
}

impl Parse for RequestWithoutReqBody {
    fn parse(input: ParseStream) -> Result<Self> {
        let request_lit_str: LitStr = input.parse()?;
        let request_path: Vec<PathElement> = request_lit_str
            .value()
            .split('/')
            .filter(|segment| !segment.is_empty())
            .map(|element| PathElement::from_str(element))
            .collect();

        input.parse::<Token![:]>()?;


        let members;
        braced!(members in input);

        members.parse::<kw::description>()?;
        members.parse::<Token![:]>()?;
        let description: LitStr = members.parse()?;
        members.parse::<Token![,]>()?;

        members.parse::<kw::request_parameters>()?;
        members.parse::<Token![:]>()?;
        let request_parameters = members.parse::<URLEncodableStructBuilderParameters>()?;
        members.parse::<Token![,]>()?;

        members.parse::<kw::response>()?;
        members.parse::<Token![:]>()?;
        let response = members.parse::<StructBuilderParameters>()?;
        parse_optional_comma(&members)?;

        Ok(Self {request_path, description, request_parameters, response})
    }
}


/// Any other request that isnt a Read Request
struct RequestWithReqBody {
    request_path: Vec<PathElement>,
    description: LitStr,
    request_parameters: URLEncodableStructBuilderParameters,
    request: StructBuilderParameters,
    response: StructBuilderParameters
}

impl Parse for RequestWithReqBody {
    fn parse(input: ParseStream) -> Result<Self> {
        let request_lit_str: LitStr = input.parse()?;
        let request_path: Vec<PathElement> = request_lit_str
            .value()
            .split('/')
            .filter(|segment| !segment.is_empty())
            .map(|element| PathElement::from_str(element))
            .collect();

        input.parse::<Token![:]>()?;


        let members;
        braced!(members in input);

        members.parse::<kw::description>()?;
        members.parse::<Token![:]>()?;
        let description: LitStr = members.parse()?;
        members.parse::<Token![,]>()?;

        members.parse::<kw::request_parameters>()?;
        members.parse::<Token![:]>()?;
        let request_parameters = members.parse::<URLEncodableStructBuilderParameters>()?;
        members.parse::<Token![,]>()?;

        members.parse::<kw::request>()?;
        members.parse::<Token![:]>()?;
        let request = members.parse::<StructBuilderParameters>()?;
        members.parse::<Token![,]>()?;

        members.parse::<kw::response>()?;
        members.parse::<Token![:]>()?;
        let response = members.parse::<StructBuilderParameters>()?;
        parse_optional_comma(&members)?;

        Ok(Self {request_path, description, request_parameters, request, response})
    }
}

type URLEncodableStructBuilderParameters = StructBuilderParameters;

//endregion

//endregion

//endregion


/// An element of a path that may be statically defined or represent a parameter
enum PathElement {
    /// A set path element name
    Static(LitStr),
    /// A parameter in the path itself
    Parameter(LitStr)
}

impl PathElement {
    pub fn from_str(str: &str) -> Self {
        if str.contains("{") && str.contains("}") {
            return Self::Parameter(LitStr::new(str, str.span().unwrap()))
        }
        return Self::Static(LitStr::new(str, str.span().unwrap()))
    }
}

impl Parse for PathElement {
    fn parse(input: ParseStream) -> Result<Self> {
        if input.peek(syn::token::Brace) {
            let param_body;
            braced!(param_body in input);

        }
    }
}

