use heck::ToSnakeCase;
use quote::{format_ident, quote};
use syn::LitStr;
use crate::basis::{lit_str_to_ident, GeneratorFromTemplate, StructParameters};
use crate::templates::requests_responses::requests_responses_structs::{
    CategoryType, ParameterElementPath, ParameterPathElement, RequestResponseContract,
    TemplateRequestResponseCategory,
};

pub struct OhkamiServerGenerator;

impl OhkamiServerGenerator {
    /// Produce `field: #source.field,` initializers for every field in `parameters`.
    fn field_initializers(parameters: &StructParameters, source: &syn::Ident) -> proc_macro2::TokenStream {
        let mut initializers = proc_macro2::TokenStream::new();
        for field in parameters.fields() {
            let field_name = lit_str_to_ident(&field.parameter_name).unwrap(); // TODO error handling
            initializers.extend(quote! { #field_name: #source.#field_name, });
        }
        initializers
    }

    /// Build the `/base_path/rendered_path` route string, using `:name` for path
    /// parameters (Ohkami routing syntax).
    fn route_path(base_path: &str, path: &ParameterElementPath) -> String {
        let fragment = path
            .elements()
            .iter()
            .map(|element| match element {
                ParameterPathElement::Static(segment) => segment.value(),
                ParameterPathElement::Parameter(segment) => format!(":{}", segment.value()),
            })
            .collect::<Vec<_>>()
            .join("/");

        if fragment.is_empty() {
            format!("/{}", base_path)
        } else {
            format!("/{}/{}", base_path, fragment)
        }
    }

    /// Whether `ty` is the primitive `bool` (bare or path-qualified, e.g.
    /// `std::primitive::bool`).
    fn is_bool_type(ty: &syn::Type) -> bool {
        if let syn::Type::Path(type_path) = ty {
            if type_path.qself.is_none() {
                if let Some(segment) = type_path.path.segments.last() {
                    return segment.ident == "bool" && segment.arguments.is_empty();
                }
            }
        }
        false
    }

    /// Emit a `pub struct` deriving `openapi::Schema` + `#[openapi(component)]` and the
    /// given serde trait (`serde::Serialize` for responses, `serde::Deserialize` for
    /// incoming data).
    ///
    /// `bool` fields are special-cased: ohkami 0.24.9 has no `Schema for bool`, so the
    /// field is routed through `openapi::bool()` via an `#[openapi(schema_with = ...)]`
    /// wrapper. `schema_with` otherwise drops the field's doc description, so the wrapper
    /// re-attaches it. Note the resulting OpenAPI type is ohkami's non-standard `"bool"`
    /// (should be `"boolean"`); this is an accepted, non-blocking limitation of the crate.
    /// TODO We should review this deeper, and hand create a PR for ohkami if this really is the
    /// case as I am still skeptical
    fn emit_component_struct(
        struct_name: &syn::Ident,
        parameters: &StructParameters,
        description: &LitStr,
        serde_trait: proc_macro2::TokenStream,
        output: &mut proc_macro2::TokenStream,
    ) {
        let empty_doc = LitStr::new("", proc_macro2::Span::call_site());
        let mut fields = proc_macro2::TokenStream::new();

        for field in parameters.fields() {
            let field_ident = lit_str_to_ident(&field.parameter_name).unwrap(); // TODO error handling
            let field_type = &field.parameter_type;
            let field_doc = field.description.as_ref().unwrap_or(&empty_doc);

            if Self::is_bool_type(field_type) {
                let wrapper_ident = format_ident!(
                    "{}_{}_bool_schema",
                    struct_name.to_string().to_snake_case(),
                    field_ident
                );
                let wrapper_path = LitStr::new(&wrapper_ident.to_string(), proc_macro2::Span::call_site());
                let description_call = match &field.description {
                    Some(doc) => quote! { .description(#doc) },
                    None => quote! {},
                };
                output.extend(quote! {
                    fn #wrapper_ident() -> impl ::core::convert::Into<ohkami::openapi::schema::SchemaRef> {
                        ohkami::openapi::bool() #description_call
                    }
                });
                fields.extend(quote! {
                    #[doc = #field_doc]
                    #[openapi(schema_with = #wrapper_path)]
                    pub #field_ident: #field_type,
                });
            } else {
                fields.extend(quote! {
                    #[doc = #field_doc]
                    pub #field_ident: #field_type,
                });
            }
        }

        output.extend(quote! {
            #[doc = #description]
            #[derive(Debug, Clone, #serde_trait, openapi::Schema)]
            #[openapi(component)]
            pub struct #struct_name {
                #fields
            }
        });
    }

    /// Generate all code for a single endpoint (structs, handler, route).
    fn generate_endpoint(
        category: CategoryType,
        http_method: &str,
        contract: &RequestResponseContract,
        base_path: &str,
        module: &syn::Ident,
        routes: &mut Vec<proc_macro2::TokenStream>,
        output: &mut proc_macro2::TokenStream,
    ) {
        let category_ident = category.as_ident();
        let title_ident = lit_str_to_ident(&contract.title).unwrap(); // TODO error handling
        let description = &contract.description;

        // Structs the hand-written async function consumes / produces.
        let request_struct = format_ident!("{}{}Request", category_ident, title_ident);
        let response_struct = format_ident!("{}{}Response", category_ident, title_ident);
        // The dedicated async function this endpoint calls (under `module`).
        let function_name = contract.get_async_function_name(category);

        let path_present = !contract.path_parameters.is_empty();
        let request_present = !contract.request.is_empty();
        let uses_query = http_method == "GET";

        // --- Response struct: returned by the async function, sent back as JSON. ---
        // `#[openapi(component)]` registers the schema as a named component so Swagger
        // lists it (with its doc descriptions) and references it by `$ref`.
        Self::emit_component_struct(
            &response_struct,
            &contract.response,
            description,
            quote! { serde::Serialize },
            output,
        );

        // Request struct: passed to the async function (path + request merged)
        let merged_request = contract.request.make_combination_with(&contract.path_parameters);
        Self::emit_component_struct(
            &request_struct,
            &merged_request,
            description,
            quote! { serde::Deserialize },
            output,
        );

        // Handler: decode inputs, bind `request`, call the async function
        let (extractors, build_request) = match (path_present, request_present) {
            (false, false) => (
                quote! {},
                quote! { let request = #request_struct {}; },
            ),
            (true, false) => (
                quote! { ohkami::claw::Path(request): ohkami::claw::Path<#request_struct> },
                quote! {},
            ),
            (false, true) if uses_query => (
                quote! { ohkami::claw::Query(request): ohkami::claw::Query<#request_struct> },
                quote! {},
            ),
            (false, true) => (
                quote! { ohkami::claw::Json(request): ohkami::claw::Json<#request_struct> },
                quote! {},
            ),
            (true, true) => {
                let path_struct = format_ident!("{}{}PathParameters", category_ident, title_ident);
                let body_role = if uses_query { "Query" } else { "Body" };
                let body_struct = format_ident!("{}{}{}", category_ident, title_ident, body_role);

                Self::emit_component_struct(
                    &path_struct,
                    &contract.path_parameters,
                    description,
                    quote! { serde::Deserialize },
                    output,
                );
                Self::emit_component_struct(
                    &body_struct,
                    &contract.request,
                    description,
                    quote! { serde::Deserialize },
                    output,
                );

                let body_extractor = if uses_query {
                    quote! { ohkami::claw::Query(body): ohkami::claw::Query<#body_struct> }
                } else {
                    quote! { ohkami::claw::Json(body): ohkami::claw::Json<#body_struct> }
                };
                let path_inits = Self::field_initializers(&contract.path_parameters, &format_ident!("path"));
                let body_inits = Self::field_initializers(&contract.request, &format_ident!("body"));

                (
                    quote! { ohkami::claw::Path(path): ohkami::claw::Path<#path_struct>, #body_extractor },
                    quote! { let request = #request_struct { #body_inits #path_inits }; },
                )
            }
        };

        let handler_ident = format_ident!(
            "{}_{}",
            http_method.to_lowercase(),
            contract.title.value().to_snake_case()
        );

        output.extend(quote! {
            #[doc = #description]
            async fn #handler_ident(#extractors) -> ohkami::claw::Json<#response_struct> {
                #build_request
                let response = #module::#function_name(request).await;
                ohkami::claw::Json(response)
            }
        });

        // --- Route ---
        let method_ident = syn::Ident::new(http_method, proc_macro2::Span::call_site());
        let route = Self::route_path(base_path, &contract.request_path);
        let route_lit = LitStr::new(&route, proc_macro2::Span::call_site());
        routes.push(quote! { #route_lit.#method_ident(#handler_ident), });
    }
}

impl GeneratorFromTemplate<TemplateRequestResponseCategory> for OhkamiServerGenerator {
    fn generate_code_from_template(template: TemplateRequestResponseCategory) -> proc_macro2::TokenStream {
        let mut output = proc_macro2::TokenStream::new();
        let mut routes: Vec<proc_macro2::TokenStream> = Vec::new();

        // Module holding the hand-written async functions, imported at the call site.
        let module = &template.async_functions_module;
        let base_path = template.base_path.value();

        // GET carries its request set as query parameters; the body-bearing methods
        // carry it as a JSON body. Otherwise every bucket is handled identically.
        let buckets: [(CategoryType, &str, &Vec<RequestResponseContract>); 5] = [
            (CategoryType::Read,   "GET",    &template.read),
            (CategoryType::Create, "POST",   &template.create),
            (CategoryType::Edit,   "PUT",    &template.edit),
            (CategoryType::Delete, "DELETE", &template.delete),
            (CategoryType::Patch,  "PATCH",  &template.patch),
        ];

        for (category, http_method, contracts) in buckets {
            for contract in contracts.iter() {
                Self::generate_endpoint(category, http_method, contract, &base_path, module, &mut routes, &mut output);
            }
        }

        // Routing builder assembling every endpoint of this category.
        let builder_ident = format_ident!("create_{}_ohkami", template.category_name.value().to_snake_case());
        output.extend(quote! {
            pub fn #builder_ident() -> ohkami::Ohkami {
                ohkami::Ohkami::new(( #(#routes)* ))
            }
        });

        output
    }
}
