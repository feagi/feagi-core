use proc_macro2::{Ident, Span};
use quote::{quote, TokenStreamExt};
use syn::{braced, LitStr, Token};
use syn::parse::{Parse, ParseBuffer, ParseStream};
use crate::basis::{parse_optional_comma, parse_property_colon_member, StructBuilderParameterField, StructBuilderParameters, TemplateStruct};

mod kw {

    syn::custom_keyword!(category_name);
    syn::custom_keyword!(base_path);
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
    pub base_path: LitStr,
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

        let read = parse_category::<kw::read, RequestResponseContract>(&input)?;
        let create = parse_category::<kw::create, RequestResponseContract>(&input)?;
        let edit = parse_category::<kw::edit, RequestResponseContract>(&input)?;
        let delete = parse_category::<kw::delete, RequestResponseContract>(&input)?;
        let patch = parse_category::<kw::patch, RequestResponseContract>(&input)?;

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

impl TemplateStruct for TemplateRequestResponseCategory {
    fn expand_template(&self) -> proc_macro2::TokenStream {
        let category_name = &self.category_name;
        let base_path = &self.base_path;
        let read = RequestResponseContract::expand_verb_bucket("read", &self.read);
        let create = RequestResponseContract::expand_verb_bucket("create", &self.create);
        let edit = RequestResponseContract::expand_verb_bucket("edit", &self.edit);
        let delete = RequestResponseContract::expand_verb_bucket("delete", &self.delete);
        let patch = RequestResponseContract::expand_verb_bucket("patch", &self.patch);

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

/// A request of some sort
pub struct RequestResponseContract {
    /// The given request path, including path parameters, beyond the root path definition
    pub request_path: ParameterElementPath,
    /// The title of this ReqResContract, Mainly used for struct generation
    pub title: LitStr,
    /// The Doc String to use for the generated structs
    pub description: LitStr,
    /// What Struct will be sent as path parameters
    pub path_parameters: StructBuilderParameters,
    /// General Request Payload
    pub request: StructBuilderParameters,
    /// Response Payload (assuming no error)
    pub response: StructBuilderParameters,
}

impl Parse for RequestResponseContract {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let request_lit_str: LitStr = input.parse()?;
        let request_path = ParameterElementPath::parse(input)?;
        input.parse::<Token![:]>()?;

        let members;
        braced!(members in input);

        let title = parse_property_colon_member::<kw::title, LitStr>(&members)?;
        let description = parse_property_colon_member::<kw::description, LitStr>(&members)?;
        let path_parameters = parse_property_colon_member::<kw::path_parameters, StructBuilderParameters>(&members)?;

        let request = parse_property_colon_member::<kw::request, StructBuilderParameters>(&members)?;
        let response = parse_property_colon_member::<kw::response, StructBuilderParameters>(&members)?;



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
        let description = &self.description;

        quote! {
            #request_path: {
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

        let o: Vec<ParameterPathElement> =
            path_parts.iter().map(|x| ParameterPathElement::from_lit_str(x)?).collect();

        Ok(Self(o))
    }
}

impl TemplateStruct for ParameterElementPath {
    fn expand_template(&self) -> proc_macro2::TokenStream {
        let mut out: proc_macro2::TokenStream = proc_macro2::TokenStream::new();
        for element in self.0 {
            let e = element.expand_template();
            out.append(quote! {/#e});
        };
        out
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
        match &self {
            ParameterPathElement::Static(e) => {quote!(e)}
            ParameterPathElement::Parameter(e) => {
                let o = format!("{{{}}}", e.value());
                quote! {o}
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

