use syn::parse::{Parse, ParseBuffer, ParseStream};
use syn::{braced, LitStr, Result, Token};

use crate::common::parse_optional_comma;
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

//region Request

pub struct RootRequest {
    pub macro_name: syn::Ident,
    pub root_path: Vec<LitStr>,
    pub categories: Vec<RequestCategories>,
}

impl Parse for RootRequest {
    fn parse(input: ParseStream) -> Result<Self> {
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
        input.parse::<kw::request_response>()?;

        let cat_input;
        braced!(cat_input in input);
        let mut categories: Vec<RequestCategories> = Vec::new();
        while !cat_input.is_empty() {
            categories.push(cat_input.parse::<RequestCategories>()?);
            parse_optional_comma(&cat_input)?;
        }

        Ok(Self {
            macro_name,
            root_path,
            categories,
        })
    }
}

//region Request Categories

pub struct RequestCategories {
    pub category_name: LitStr,
    pub read: Vec<RequestWithoutReqBody>,
    pub create: Vec<RequestWithReqBody>,
    pub edit: Vec<RequestWithReqBody>,
    pub delete: Vec<RequestWithReqBody>,
    pub patch: Vec<RequestWithReqBody>,
}

impl Parse for RequestCategories {
    fn parse(input: ParseStream) -> Result<Self> {

        fn parse_category<TYPE: Parse, BODY: Parse>(buffer: &ParseBuffer) -> Result<Vec<BODY>> {
            buffer.parse::<TYPE>()?;
            buffer.parse::<Token![:]>()?;

            let braced_buffer;
            braced!(braced_buffer in buffer);
            let mut request = Vec::new();
            while !braced_buffer.is_empty() {
                request.push(braced_buffer.parse::<BODY>()?);
                parse_optional_comma(&braced_buffer)?;
            }

            parse_optional_comma(&buffer)?;
            Ok(request)
        }

        let category_name: LitStr = input.parse()?;
        input.parse::<Token![:]>()?;

        let category_body;
        braced!(category_body in input);

        let read = parse_category::<kw::read, RequestWithoutReqBody>(&category_body)?;
        let create = parse_category::<kw::create, RequestWithReqBody>(&category_body)?;
        let edit = parse_category::<kw::edit, RequestWithReqBody>(&category_body)?;
        let delete = parse_category::<kw::delete, RequestWithReqBody>(&category_body)?;
        let patch = parse_category::<kw::patch, RequestWithReqBody>(&category_body)?;

        Ok(Self {
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
        let request_path = path_elements_from_lit_str(&request_lit_str)?;

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
        let request_path = path_elements_from_lit_str(&request_lit_str)?;

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

//endregion

//endregion

//endregion

/// An element of a path that may be statically defined or represent a parameter
pub enum PathElement {
    /// A set path element name
    Static(LitStr),
    /// A parameter in the path itself (`{name}` segment)
    Parameter(LitStr),
}

impl PathElement {
    pub fn from_path_segment(segment: &str, span: proc_macro2::Span) -> Self {
        if segment.starts_with('{') && segment.ends_with('}') {
            Self::Parameter(LitStr::new(segment, span))
        } else {
            Self::Static(LitStr::new(segment, span))
        }
    }
}

fn path_elements_from_lit_str(path_lit: &LitStr) -> Result<Vec<PathElement>> {
    let span = path_lit.span();
    let value = path_lit.value();
    if value.is_empty() {
        return Ok(Vec::new());
    }
    Ok(value
        .split('/')
        .filter(|segment| !segment.is_empty())
        .map(|segment| PathElement::from_path_segment(segment, span))
        .collect())
}
