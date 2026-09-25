use proc_macro2::{Ident, Span};
use quote::{format_ident, quote};
use syn::{braced, LitStr, Token};
use syn::parse::{Parse, ParseBuffer, ParseStream};
use crate::basis::{parse_optional_comma, StructBuilderParameters};

mod kw {
    syn::custom_keyword!(categories);
    
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



//region Template Request

/// All request / responses of a certain category
pub struct CompleteRequestResponsesTemplate {
    pub root_path: Vec<LitStr>,
    pub categories: Vec<TemplateRequestCategory>,
}

impl Parse for CompleteRequestResponsesTemplate {
    fn parse(input: ParseStream) -> syn::Result<Self> {

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
            root_path,
            categories,
        })
    }
}

impl CompleteRequestResponsesTemplate {
    /// Emits per-category `macro_rules!` helpers plus the root `macro_rules! #macro_name`.
    pub fn expand_macro(&self) -> proc_macro2::TokenStream {
        let category_macros = self.categories.iter().map(TemplateRequestCategory::expand_macro);
        let template_body = self.expand_template();

        quote! {#template_body}
    }

    /// Emits `#macro_name, "v2", categories { template_request_category_*!(), ... }`.
    pub fn expand_template(&self) -> proc_macro2::TokenStream {
        let root_path = self.root_path_lit();
        let category_invocations = self.categories.iter().map(|category| {
            let category_macro_name = &category.category_macro_name;
            quote! {
                #category_macro_name!()
            }
        });

        quote! {
            #root_path,
            requests_responses {
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

impl TemplateRequestCategory {
    /// Emits `macro_rules! template_request_category_<name> { () => { ... } };`
    pub fn expand_macro(&self) -> proc_macro2::TokenStream {
        let macro_name = &self.category_macro_name;
        let category_fragment = self.expand_template();

        quote! {
            macro_rules! #macro_name {
                () => {
                    #category_fragment
                };
            }
        }
    }

    /// Emits `"category": { read: { ... }, create: { ... }, ... }`.
    pub fn expand_template(&self) -> proc_macro2::TokenStream {
        let category_name = &self.category_name;
        let read = RequestWithoutReqBody::expand_read_bucket(&self.read);
        let create = RequestWithReqBody::expand_verb_bucket("create", &self.create);
        let edit = RequestWithReqBody::expand_verb_bucket("edit", &self.edit);
        let delete = RequestWithReqBody::expand_verb_bucket("delete", &self.delete);
        let patch = RequestWithReqBody::expand_verb_bucket("patch", &self.patch);

        quote! {
            #category_name: {
                #read
                #create
                #edit
                #delete
                #patch
            }
        }
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

impl RequestWithoutReqBody {
    pub fn expand_read_bucket(endpoints: &[Self]) -> proc_macro2::TokenStream {
        let entries = endpoints.iter().map(Self::expand_template);
        quote! {
            read: {
                #(#entries,)*
            },
        }
    }

    pub fn expand_template(&self) -> proc_macro2::TokenStream {
        let path = PathElement::path_to_lit_str(&self.request_path);
        let request_parameters = self.request_parameters.expand_template();
        let response = self.response.expand_template();
        let description = &self.description;

        quote! {
            #path: {
                description: #description,
                request_parameters: #request_parameters,
                response: #response
            }
        }
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

impl RequestWithReqBody {
    pub fn expand_verb_bucket(verb: &str, endpoints: &[Self]) -> proc_macro2::TokenStream {
        let verb_ident = Ident::new(verb, Span::call_site());
        let entries = endpoints.iter().map(Self::expand_template);
        quote! {
            #verb_ident: {
                #(#entries,)*
            },
        }
    }

    pub fn expand_template(&self) -> proc_macro2::TokenStream {
        let path = PathElement::path_to_lit_str(&self.request_path);
        let request_parameters = self.request_parameters.expand_template();
        let request = self.request.expand_template();
        let response = self.response.expand_template();
        let description = &self.description;

        quote! {
            #path: {
                description: #description,
                request_parameters: #request_parameters,
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

//endregion