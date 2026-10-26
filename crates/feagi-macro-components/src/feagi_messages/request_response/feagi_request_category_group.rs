use crate::common::{alloc_gated, basis_path, parse_optional_comma, parse_property_colon_member};
use indexmap::IndexMap;
use proc_macro2::Span;
use quote::{format_ident, quote};
use syn::parse::{Parse, ParseStream};
use syn::{braced, LitStr, Token, Type};

mod kw {
    syn::custom_keyword!(group_name);
    syn::custom_keyword!(categories);
}

//region Feagi Request Category Group

/// A group of request categories, passed around as one. Formatted as the following:
/// group_name: GroupName,
/// categories: {
///     field_name: path::to::CategoryMarker,
/// }
/// Categories keep their template order
pub struct FeagiRequestCategoryGroup {
    pub group_name: syn::Ident,
    pub categories: IndexMap<syn::Ident, Type>,
}

impl FeagiRequestCategoryGroup {
    pub fn requesters_struct_name(&self) -> syn::Ident {
        format_ident!("{}Requesters", self.group_name)
    }

    pub fn responders_struct_name(&self) -> syn::Ident {
        format_ident!("{}Responders", self.group_name)
    }

    pub fn configurables_struct_name(&self) -> syn::Ident {
        format_ident!("{}Configurables", self.group_name)
    }

    /// Generate the requesters, responders and configurables structs of the group, and the
    /// requesters' `FeagiRequestCategoryGroup` impl
    pub fn generate_rust_group(&self) -> proc_macro2::TokenStream {
        let basis = basis_path();
        let category_module = quote!(#basis::feagi_message::request::feagi_request_message_category);
        let grouping = quote!(#basis::feagi_message::request::feagi_request_message_grouping);
        let backend_trait = quote!(#basis::parallelism::feagi_channel::request_response_channel::FeagiRequestResponseChannelBackend);
        let alloc_backend = quote!(#basis::parallelism::feagi_channel::request_response_channel::dynamic::FeagiRequestResponseChannelsAllocBackend);

        let group_name = self.group_name.to_string();
        let requesters = self.requesters_struct_name();
        let responders = self.responders_struct_name();
        let configurables = self.configurables_struct_name();

        let fields: Vec<&syn::Ident> = self.categories.keys().collect();
        let categories: Vec<&Type> = self.categories.values().collect();
        let missing_configurables = fields.iter().map(|field| {
            LitStr::new(
                &format!("category `{field}` of group `{group_name}` declares configurables, so they must be given"),
                Span::call_site(),
            )
        });

        // One chain, each link pairing a category's requester with its configurables
        let nested_type = fields.iter().zip(&categories).rev().fold(quote!(#grouping::NNull), |tail, (_, category)| {
            quote!(#grouping::NCat<#grouping::FeagiConfiguredCategoryRequester<#category, Channels>, #tail>)
        });
        let nested_value = fields.iter().rev().fold(quote!(#grouping::NNull), |tail, field| {
            quote! {
                #grouping::NCat {
                    head: #grouping::FeagiConfiguredCategoryRequester {
                        requester: self.#field,
                        configurables: configurables.#field,
                    },
                    tail: #tail,
                }
            }
        });

        let requesters_doc = LitStr::new(&format!("The requester of every category of the `{group_name}` group. Cloning clones each requester"), Span::call_site());
        let responders_doc = LitStr::new(&format!("The responder of every category of the `{group_name}` group, held by the worker serving them"), Span::call_site());
        let configurables_doc = LitStr::new(&format!("The configurables of every category of the `{group_name}` group. They live as long as the program"), Span::call_site());

        let alloc_constructor = alloc_gated(
            quote! {
                impl #requesters<#alloc_backend> {
                    /// Creates heap allocated channels for every category of the group, each
                    /// queueing up to `queue_capacity` requests
                    pub fn create_alloc_channel_pairs(queue_capacity: usize) -> (Self, #responders<#alloc_backend>) {
                        #(
                            let #fields = <#categories as #category_module::FeagiRequestCategory>::spawn_alloc_requester_responser_pair(queue_capacity);
                        )*
                        (
                            #requesters { #( #fields: #fields.0, )* },
                            #responders { #( #fields: #fields.1, )* },
                        )
                    }
                }
            },
            quote!(),
        );

        quote! {
            #[doc = #requesters_doc]
            pub struct #requesters<Channels: #backend_trait> {
                #( pub #fields: #category_module::FeagiCategoryRequester<#categories, Channels>, )*
            }

            impl<Channels> ::core::clone::Clone for #requesters<Channels>
            where
                Channels: #backend_trait,
                #( #category_module::FeagiCategoryRequester<#categories, Channels>: ::core::clone::Clone, )*
            {
                fn clone(&self) -> Self {
                    #requesters { #( #fields: self.#fields.clone(), )* }
                }
            }

            impl<Channels: #backend_trait> #requesters<Channels> {
                /// Creates channels of the backend's default kind for every category of the group,
                /// each queueing up to `QUEUE_SIZE` requests
                pub fn create_channel_pairs<const QUEUE_SIZE: usize>() -> (Self, #responders<Channels>) {
                    #(
                        let #fields = <#categories as #category_module::FeagiRequestCategory>::spawn_requester_responser_pair::<Channels, QUEUE_SIZE>();
                    )*
                    (
                        #requesters { #( #fields: #fields.0, )* },
                        #responders { #( #fields: #fields.1, )* },
                    )
                }
            }

