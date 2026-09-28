use quote::{format_ident, quote};
use syn::LitStr;
use crate::basis::{append_to_lit_str, lit_str_to_ident, prepend_to_lit_str, GeneratorFromTemplate, StructParameters};
use crate::templates::requests_responses::requests_responses_structs::{RequestResponseContract, TemplateRequestResponseCategory};

/// Generates the Ohkami-facing (web server specific) code for a request/response
/// category:
/// - The incoming / outgoing structs (distinct path / query|body / response structs).
/// - `From` conversions bridging them to the shared category structs.
/// - One async Ohkami handler per endpoint that decodes the request, builds the
///   shared request enum, calls a hand-written dispatch function, then maps the
///   returned response enum back to an Ohkami response.
/// - A routing builder `create_<category>_ohkami()` wiring every endpoint.
///
/// The dispatch function is intentionally NOT generated: the generated handlers await
/// `handle_<category>` (named after the category title), which the developer implements
/// by hand with the signature
/// `async fn handle_<category>(<Category>RequestsEnum) -> <Category>ResponsesEnum`.
///
/// Parsing itself flows through serde (all FEAGI field types are serde-capable), so the
/// generated `From` impls are plain field-moves. This generator is expected to be
/// invoked in the same module as `EnumRequestResponseGenerator`, so the shared structs
/// and enums it references are local (also keeping the `From` impls clear of the orphan
/// rule).
pub struct OhkamiServerGenerator;

/// Per-bucket naming information.
///
/// The shared struct prefixes MUST match the identifiers emitted by
/// `EnumRequestResponseGenerator` (`{core_category}Request` / `{core_category}Response`),
/// since the generated code references those structs / enums by name.
struct BucketNaming {
    /// Core category name used by `EnumRequestResponseGenerator` (e.g. "Read", "Create").
    core_category: &'static str,
    /// HTTP method this bucket maps to; prefixes the Ohkami struct names (e.g. "GET").
    http_method: &'static str,
    /// Role of the `request` parameter set for this method. GET carries it as query
    /// parameters, the body-bearing methods carry it as a JSON body. This affects the
    /// generated struct name and which Ohkami extractor the handler uses.
    request_role: &'static str,
}

impl BucketNaming {
    /// Whether this method carries its request set as query parameters (GET) rather
    /// than a JSON body.
    fn uses_query(&self) -> bool {
        self.http_method == "GET"
    }
}

/// Shared identifiers for the category, referenced by every generated handler.
struct CategoryContext {
    /// `<Category>RequestsEnum`.
    request_enum: syn::Ident,
    /// `<Category>ResponsesEnum`.
    response_enum: syn::Ident,
    /// Hand-written dispatch function, `handle_<category>`.
    dispatch_fn: syn::Ident,
    /// True when the category has exactly one endpoint, so response matches are
    /// exhaustive without a catch-all arm.
    single_variant: bool,
}

/// Direction of a struct relative to the server. This decides which serde derive the
/// generated Ohkami struct needs (`Deserialize` for parsing incoming data, `Serialize`
/// for emitting outgoing data).
#[derive(Clone, Copy)]
enum DataDirection {
    /// Parsed from an incoming request (path parameters, query parameters, or body).
    Incoming,
    /// Serialized into an outgoing response.
    Outgoing,
}

impl OhkamiServerGenerator {
    /// Convert a PascalCase title into snake_case for identifier generation.
    fn to_snake_case(input: &str) -> String {
        let mut output = String::new();
        for (index, character) in input.chars().enumerate() {
            if character.is_uppercase() {
                if index != 0 {
                    output.push('_');
                }
                output.extend(character.to_lowercase());
            } else {
                output.push(character);
            }
        }
        output
    }

    /// Build a struct identifier from a prefix and a contract title.
    fn struct_ident(prefix: &str, title: &LitStr) -> syn::Ident {
        let name = prepend_to_lit_str(prefix, title);
        lit_str_to_ident(&name).unwrap() // TODO error handling
    }

