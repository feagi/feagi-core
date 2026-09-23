//! Handles processing request and responses for agent / server communication

use proc_macro::TokenStream;
use quote::format_ident;
use syn::parse::{Parse, ParseBuffer, ParseStream};
use syn::{braced, LitStr, Result, Token, Ident, parse_macro_input};
use crate::common::{parse_optional_comma, StructBuilderParameters};

mod kw {
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


pub fn template_requests_category(input: TokenStream) -> TokenStream {
    let request_category = parse_macro_input!(input as TemplateRequestCategory);

    // TODO validations

    // TODO Generate new macro

}


//region Request Category

pub struct TemplateRequestCategory {
    pub category_macro_name: Ident,
    pub category_name: LitStr,
    pub read: Vec<RequestWithoutReqBody>,
    pub create: Vec<RequestWithReqBody>,
    pub edit: Vec<RequestWithReqBody>,
    pub delete: Vec<RequestWithReqBody>,
    pub patch: Vec<RequestWithReqBody>,
}

impl Parse for TemplateRequestCategory {
    fn parse(input: ParseStream) -> Result<Self> {

        fn parse_category<RequestType: Parse, RequestBody: Parse>(buffer: &ParseBuffer) -> Result<Vec<RequestBody>> {
            buffer.parse::<RequestType>()?;
            buffer.parse::<Token![:]>()?;

            let braced_buffer;
            braced!(braced_buffer in buffer);
            let mut request = Vec::new();
            while !braced_buffer.is_empty() {
                request.push(braced_buffer.parse::<RequestBody>()?);
                parse_optional_comma(&braced_buffer)?;
            }

            parse_optional_comma(&buffer)?;
            Ok(request)
        }

        let category_name: LitStr = input.parse()?;
        
        let category_macro_name_str: String = "template_request_category_".to_string() + &*category_name.value();
        let category_macro_name: Ident = format_ident!("{}", category_macro_name_str);
        
        input.parse::<Token![:]>()?;

        let category_body;
        braced!(category_body in input);

        let read = parse_category::<kw::read, RequestWithoutReqBody>(&category_body)?;
        let create = parse_category::<kw::create, RequestWithReqBody>(&category_body)?;
        let edit = parse_category::<kw::edit, RequestWithReqBody>(&category_body)?;
        let delete = parse_category::<kw::delete, RequestWithReqBody>(&category_body)?;
        let patch = parse_category::<kw::patch, RequestWithReqBody>(&category_body)?;

        Ok(Self {
            category_macro_name,
            category_name,
            read,
            create,
            edit,
            delete,
            patch,
        })
    }
}

//region Request

/// A request to read / get state (GET).
pub struct RequestWithoutReqBody {
    pub request_path: Vec<PathElement>,
    pub description: LitStr,
    pub request_parameters: URLEncodableStructBuilderParameters,
    pub response: StructBuilderParameters,
}

impl Parse for RequestWithoutReqBody {
    fn parse(input: ParseStream) -> Result<Self> {
        fn request_fields<FieldIdent: Parse, FieldType: Parse>(buffer: &ParseBuffer) -> Result<FieldType> {
            buffer.parse::<FieldIdent>()?;
            buffer.parse::<Token![:]>()?;
            let output = buffer.parse::<FieldType>()?;
            parse_optional_comma(&buffer)?;
            Ok(output)
        }

        let request_lit_str: LitStr = input.parse()?;
        let request_path = PathElement::path_elements_from_lit_str(&request_lit_str)?;

        input.parse::<Token![:]>()?;

        let members;
        braced!(members in input);

        let description = request_fields::<kw::description, LitStr>(&members)?;
        let request_parameters = request_fields::<kw::request_parameters, URLEncodableStructBuilderParameters>(&members)?;
        let response = request_fields::<kw::response, StructBuilderParameters>(&members)?;

        Ok(Self {
            request_path,
            description,
            request_parameters,
            response,
        })
    }
}

/// Any other request that is not a read request.
pub struct RequestWithReqBody {
    pub request_path: Vec<PathElement>,
    pub description: LitStr,
    pub request_parameters: URLEncodableStructBuilderParameters,
    pub request: StructBuilderParameters,
    pub response: StructBuilderParameters,
}

impl Parse for RequestWithReqBody {
    fn parse(input: ParseStream) -> Result<Self> {

        fn request_fields<FieldIdent: Parse, FieldType: Parse>(buffer: &ParseBuffer) -> Result<FieldType> {
            buffer.parse::<FieldIdent>()?;
            buffer.parse::<Token![:]>()?;
            let output = buffer.parse::<FieldType>()?;
            parse_optional_comma(&buffer)?;
            Ok(output)
        }

        let request_lit_str: LitStr = input.parse()?;
        let request_path = PathElement::path_elements_from_lit_str(&request_lit_str)?;

        input.parse::<Token![:]>()?;

        let members;
        braced!(members in input);

        let description = request_fields::<kw::description, LitStr>(&members)?;
        let request_parameters = request_fields::<kw::request_parameters, URLEncodableStructBuilderParameters>(&members)?;
        let request = request_fields::<kw::request, StructBuilderParameters>(&members)?;
        let response = request_fields::<kw::response, StructBuilderParameters>(&members)?;

        Ok(Self {
            request_path,
            description,
            request_parameters,
            request,
            response,
        })
    }
}

type URLEncodableStructBuilderParameters = StructBuilderParameters;

/// An element of a path that may be statically defined or represent a parameter
pub enum PathElement {
    /// A set path element name
    Static(LitStr),
    /// A parameter in the path itself (`{name}` segment)
    Parameter(LitStr),
}

impl PathElement {
    pub fn path_elements_from_lit_str(path_lit: &LitStr) -> Result<Vec<PathElement>> {
        let span = path_lit.span();
        let value = path_lit.value();
        if value.is_empty() {
            return Ok(Vec::new());
        }
        Ok(value
            .split('/')
            .filter(|segment| !segment.is_empty())
            .map(|segment| Self::from_path_segment(segment, span))
            .collect())
    }

    pub fn from_path_segment(segment: &str, span: proc_macro2::Span) -> Self {
        if segment.starts_with('{') && segment.ends_with('}') {
            Self::Parameter(LitStr::new(segment, span))
        } else {
            Self::Static(LitStr::new(segment, span))
        }
    }
}

//endregion

//endregion

