use std::fmt::format;
use heck::ToSnakeCase;
use proc_macro2::{Ident, Span};
use quote::{format_ident, quote};
use syn::{braced, LitStr, Token};
use syn::parse::{Parse, ParseBuffer, ParseStream};
use crate::basis::{parse_optional_comma, parse_property_colon_member, StructParameters, TemplateStruct};
use crate::templates::requests_responses::requests_responses_structs::kw::feagi_error_type;

mod kw {

    syn::custom_keyword!(category_name);
    syn::custom_keyword!(base_path);
    syn::custom_keyword!(category_description);
    syn::custom_keyword!(feagi_error_type);
    syn::custom_keyword!(read);
    syn::custom_keyword!(create);
    syn::custom_keyword!(edit);
    syn::custom_keyword!(delete);
    syn::custom_keyword!(patch);

    syn::custom_keyword!(title);
    syn::custom_keyword!(description);
    syn::custom_keyword!(path_parameters);
    syn::custom_keyword!(request);
    syn::custom_keyword!(response);
}

//region Request Category

pub struct TemplateRequestResponseCategory {
    pub category_name: LitStr,
    /// Any additional path to this request that is appended to all defined
    pub base_path: LitStr,
    /// Comment Description of this category
    pub category_description: LitStr,
    /// What type of error to use (FeagiError)
    pub feagi_error_type: syn::Type,
    pub read: Vec<RequestResponseContract>,
    pub create: Vec<RequestResponseContract>,
    pub edit: Vec<RequestResponseContract>,
    pub delete: Vec<RequestResponseContract>,
    pub patch: Vec<RequestResponseContract>,
}

impl Parse for TemplateRequestResponseCategory {
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

        let category_name: LitStr = parse_property_colon_member::<kw::category_name, LitStr>(&input)?;
        let base_path: LitStr = parse_property_colon_member::<kw::base_path, LitStr>(&input)?;
        let category_description: LitStr = parse_property_colon_member::<kw::category_description, LitStr>(&input)?;
        let feagi_error_type: syn::Type = parse_property_colon_member::<kw::feagi_error_type, syn::Type>(&input)?;

        let read = parse_category::<kw::read, RequestResponseContract>(&input)?;
        let create = parse_category::<kw::create, RequestResponseContract>(&input)?;
        let edit = parse_category::<kw::edit, RequestResponseContract>(&input)?;
        let delete = parse_category::<kw::delete, RequestResponseContract>(&input)?;
        let patch = parse_category::<kw::patch, RequestResponseContract>(&input)?;

        Ok(Self {
            category_name,
            base_path,
            category_description,
            feagi_error_type,
            read,
            create,
            edit,
            delete,
            patch,
        })
    }
}

impl TemplateStruct for TemplateRequestResponseCategory {
    fn expand_template(&self) -> proc_macro2::TokenStream {
        let category_name = &self.category_name;
        let base_path = &self.base_path;
        let category_description = &self.category_description;
        let feagi_error_type = &self.feagi_error_type;
        let read = RequestResponseContract::expand_verb_bucket("read", &self.read);
        let create = RequestResponseContract::expand_verb_bucket("create", &self.create);
        let edit = RequestResponseContract::expand_verb_bucket("edit", &self.edit);
        let delete = RequestResponseContract::expand_verb_bucket("delete", &self.delete);
        let patch = RequestResponseContract::expand_verb_bucket("patch", &self.patch);

        quote! {
            category_name: #category_name,
            base_path: #base_path,
            category_description: #category_description,
            feagi_error_type: #feagi_error_type,
            #read
            #create
            #edit
            #delete
            #patch
        }
    }
}

//region Request

/// A request of some sort
pub struct RequestResponseContract {
    /// The given request path, including path parameters, beyond the root path definition
    pub request_path: ParameterElementPath,
    /// The title of this ReqResContract, Mainly used for struct generation
    pub title: LitStr,
    /// The Doc String to use for the generated structs
    pub description: LitStr,
    /// What Struct will be sent as path parameters
    pub path_parameters: StructParameters,
    /// General Request Payload
    pub request: StructParameters,
    /// Response Payload (assuming no error)
    pub response: StructParameters,
}

impl Parse for RequestResponseContract {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let request_path = input.parse()?;
        input.parse::<Token![:]>()?;

        let members;
        braced!(members in input);

        let title = parse_property_colon_member::<kw::title, LitStr>(&members)?;
        let description = parse_property_colon_member::<kw::description, LitStr>(&members)?;
        let path_parameters = parse_property_colon_member::<kw::path_parameters, StructParameters>(&members)?;

        let request = parse_property_colon_member::<kw::request, StructParameters>(&members)?;
        let response = parse_property_colon_member::<kw::response, StructParameters>(&members)?;



        Ok(Self {
            request_path,
            title,
            description,
            path_parameters,
            request,
            response,
        })
    }
}

impl TemplateStruct for RequestResponseContract {
    fn expand_template(&self) -> proc_macro2::TokenStream {
        let request_path = self.request_path.expand_template();
        let path_parameters = self.path_parameters.expand_template();
        let request = self.request.expand_template();
        let response = self.response.expand_template();
        let title = &self.title;
        let description = &self.description;

        quote! {
            #request_path: {
                title: #title,
                description: #description,
                path_parameters: #path_parameters,
                request: #request,
                response: #response
            }
        }
    }
}

