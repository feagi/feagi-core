use quote::{format_ident, quote};
use syn::LitStr;
use crate::basis::{lit_str_to_ident, prepend_to_lit_str, GeneratorFromTemplate, StructParameters};
use crate::templates::requests_responses::requests_responses_structs::{RequestResponseContract, TemplateRequestResponseCategory};

pub struct OhkamiServerGenerator;

// Ohkami-facing struct name prefixes.
const OHKAMI_PATH_PREFIX: &str = "GETPathParameter";
const OHKAMI_QUERY_PREFIX: &str = "GETQueryParameter";
const OHKAMI_RESPONSE_PREFIX: &str = "GETResponse";

const CORE_READ_REQUEST_PREFIX: &str = "ReadRequest";
const CORE_READ_RESPONSE_PREFIX: &str = "ReadResponse";

#[derive(Clone, Copy)]
enum DataDirection {
    /// Parsed from an incoming request (path parameters or query parameters).
    Incoming,
    /// Serialized into an outgoing response.
    Outgoing,
}

impl OhkamiServerGenerator {
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
        // OpenAPI
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

    /// Generate the structs and `From` impls for a single `read` (GET) endpoint.
    fn generate_read_endpoint(contract: &RequestResponseContract, output: &mut proc_macro2::TokenStream) {
        let title = &contract.title;
        let description = &contract.description;

        // Ohkami-facing struct identifiers.
        let path_struct = Self::struct_ident(OHKAMI_PATH_PREFIX, title);
        let query_struct = Self::struct_ident(OHKAMI_QUERY_PREFIX, title);
        let response_struct = Self::struct_ident(OHKAMI_RESPONSE_PREFIX, title);

        // Shared/core struct identifiers (produced by the enum generator).
        let core_request_struct = Self::struct_ident(CORE_READ_REQUEST_PREFIX, title);
        let core_response_struct = Self::struct_ident(CORE_READ_RESPONSE_PREFIX, title);

        let path_present = !contract.path_parameters.is_empty();
        let query_present = !contract.request.is_empty();
        let response_present = !contract.response.is_empty();

        // Incoming structs (path parameters + query parameters)
        if path_present {
            Self::emit_rest_struct(&contract.path_parameters, &path_struct, description, DataDirection::Incoming, output);
        }
        if query_present {
            Self::emit_rest_struct(&contract.request, &query_struct, description, DataDirection::Incoming, output);
        }

        // Outgoing struct (response payload)
        if response_present {
            Self::emit_rest_struct(&contract.response, &response_struct, description, DataDirection::Outgoing, output);
        }

        // Incoming merge: Ohkami path/query structs -> shared request struct
        match (path_present, query_present) {
            (false, false) => {
                // No shared request struct exists; nothing to convert.
            }
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
                let query_inits = Self::field_initializers(&contract.request, &format_ident!("value"));
                output.extend(quote! {
                    impl From<#query_struct> for #core_request_struct {
                        fn from(value: #query_struct) -> Self {
                            Self { #query_inits }
                        }
                    }
                });
            }
            (true, true) => {
                let path_inits = Self::field_initializers(&contract.path_parameters, &format_ident!("path"));
                let query_inits = Self::field_initializers(&contract.request, &format_ident!("query"));
                output.extend(quote! {
                    impl From<(#path_struct, #query_struct)> for #core_request_struct {
                        fn from((path, query): (#path_struct, #query_struct)) -> Self {
                            Self {
                                #query_inits
                                #path_inits
                            }
                        }
                    }
                });
            }
        }

        // Outgoing (one-to-one): shared response struct -> Ohkami response struct
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
}

impl GeneratorFromTemplate<TemplateRequestResponseCategory> for OhkamiServerGenerator {
    fn generate_code_from_template(template: TemplateRequestResponseCategory) -> proc_macro2::TokenStream {
        let mut output = proc_macro2::TokenStream::new();

        for get_request in &template.read {
            Self::generate_read_endpoint(get_request, &mut output);
        }

        // TODO: create/edit/delete/patch (POST/PUT/DELETE/PATCH) buckets.

        output
    }
}
