//! Generates the path matcher of a category: which message a request is for, from its method and
//! path, walking the category's paths as a tree. Transport agnostic, used by REST routing and
//! zenoh key matching alike

use crate::common::{basis_path, lit_str_to_ident};
use crate::feagi_messages::message_path_element::MessagePathElement;
use crate::feagi_messages::request_response::common_structs::FeagiRequestMessageMethod;
use indexmap::IndexMap;
use quote::{format_ident, quote};
use syn::{LitStr, Type};

/// What the matcher needs to know about one message of the category
pub struct MatcherMessage {
    /// Variant name in the category's request / response / path match enums
    pub variant: syn::Ident,
    pub method: FeagiRequestMessageMethod,
    /// Path of the message, without the category's base path
    pub elements: Vec<MessagePathElement>,
    /// The message's parameters struct, or `()`
    pub parameters_type: Type,
    pub has_parameters: bool,
    /// The message's payload struct, or `()`
    pub payload_type: Type,
    pub message_struct: syn::Ident,
}

/// Both functions walk the same tree, one finding the message of a method, one only whether any
/// message has the path
#[derive(Clone, Copy, PartialEq)]
enum Walk {
    MatchMessage,
    PathExists,
}

/// Every path of the category as a tree, one segment per level. Messages end at leaves
#[derive(Default)]
struct PathTree {
    statics: IndexMap<String, (LitStr, PathTree)>,
    configurables: IndexMap<String, (LitStr, PathTree)>,
    /// Parameters of the same name and type at the same level share a branch
    parameters: Vec<(LitStr, Type, String, PathTree)>,
    /// Indices of the messages whose path ends here
    ends: Vec<usize>,
}