            #alloc_constructor

            #[doc = #responders_doc]
            pub struct #responders<Channels: #backend_trait> {
                #( pub #fields: #category_module::FeagiCategoryResponder<#categories, Channels>, )*
            }

            #[doc = #configurables_doc]
            #[derive(Clone, Copy)]
            pub struct #configurables {
                #( pub #fields: &'static <#categories as #category_module::FeagiRequestCategory>::Configurables, )*
            }

            impl #configurables {
                /// For groups whose categories declare no configurables. Fails to compile when
                /// used in a const, and panics otherwise, if any category declares some
                pub const fn without_configurables() -> Self {
                    #configurables {
                        #(
                            #fields: match <#categories as #category_module::FeagiRequestCategory>::EMPTY_CONFIGURABLES {
                                ::core::option::Option::Some(configurables) => configurables,
                                ::core::option::Option::None => panic!(#missing_configurables),
                            },
                        )*
                    }
                }
            }

            impl<Channels: #backend_trait> #grouping::FeagiRequestCategoryGroup<Channels> for #requesters<Channels> {
                type Configurables = #configurables;
                type NestedCategories = #nested_type;

                fn into_nested_categories(self, configurables: Self::Configurables) -> Self::NestedCategories {
                    #nested_value
                }
            }
        }
    }
}

impl Parse for FeagiRequestCategoryGroup {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let group_name = parse_property_colon_member::<kw::group_name, syn::Ident>(input)?;

        input.parse::<kw::categories>()?;
        input.parse::<Token![:]>()?;
        let entries;
        braced!(entries in input);
        let mut categories = IndexMap::new();
        while !entries.is_empty() {
            let field: syn::Ident = entries.parse()?;
            entries.parse::<Token![:]>()?;
            let category: Type = entries.parse()?;
            let span = field.span();
            if categories.insert(field, category).is_some() {
                return Err(syn::Error::new(span, "duplicate category field name in group"));
            }
            if entries.is_empty() {
                break;
            }
            entries.parse::<Token![,]>()?;
        }
        if categories.is_empty() {
            return Err(entries.error("a request category group must have at least one category"));
        }
        parse_optional_comma(input)?;

        if !input.is_empty() {
            return Err(input.error("unexpected tokens in request category group"));
        }

        Ok(Self { group_name, categories })
    }
}

//endregion