impl RequestResponseContract {
    pub fn expand_verb_bucket(verb: &str, endpoints: &[Self]) -> proc_macro2::TokenStream {
        let verb_ident = Ident::new(verb, Span::call_site());
        let entries = endpoints.iter().map(TemplateStruct::expand_template);
        quote! {
            #verb_ident: {
                #(#entries,)*
            },
        }
    }

    /// given the category this is in, parse the name of the async function that will be called
    pub fn get_async_function_name(&self, category: CategoryType) -> Ident {
        use heck::ToSnakeCase;
        // Both parts are snake_cased so the generated function name stays idiomatic
        // (e.g. category `Read` + title `HealthCheck` -> `read_health_check`).
        let category_name = category.as_ident().to_string().to_snake_case();
        let path_name = self.title.value().to_snake_case();
        format_ident!("{}_{}", category_name, path_name)
    }
}



//region Structs

#[derive(Clone, Copy)]
pub enum CategoryType {
    Read,
    Create,
    Edit,
    Delete,
    Patch
}

impl CategoryType {
    pub fn as_ident(&self) -> Ident {
        match &self {
            CategoryType::Read => format_ident!("Read"),
            CategoryType::Create => format_ident!("Create"),
            CategoryType::Edit => format_ident!("Edit"),
            CategoryType::Delete => format_ident!("Delete"),
            CategoryType::Patch => format_ident!("Patch"),
        }
    }
}

/// The root path, doesnt have any variance
pub struct RootPath(Vec<syn::LitStr>);

impl Parse for RootPath {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let path_str: syn::LitStr = input.parse()?;
        let path_parts = path_str
            .value()
            .split('/')
            .map(|s| LitStr::new(s, Span::call_site())).collect();
        Ok(Self(path_parts))
    }
}

impl TemplateStruct for RootPath {
    fn expand_template(&self) -> proc_macro2::TokenStream {
        let string_out: String = self.0
            .iter()
            .map(|l| l.value().to_string())
            .collect::<Vec<_>>()
            .join("/");
        quote! {#string_out}
    }
}

impl RootPath {


    pub fn to_vec(self) -> Vec<syn::LitStr> {self.0}
}

/// The vector of Parameter Elements that make up a path
pub struct ParameterElementPath(Vec<ParameterPathElement>);

impl Parse for ParameterElementPath {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let path_str: syn::LitStr = input.parse()?;
        let path_parts: Vec<LitStr> = path_str
            .value()
            .split('/')
            .map(|s| LitStr::new(s, Span::call_site())).collect();

        // verify no collisions
        // TODO check for repititions

        let o: Vec<ParameterPathElement> = path_parts
            .iter()
            .map(ParameterPathElement::from_lit_str)
            .collect::<syn::Result<Vec<_>>>()?;

        Ok(Self(o))
    }
}

impl TemplateStruct for ParameterElementPath {
    fn expand_template(&self) -> proc_macro2::TokenStream {
        let parts: Vec<String> = self.0
            .iter()
            .map(|element| match element {
                ParameterPathElement::Static(segment) => segment.value(),
                ParameterPathElement::Parameter(segment) => format!("{{{}}}", segment.value()),
            })
            .collect();
        let path = LitStr::new(&parts.join("/"), Span::call_site());
        quote! { #path }
    }
}

impl ParameterElementPath {
    /// Borrow the ordered path elements so callers can render them for whatever
    /// framework/routing syntax they target (kept generic; no framework specifics here).
    pub fn elements(&self) -> &[ParameterPathElement] {
        &self.0
    }
}




/// An element of a path that may be statically defined or represent a parameter
pub enum ParameterPathElement {
    /// A set path element name
    Static(LitStr),
    /// A parameter in the path itself (`{name}` segment)
    Parameter(LitStr),
}

impl Parse for ParameterPathElement {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let element: syn::LitStr = input.parse()?;
        Self::from_lit_str(&element)
    }
}

impl TemplateStruct for ParameterPathElement {
    fn expand_template(&self) -> proc_macro2::TokenStream {
        match self {
            ParameterPathElement::Static(segment) => quote! { #segment },
            ParameterPathElement::Parameter(segment) => {
                let wrapped = format!("{{{}}}", segment.value());
                let wrapped = LitStr::new(&wrapped, segment.span());
                quote! { #wrapped }
            }
        }
    }
}

impl ParameterPathElement {

    pub fn from_lit_str(lit_str: &LitStr) -> syn::Result<Self> {
        let element_str = lit_str.value();

        let mut bracket_count: u8 = 0;
        if element_str.starts_with("{") { bracket_count += 1; }
        if element_str.ends_with("}") { bracket_count += 1; }

        if bracket_count == 1u8 {
            return Err(syn::Error::new(Span::call_site(), "Brackets needed on both ends!"))
        }
        if bracket_count == 0 {
            return Ok(Self::Static(lit_str.clone()))
        } else {
            let mut c = element_str.chars();
            c.next();
            c.next_back();
            let element_str = c.as_str();
            let element: LitStr = LitStr::new(element_str, Span::call_site());
            Ok(Self::Parameter(element))
        }
    }


}


//endregion

//endregion

//endregion