impl PathTree {
    fn insert(&mut self, elements: &[MessagePathElement], message: usize) {
        let Some((first, rest)) = elements.split_first() else {
            self.ends.push(message);
            return;
        };
        let branch = match first {
            MessagePathElement::StaticPath(text) => &mut self.statics.entry(text.value()).or_insert_with(|| (text.clone(), PathTree::default())).1,
            MessagePathElement::ConfigurableOfName(name) => {
                &mut self.configurables.entry(name.value()).or_insert_with(|| (name.clone(), PathTree::default())).1
            }
            MessagePathElement::ParameterOfName(name, parameter_type) => {
                let type_key = quote!(#parameter_type).to_string();
                let existing = self
                    .parameters
                    .iter()
                    .position(|(existing_name, _, existing_type, _)| existing_name.value() == name.value() && *existing_type == type_key);
                let index = match existing {
                    Some(index) => index,
                    None => {
                        self.parameters.push((name.clone(), parameter_type.clone(), type_key, PathTree::default()));
                        self.parameters.len() - 1
                    }
                };
                &mut self.parameters[index].3
            }
        };
        branch.insert(rest, message);
    }

    fn has_branches(&self) -> bool {
        !self.statics.is_empty() || !self.configurables.is_empty() || !self.parameters.is_empty()
    }

    /// Code checking this level of the tree, at segment `depth`. Static segments are tried
    /// first, then configurables, then parameters, each falling through to the next when its
    /// branch finds nothing
    fn generate(&self, depth: usize, walk: Walk, messages: &[MatcherMessage], path_match_enum: &syn::Ident) -> proc_macro2::TokenStream {
        let basis = basis_path();
        let static_element = quote!(#basis::feagi_message::feagi_message_path::configurables::FeagiMessagePathStaticElement);
        let mut output = proc_macro2::TokenStream::new();

        if !self.ends.is_empty() {
            let at_end = match walk {
                Walk::PathExists => quote!(return true;),
                Walk::MatchMessage => {
                    let checks = self.ends.iter().map(|&index| {
                        let message = &messages[index];
                        let method = format_ident!("{}", message.method.as_str());
                        let variant = &message.variant;
                        let parameters = parameters_value(message);
                        quote! {
                            if method == #basis::feagi_message::request::feagi_request_descriptor::FeagiMessageEnum::#method {
                                return ::core::option::Option::Some(#path_match_enum::#variant(#parameters));
                            }
                        }
                    });
                    quote!(#( #checks )*)
                }
            };
            output.extend(quote! {
                if segments.len() == #depth {
                    #at_end
                }
            });
        }

        if self.has_branches() {
            let mut branches = proc_macro2::TokenStream::new();
            if !self.statics.is_empty() {
                let arms = self.statics.values().map(|(text, branch)| {
                    let inner = branch.generate(depth + 1, walk, messages, path_match_enum);
                    quote!(#text => { #inner })
                });
                branches.extend(quote! {
                    match segment {
                        #( #arms )*
                        _ => {}
                    }
                });
            }
            for (name, branch) in self.configurables.values() {
                let field = lit_str_to_ident(name).unwrap();
                let inner = branch.generate(depth + 1, walk, messages, path_match_enum);
                branches.extend(quote! {
                    if segment == #static_element::to_str_slice(&configurables.#field) {
                        #inner
                    }
                });
            }
            for (name, parameter_type, _, branch) in &self.parameters {
                let inner = branch.generate(depth + 1, walk, messages, path_match_enum);
                match walk {
                    Walk::MatchMessage => {
                        let binding = parameter_binding(depth, name);
                        branches.extend(quote! {
                            if let ::core::result::Result::Ok(#binding) = <#parameter_type as ::core::str::FromStr>::from_str(segment) {
                                #inner
                            }
                        });
                    }
                    Walk::PathExists => branches.extend(quote! {
                        if <#parameter_type as ::core::str::FromStr>::from_str(segment).is_ok() {
                            #inner
                        }
                    }),
                }
            }
            output.extend(quote! {
                if let ::core::option::Option::Some(&segment) = segments.get(#depth) {
                    #branches
                }
            });
        }

        output
    }
}

/// Name of the variable a parameter parsed at segment `depth` is bound to
fn parameter_binding(depth: usize, name: &LitStr) -> syn::Ident {
    format_ident!("parameter_{}_{}", depth, name.value())
}

/// The parameters struct of `message`, built from the bindings of its path's parameters
fn parameters_value(message: &MatcherMessage) -> proc_macro2::TokenStream {
    if !message.has_parameters {
        return quote!(());
    }
    let fields = message.elements.iter().enumerate().filter_map(|(depth, element)| match element {
        MessagePathElement::ParameterOfName(name, _) => {
            let field = lit_str_to_ident(name).unwrap();
            let binding = parameter_binding(depth, name);
            Some(quote!(#field: #binding))
        }
        _ => None,
    });
    let parameters_type = &message.parameters_type;
    quote!(#parameters_type { #( #fields, )* })
}

/// Code checking the base path, then narrowing `segments` to the rest of the path. Returns
/// `fail` when the base path differs
fn base_path_prefix(base_path: &[MessagePathElement], fail: proc_macro2::TokenStream) -> proc_macro2::TokenStream {
    if base_path.is_empty() {
        return quote!();
    }
    let basis = basis_path();
    let static_element = quote!(#basis::feagi_message::feagi_message_path::configurables::FeagiMessagePathStaticElement);
    let checks = base_path.iter().map(|element| {
        let expected = match element {
            MessagePathElement::StaticPath(text) => quote!(#text),
            MessagePathElement::ConfigurableOfName(name) => {
                let field = lit_str_to_ident(name).unwrap();
                quote!(#static_element::to_str_slice(&configurables.#field))
            }
            MessagePathElement::ParameterOfName(..) => unreachable!("base paths have no parameters"),
        };
        quote! {
            match segments.split_first() {
                ::core::option::Option::Some((first, rest)) if *first == #expected => segments = rest,
                _ => #fail,
            }
        }
    });
    quote! {
        let mut segments = segments;
        #( #checks )*
    }
}

/// The category's path match enum, and the bodies of `match_message_path`,
/// `message_path_exists` and `decode_request` for its `FeagiRequestCategory` impl
pub struct GeneratedMatcher {
    pub path_match_enum: proc_macro2::TokenStream,
    pub trait_functions: proc_macro2::TokenStream,
}

pub fn generate_matcher(
    path_match_enum: &syn::Ident,
    request_enum: &syn::Ident,
    configurables: &syn::Ident,
    base_path: &[MessagePathElement],
    messages: &[MatcherMessage],
) -> GeneratedMatcher {
    let basis = basis_path();
    let method_enum = quote!(#basis::feagi_message::request::feagi_request_descriptor::FeagiMessageEnum);
    let decoder_trait = quote!(#basis::feagi_message::request::feagi_request_message_category::FeagiRequestPayloadDecoder);
    let message_trait = quote!(#basis::feagi_message::request::feagi_request_message::FeagiRequestMessage);

    let mut tree = PathTree::default();
    for (index, message) in messages.iter().enumerate() {
        tree.insert(&message.elements, index);
    }

    let variants = messages.iter().map(|message| {
        let variant = &message.variant;
        let parameters_type = &message.parameters_type;
        quote!(#variant(#parameters_type),)
    });
    let path_match_enum_tokens = quote! {
        /// Which message of the category a path is, with the path parameters parsed from it
        #[derive(Debug)]
        pub enum #path_match_enum {
            #( #variants )*
        }
    };

    let match_prefix = base_path_prefix(base_path, quote!(return ::core::option::Option::None));
    let match_tree = tree.generate(0, Walk::MatchMessage, messages, path_match_enum);
    let exists_prefix = base_path_prefix(base_path, quote!(return false));
    let exists_tree = tree.generate(0, Walk::PathExists, messages, path_match_enum);

    let decode_arms = messages.iter().map(|message| {
        let variant = &message.variant;
        let payload_type = &message.payload_type;
        let message_struct = &message.message_struct;
        quote! {
            #path_match_enum::#variant(parameters) => {
                let request_payload = #decoder_trait::decode_payload::<#payload_type>(decoder)?;
                ::core::option::Option::Some(#request_enum::#variant(<#message_struct as #message_trait>::compose(parameters, request_payload)))
            }
        }
    });
    // An empty enum has no arms, and so nothing uses the decoder
    let consume_decoder = if messages.is_empty() { quote!(let _ = decoder;) } else { quote!() };

    let trait_functions = quote! {
        fn match_message_path(configurables: &#configurables, method: #method_enum, segments: &[&str]) -> ::core::option::Option<#path_match_enum> {
            let _ = (configurables, method);
            #match_prefix
            #match_tree
            ::core::option::Option::None
        }

        fn message_path_exists(configurables: &#configurables, segments: &[&str]) -> bool {
            let _ = configurables;
            #exists_prefix
            #exists_tree
            false
        }

        fn decode_request<Decoder: #decoder_trait>(path_match: #path_match_enum, decoder: Decoder) -> ::core::option::Option<#request_enum> {
            #consume_decoder
            match path_match {
                #( #decode_arms )*
            }
        }
    };

    GeneratedMatcher {
        path_match_enum: path_match_enum_tokens,
        trait_functions,
    }
}