    /// Emit a single Ohkami-facing struct for the given parameter set.
    fn emit_rest_struct(
        parameters: &StructParameters,
        struct_name: &syn::Ident,
        description: &LitStr,
        direction: DataDirection,
        output: &mut proc_macro2::TokenStream,
    ) {
        let struct_tokens = parameters.generate_priv_struct(struct_name);

        // Ohkami relies on serde for (de)serialization and on `openapi::Schema` for
        // OpenAPI document generation. The serde direction is fixed by the data flow;
        // the Schema derive is required in both directions.
        let serde_derive = match direction {
            DataDirection::Incoming => quote! { serde::Deserialize },
            DataDirection::Outgoing => quote! { serde::Serialize },
        };

        output.extend(quote! {
            #[doc = #description]
            #[derive(Debug, Clone, #serde_derive, openapi::Schema)]
            #struct_tokens
        });
    }

    /// Produce `field: #source.field,` initializers for every field in `parameters`.
    fn field_initializers(parameters: &StructParameters, source: &syn::Ident) -> proc_macro2::TokenStream {
        let mut initializers = proc_macro2::TokenStream::new();
        for field in parameters.fields() {
            let field_name = lit_str_to_ident(&field.parameter_name).unwrap(); // TODO error handling
            initializers.extend(quote! { #field_name: #source.#field_name, });
        }
        initializers
    }

    /// Emit the `From` impls merging Ohkami structs into the shared request struct and
    /// mapping the shared response struct to the Ohkami response struct.
    fn emit_conversions(
        contract: &RequestResponseContract,
        path_struct: &syn::Ident,
        request_struct: &syn::Ident,
        response_struct: &syn::Ident,
        core_request_struct: &syn::Ident,
        core_response_struct: &syn::Ident,
        path_present: bool,
        request_present: bool,
        response_present: bool,
        output: &mut proc_macro2::TokenStream,
    ) {
        // Incoming merge: the shared request struct only exists when at least one of
        // path/request has fields (mirroring `EnumRequestResponseGenerator`).
        match (path_present, request_present) {
            (false, false) => {}
            (true, false) => {
                let path_inits = Self::field_initializers(&contract.path_parameters, &format_ident!("value"));
                output.extend(quote! {
                    impl From<#path_struct> for #core_request_struct {
                        fn from(value: #path_struct) -> Self {
                            Self { #path_inits }
                        }
                    }
                });
            }
            (false, true) => {
                let request_inits = Self::field_initializers(&contract.request, &format_ident!("value"));
                output.extend(quote! {
                    impl From<#request_struct> for #core_request_struct {
                        fn from(value: #request_struct) -> Self {
                            Self { #request_inits }
                        }
                    }
                });
            }
            (true, true) => {
                let path_inits = Self::field_initializers(&contract.path_parameters, &format_ident!("path"));
                let request_inits = Self::field_initializers(&contract.request, &format_ident!("request"));
                output.extend(quote! {
                    impl From<(#path_struct, #request_struct)> for #core_request_struct {
                        fn from((path, request): (#path_struct, #request_struct)) -> Self {
                            Self {
                                #request_inits
                                #path_inits
                            }
                        }
                    }
                });
            }
        }

        // Outgoing (one-to-one): shared response struct -> Ohkami response struct.
        if response_present {
            let response_inits = Self::field_initializers(&contract.response, &format_ident!("value"));
            output.extend(quote! {
                impl From<#core_response_struct> for #response_struct {
                    fn from(value: #core_response_struct) -> Self {
                        Self { #response_inits }
                    }
                }
            });
        }
    }

    /// Emit the async Ohkami handler that decodes the request, builds the request enum,
    /// calls the dispatch function, and maps the response enum back to an Ohkami response.
    fn emit_handler(
        naming: &BucketNaming,
        contract: &RequestResponseContract,
        ctx: &CategoryContext,
        handler_ident: &syn::Ident,
        path_struct: &syn::Ident,
        request_struct: &syn::Ident,
        response_struct: &syn::Ident,
        core_request_struct: &syn::Ident,
        path_present: bool,
        request_present: bool,
        response_present: bool,
        output: &mut proc_macro2::TokenStream,
    ) {
        let title = lit_str_to_ident(&contract.title).unwrap(); // TODO error handling
        let description = &contract.description;
        let request_enum = &ctx.request_enum;
        let response_enum = &ctx.response_enum;
        let dispatch_fn = &ctx.dispatch_fn;

        // The request-set extractor differs by method: query for GET, JSON body otherwise.
        let request_extractor = if naming.uses_query() {
            quote! { ohkami::claw::Query(request): ohkami::claw::Query<#request_struct> }
        } else {
            quote! { ohkami::claw::Json(request): ohkami::claw::Json<#request_struct> }
        };

        // Handler extractors and the resulting request-enum expression.
        let (extractors, request_expr) = match (path_present, request_present) {
            (false, false) => (
                quote! {},
                quote! { #request_enum::#title() },
            ),
            (true, false) => (
                quote! { ohkami::claw::Path(path): ohkami::claw::Path<#path_struct> },
                quote! { #request_enum::#title(#core_request_struct::from(path)) },
            ),
            (false, true) => (
                request_extractor,
                quote! { #request_enum::#title(#core_request_struct::from(request)) },
            ),
            (true, true) => (
                quote! { ohkami::claw::Path(path): ohkami::claw::Path<#path_struct>, #request_extractor },
                quote! { #request_enum::#title(#core_request_struct::from((path, request))) },
            ),
        };

        // Catch-all arm is only needed (and only valid) when the enum has >1 variant.
        let catch_all = if ctx.single_variant {
            quote! {}
        } else {
            quote! { _ => ::core::unreachable!("dispatch returned an unexpected variant for this endpoint"), }
        };

        let (return_type, dispatch_expr) = if response_present {
            (
                quote! { ohkami::claw::Json<#response_struct> },
                quote! {
                    match #dispatch_fn(request).await {
                        #response_enum::#title(payload) => ohkami::claw::Json(#response_struct::from(payload)),
                        #catch_all
                    }
                },
            )
        } else {
            // No response body: acknowledge with 204. (Not exercised by current templates.)
            (
                quote! { ohkami::claw::status::NoContent },
                quote! {
                    match #dispatch_fn(request).await {
                        #response_enum::#title() => ohkami::claw::status::NoContent,
                        #catch_all
                    }
                },
            )
        };

        output.extend(quote! {
            #[doc = #description]
            async fn #handler_ident(#extractors) -> #return_type {
                let request = #request_expr;
                #dispatch_expr
            }
        });
    }

    /// Generate all code for a single endpoint (structs, conversions, handler, route).
    fn generate_endpoint(
        naming: &BucketNaming,
        contract: &RequestResponseContract,
        base_path: &LitStr,
        ctx: &CategoryContext,
        routes: &mut Vec<proc_macro2::TokenStream>,
        output: &mut proc_macro2::TokenStream,
    ) {
        let title = &contract.title;
        let description = &contract.description;

        // Ohkami-facing struct identifiers.
        let path_struct = Self::struct_ident(&format!("{}PathParameter", naming.http_method), title);
        let request_struct = Self::struct_ident(&format!("{}{}", naming.http_method, naming.request_role), title);
        let response_struct = Self::struct_ident(&format!("{}Response", naming.http_method), title);

        // Shared/core struct identifiers (produced by the enum generator).
        let core_request_struct = Self::struct_ident(&format!("{}Request", naming.core_category), title);
        let core_response_struct = Self::struct_ident(&format!("{}Response", naming.core_category), title);

        let path_present = !contract.path_parameters.is_empty();
        let request_present = !contract.request.is_empty();
        let response_present = !contract.response.is_empty();

        // --- Structs ---
        if path_present {
            Self::emit_rest_struct(&contract.path_parameters, &path_struct, description, DataDirection::Incoming, output);
        }
        if request_present {
            Self::emit_rest_struct(&contract.request, &request_struct, description, DataDirection::Incoming, output);
        }
        if response_present {
            Self::emit_rest_struct(&contract.response, &response_struct, description, DataDirection::Outgoing, output);
        }

        // --- Conversions (From impls) ---
        Self::emit_conversions(
            contract,
            &path_struct,
            &request_struct,
            &response_struct,
            &core_request_struct,
            &core_response_struct,
            path_present,
            request_present,
            response_present,
            output,
        );

        // --- Handler ---
        let handler_ident = format_ident!(
            "{}_{}",
            naming.http_method.to_lowercase(),
            Self::to_snake_case(&title.value())
        );
        Self::emit_handler(
            naming,
            contract,
            ctx,
            &handler_ident,
            &path_struct,
            &request_struct,
            &response_struct,
            &core_request_struct,
            path_present,
            request_present,
            response_present,
            output,
        );

        // --- Route ---
        let method_ident = syn::Ident::new(naming.http_method, proc_macro2::Span::call_site());
        let path_fragment = contract.request_path.to_ohkami_path();
        let route_path = if path_fragment.is_empty() {
            format!("/{}", base_path.value())
        } else {
            format!("/{}/{}", base_path.value(), path_fragment)
        };
        let route_lit = LitStr::new(&route_path, proc_macro2::Span::call_site());
        routes.push(quote! { #route_lit.#method_ident(#handler_ident), });
    }
}

impl GeneratorFromTemplate<TemplateRequestResponseCategory> for OhkamiServerGenerator {
    fn generate_code_from_template(template: TemplateRequestResponseCategory) -> proc_macro2::TokenStream {
        let mut output = proc_macro2::TokenStream::new();
        let mut routes: Vec<proc_macro2::TokenStream> = Vec::new();

        // Shared identifiers referenced by all generated handlers.
        let request_enum = {
            let name = append_to_lit_str(&template.category_name, "RequestsEnum");
            lit_str_to_ident(&name).unwrap() // TODO error handling
        };
        let response_enum = {
            let name = append_to_lit_str(&template.category_name, "ResponsesEnum");
            lit_str_to_ident(&name).unwrap() // TODO error handling
        };
        let category_snake = Self::to_snake_case(&template.category_name.value());
        let dispatch_fn = format_ident!("handle_{}", category_snake);

        let total_endpoints = template.read.len()
            + template.create.len()
            + template.edit.len()
            + template.delete.len()
            + template.patch.len();

        let ctx = CategoryContext {
            request_enum,
            response_enum,
            dispatch_fn,
            single_variant: total_endpoints == 1,
        };

        // GET carries its request set as query parameters; the body-bearing methods
        // carry it as a JSON body. Each bucket's contracts are otherwise handled
        // identically.
        let buckets: [(BucketNaming, &Vec<RequestResponseContract>); 5] = [
            (BucketNaming { core_category: "Read",   http_method: "GET",    request_role: "QueryParameter" }, &template.read),
            (BucketNaming { core_category: "Create", http_method: "POST",   request_role: "Body" },           &template.create),
            (BucketNaming { core_category: "Edit",   http_method: "PUT",    request_role: "Body" },           &template.edit),
            (BucketNaming { core_category: "Delete", http_method: "DELETE", request_role: "Body" },           &template.delete),
            (BucketNaming { core_category: "Patch",  http_method: "PATCH",  request_role: "Body" },           &template.patch),
        ];

        for (naming, contracts) in &buckets {
            for contract in contracts.iter() {
                Self::generate_endpoint(naming, contract, &template.base_path, &ctx, &mut routes, &mut output);
            }
        }

        // Routing builder assembling every endpoint of this category.
        let builder_ident = format_ident!("create_{}_ohkami", category_snake);
        output.extend(quote! {
            pub fn #builder_ident() -> ohkami::Ohkami {
                ohkami::Ohkami::new(( #(#routes)* ))
            }
        });

        output
    }
}
