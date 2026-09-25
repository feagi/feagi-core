use proc_macro2::{Ident, Span};
use quote::quote;
use syn::{braced, LitStr, Token};
use syn::parse::{Parse, ParseBuffer, ParseStream};
use crate::basis::{parse_optional_comma, StructBuilderParameterField, StructBuilderParameters, TemplateStruct};

mod kw {

    syn::custom_keyword!(category_name);
    syn::custom_keyword!(base_path);
    syn::custom_keyword!(read);
    syn::custom_keyword!(create);
    syn::custom_keyword!(edit);
    syn::custom_keyword!(delete);
    syn::custom_keyword!(patch);

    syn::custom_keyword!(description);
    syn::custom_keyword!(path_parameters);
    syn::custom_keyword!(request);
    syn::custom_keyword!(response);
}

//region Request Category

pub struct TemplateRequestCategory {
    pub category_name: LitStr,
    pub base_path: LitStr,
    pub read: Vec<RequestWithReqBody>,
    pub create: Vec<RequestWithReqBody>,
    pub edit: Vec<RequestWithReqBody>,
    pub delete: Vec<RequestWithReqBody>,
    pub patch: Vec<RequestWithReqBody>,
}

impl Parse for TemplateRequestCategory {
    fn parse(input: ParseStream) -> syn::Result<Self> {

        fn parse_category<RequestType: Parse, RequestBody: Parse>(buffer: &ParseBuffer) -> syn::Result<Vec<RequestBody>> {
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


        input.parse::<kw::category_name>()?;
        input.parse::<Token![:]>()?;
        let category_name: LitStr = input.parse()?;
        input.parse::<Token![,]>()?;

        input.parse::<kw::base_path>()?;
        input.parse::<Token![:]>()?;
        let base_path: LitStr = input.parse()?;
        input.parse::<Token![,]>()?;

        let read = parse_category::<kw::read, RequestWithReqBody>(&input)?;
        let create = parse_category::<kw::create, RequestWithReqBody>(&input)?;
        let edit = parse_category::<kw::edit, RequestWithReqBody>(&input)?;
        let delete = parse_category::<kw::delete, RequestWithReqBody>(&input)?;
        let patch = parse_category::<kw::patch, RequestWithReqBody>(&input)?;

        Ok(Self {
            category_name,
            base_path,
            read,
            create,
            edit,
            delete,
            patch,
        })
    }
}

impl TemplateStruct for TemplateRequestCategory {
    fn expand_template(&self) -> proc_macro2::TokenStream {
        let category_name = &self.category_name;
        let base_path = &self.base_path;
        let read = RequestWithReqBody::expand_verb_bucket("read", &self.read);
        let create = RequestWithReqBody::expand_verb_bucket("create", &self.create);
        let edit = RequestWithReqBody::expand_verb_bucket("edit", &self.edit);
        let delete = RequestWithReqBody::expand_verb_bucket("delete", &self.delete);
        let patch = RequestWithReqBody::expand_verb_bucket("patch", &self.patch);

        quote! {
            category_name: #category_name,
            base_path: #base_path,
            #read
            #create
            #edit
            #delete
            #patch
        }
    }
}

//region Request

/// Any other request that is not a read request.
pub struct RequestWithReqBody {
    pub request_path: Vec<PathElement>,
    pub description: LitStr,
    pub path_parameters: URLEncodableStructBuilderParameters,
    pub request: StructBuilderParameters,
    pub response: StructBuilderParameters,
}

impl Parse for RequestWithReqBody {
    fn parse(input: ParseStream) -> syn::Result<Self> {

        fn request_fields<FieldIdent: Parse, FieldType: Parse>(buffer: &ParseBuffer) -> syn::Result<FieldType> {
            buffer.parse::<FieldIdent>()?;
            buffer.parse::<Token![:]>()?;
            let output = buffer.parse::<FieldType>()?;
            parse_optional_comma(&buffer)?;
            Ok(output)
        }

        let request_lit_str: LitStr = input.parse()?;
        let request_path = PathElement::path_elements_from_lit_str(&request_lit_str)?;

        // TODO ensure no "Parameter" PathElements in request_path are repeating

        input.parse::<Token![:]>()?;

        let members;
        braced!(members in input);

        let description = request_fields::<kw::description, LitStr>(&members)?;
        let path_parameters = request_fields::<kw::path_parameters, URLEncodableStructBuilderParameters>(&members)?;

        // TODO ensure each member of path_parameters .parameter_name property has a a matching member from path_parameters "Parameter" PathElements and vice versa (there should be a 1-1 mapping between them with no open pairs)
        
        let request = request_fields::<kw::request, StructBuilderParameters>(&members)?;
        let response = request_fields::<kw::response, StructBuilderParameters>(&members)?;



        Ok(Self {
            request_path,
            description,
            path_parameters,
            request,
            response,
        })
    }
}

impl RequestWithReqBody {
    pub fn expand_verb_bucket(verb: &str, endpoints: &[Self]) -> proc_macro2::TokenStream {
        let verb_ident = Ident::new(verb, Span::call_site());
        let entries = endpoints.iter().map(TemplateStruct::expand_template);
        quote! {
            #verb_ident: {
                #(#entries,)*
            },
        }
    }
}

impl TemplateStruct for RequestWithReqBody {
    fn expand_template(&self) -> proc_macro2::TokenStream {
        let path = PathElement::path_to_lit_str(&self.request_path);
        let path_parameters = self.path_parameters.expand_template();
        let request = self.request.expand_template();
        let response = self.response.expand_template();
        let description = &self.description;

        quote! {
            #path: {
                description: #description,
                path_parameters: #path_parameters,
                request: #request,
                response: #response
            }
        }
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
    pub fn path_elements_from_lit_str(path_lit: &LitStr) -> syn::Result<Vec<PathElement>> {
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

    /// Joins parsed path elements back into a single path string literal for the contract DSL.
    pub fn path_to_lit_str(elements: &[Self]) -> LitStr {
        let span = elements
            .first()
            .map(|element| match element {
                PathElement::Static(lit) | PathElement::Parameter(lit) => lit.span(),
            })
            .unwrap_or(Span::call_site());

        let path = elements
            .iter()
            .map(|element| match element {
                PathElement::Static(lit) | PathElement::Parameter(lit) => lit.value(),
            })
            .collect::<Vec<_>>()
            .join("/");

        LitStr::new(&path, span)
    }
}




//endregion

//endregion

