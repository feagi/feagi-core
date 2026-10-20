use crate::common::{alloc_gated, basis_path, parse_optional_comma, parse_property_colon_member};
use crate::feagi_messages::message_path_element::MessagePathElement;
use crate::feagi_messages::request_response::common_structs::FeagiRequestMessageU8;
use crate::feagi_messages::request_response::feagi_request_message::FeagiRequestMessage;
use crate::feagi_messages::request_response::feagi_request_message_path::message_path_elements;
use crate::templates::Unparse;
use heck::ToPascalCase;
use indexmap::IndexMap;
use proc_macro2::Span;
use quote::{format_ident, quote};
use syn::parse::{Parse, ParseStream};
use syn::{braced, LitInt, LitStr, Token};

mod kw {
    syn::custom_keyword!(category_name);
    syn::custom_keyword!(base_path);
    syn::custom_keyword!(category_description);
    syn::custom_keyword!(configurables);
    syn::custom_keyword!(read);
    syn::custom_keyword!(create);
    syn::custom_keyword!(edit);
    syn::custom_keyword!(delete);
    syn::custom_keyword!(patch);
}

//region Feagi Request Message Category Specification Struct Names

/// Contains the names of the rust structs that a category itself produces.
/// Child message names come from [`FeagiRequestMessage`], which holds those messages.
#[derive(Clone)]
pub struct FeagiRequestMessageCategoryRustStructNames {
    pub base_name: syn::Ident,
}

impl FeagiRequestMessageCategoryRustStructNames {
    /// Build the name set from the category title.
    pub fn new(base_name: syn::Ident) -> Self {
        Self { base_name }
    }

    /// The zero sized struct implementing `FeagiRequestCategory`
    pub fn category_marker_struct_name(&self) -> syn::Ident {
        format_ident!("{}RequestCategory", self.base_name)
    }

    pub fn configurables_struct_name(&self) -> syn::Ident {
        format_ident!("{}Configurables", self.base_name)
    }

    pub fn request_enum_name(&self) -> syn::Ident {
        format_ident!("{}Requests", self.base_name)
    }

    pub fn response_enum_name(&self) -> syn::Ident {
        format_ident!("{}Responses", self.base_name)
    }
}

/// What the messages of a category need to know about it to generate their own items
pub struct FeagiRequestMessageCategoryContext {
    pub category_marker_name: syn::Ident,
    pub configurables_struct_name: syn::Ident,
    pub request_enum_name: syn::Ident,
    pub response_enum_name: syn::Ident,
    /// Prepended to the path of every message
    pub base_path_elements: Vec<MessagePathElement>,
}

//endregion

//region Feagi Request Message Category

/// Holds all messages of a given category.  Formatted as the following:
/// category_name: "category_name",
/// base_path: "base_path",                  (static segments and `$configurables` only)
/// category_description: "category_description",
/// (following sections are optional)
/// configurables: {
///     configurable_name: max_length,
/// }
/// read: {
///     "MessageTitle": FeagiMessageWithoutPayload,
/// }
/// create: {
///     "MessageTitle": FeagiMessageWithPayload,
/// }
/// edit: {
///     "MessageTitle": FeagiMessageWithPayload,
/// }
/// delete: {
///     "MessageTitle": FeagiMessageWithPayload,
/// }
/// patch: {
///     "MessageTitle": FeagiMessageWithPayload,
/// }
pub struct FeagiRequestMessageCategory {
    pub category_name: LitStr, // PascalCase
    /// Any additional path to this request that is appended to all defined
    pub base_path: LitStr,
    /// `base_path` split into its elements
    pub base_path_elements: Vec<MessagePathElement>,
    /// Comment Description of this category
    pub category_description: LitStr,
    /// Path segments set once at runtime, by name, with their maximum length
    pub configurables: IndexMap<syn::Ident, LitInt>,
    pub read: IndexMap<LitStr, FeagiRequestMessage<0>>, // TODO fix message type (why doesnt FeagiMessageType::Const work???)
    pub create: IndexMap<LitStr, FeagiRequestMessage<1>>, // Name is PascalCase
    pub edit: IndexMap<LitStr, FeagiRequestMessage<2>>,
    pub delete: IndexMap<LitStr, FeagiRequestMessage<3>>,
    pub patch: IndexMap<LitStr, FeagiRequestMessage<4>>,
}

impl FeagiRequestMessageCategory {
    /// Struct names this category will generate. Child message names are queried
    /// from the messages this category holds, not from the category name set.
    pub fn calculate_feagi_category_struct_names(&self) -> FeagiRequestMessageCategoryRustStructNames {
        FeagiRequestMessageCategoryRustStructNames::new(format_ident!("{}", self.category_name.value().to_pascal_case()))
    }

    /// What messages of this category need to generate their items
    pub fn context(&self) -> FeagiRequestMessageCategoryContext {
        let names = self.calculate_feagi_category_struct_names();
        FeagiRequestMessageCategoryContext {
            category_marker_name: names.category_marker_struct_name(),
            configurables_struct_name: names.configurables_struct_name(),
            request_enum_name: names.request_enum_name(),
            response_enum_name: names.response_enum_name(),
            base_path_elements: self.base_path_elements.iter().map(MessagePathElement::duplicate).collect(),
        }
    }

    /// Generate everything of the category: the configurables struct, the items of every
    /// message, the request and response enums, and the category marker
    pub fn generate_rust_category(&self) -> proc_macro2::TokenStream {
        let context = self.context();
        let mut output = self.generate_rust_configurables_struct(&context);

        let mut entries = Vec::new();
        collect_entries(&self.read, &mut entries);
        collect_entries(&self.create, &mut entries);
        collect_entries(&self.edit, &mut entries);
        collect_entries(&self.delete, &mut entries);
        collect_entries(&self.patch, &mut entries);

        for entry in &entries {
            let items = (entry.generate_items)(&context);
            if entry.alloc_only {
                output.extend(alloc_gated(items, quote!()));
            } else {
                output.extend(items);
            }
        }

        // The enums and the category's message list differ with and without alloc only messages
        if entries.iter().any(|entry| entry.alloc_only) {
            let with_alloc_only = self.generate_rust_enums_and_category_marker(&context, entries.iter());
            let without_alloc_only = self.generate_rust_enums_and_category_marker(&context, entries.iter().filter(|entry| !entry.alloc_only));
            output.extend(alloc_gated(with_alloc_only, without_alloc_only));
        } else {
            output.extend(self.generate_rust_enums_and_category_marker(&context, entries.iter()));
        }

        output
    }

    fn generate_rust_configurables_struct(&self, context: &FeagiRequestMessageCategoryContext) -> proc_macro2::TokenStream {
        let basis = basis_path();
        let name = &context.configurables_struct_name;
        let doc = LitStr::new(
            &format!("Path segments of the \"{}\" category that are set at runtime", self.category_name.value()),
            Span::call_site(),
        );
        let fields = self.configurables.iter().map(|(field, max_length)| {
            quote!(pub #field: #basis::__private::heapless::String<#max_length>,)
        });
        quote! {
            #[doc = #doc]
            #[derive(Clone, Debug)]
            pub struct #name {
                #( #fields )*
            }

            impl #basis::feagi_message::feagi_message_path::configurables::FeagiMessagePathConfigurables for #name {}
        }
    }

    fn generate_rust_enums_and_category_marker<'a>(
        &self,
        context: &FeagiRequestMessageCategoryContext,
        entries: impl Iterator<Item = &'a CategoryEntry<'a>>,
    ) -> proc_macro2::TokenStream {
        let basis = basis_path();
        let recyclable = quote!(#basis::parallelism::feagi_channel::recycling::Recyclable);
        let grouping = quote!(#basis::feagi_message::request::feagi_request_message_grouping);
        let request_enum = &context.request_enum_name;
        let response_enum = &context.response_enum_name;
        let marker = &context.category_marker_name;
        let configurables = &context.configurables_struct_name;

        let mut request_variants = proc_macro2::TokenStream::new();
        let mut response_variants = proc_macro2::TokenStream::new();
        let mut froms = proc_macro2::TokenStream::new();
        let mut descriptors = Vec::new();
        for entry in entries {
            let variant = entry.names.category_enum_variant_name();
            let message = entry.names.feagi_message_struct_name();
            let response_payload = &entry.response_payload_type;
            let response_error = &entry.response_error_type;
            request_variants.extend(quote!(#variant(#message),));
            response_variants.extend(quote!(#variant(::core::result::Result<#response_payload, #response_error>),));
            froms.extend(quote! {
                impl ::core::convert::From<#message> for #request_enum {
                    fn from(message: #message) -> Self {
                        #request_enum::#variant(message)
                    }
                }
            });
            descriptors.push(entry.names.feagi_message_descriptor_struct_name());
        }

        let nested_descriptors = descriptors
            .iter()
            .rev()
            .fold(quote!(#grouping::NNull), |tail, head| quote!(#grouping::NMDes<#head, #tail>));

        let category_name = &self.category_name;
        let category_description = &self.category_description;
        let base_path = &self.base_path;
        let request_doc = LitStr::new(&format!("Every request of the \"{}\" category", category_name.value()), Span::call_site());
        let response_doc = LitStr::new(&format!("Every response of the \"{}\" category", category_name.value()), Span::call_site());

        quote! {
            #[doc = #request_doc]
            #[derive(Debug)]
            pub enum #request_enum {
                /// No request. The value of an unused channel slot
                Nothing,
                #request_variants
            }

            impl #recyclable for #request_enum {
                fn new_for_recycler() -> Self {
                    #request_enum::Nothing
                }

                fn recycle_self(&mut self) {
                    *self = #request_enum::Nothing;
                }
            }

            #froms

            #[doc = #response_doc]
            #[derive(Debug)]
            pub enum #response_enum {
                /// No response. The value of an unused channel slot
                Nothing,
                #response_variants
            }

            impl #recyclable for #response_enum {
                fn new_for_recycler() -> Self {
                    #response_enum::Nothing
                }

                fn recycle_self(&mut self) {
                    *self = #response_enum::Nothing;
                }
            }

            #[doc = #category_description]
            #[derive(Clone, Copy, Debug)]
            pub struct #marker;

            impl #basis::feagi_message::request::feagi_request_message_category::FeagiRequestCategory for #marker {
                const CATEGORY_NAME: &'static str = #category_name;
                const CATEGORY_DESCRIPTION: &'static str = #category_description;
                const BASE_PATH: &'static str = #base_path;
                type RequestEnum = #request_enum;
                type ResponseEnum = #response_enum;
                type Configurables = #configurables;
                type NestedMessageDescriptors = #nested_descriptors;
            }
        }
    }

    /// Errors on any `$configurable` in the base path or a message path that the category does
    /// not define
    fn validate_configurables(&self) -> syn::Result<()> {
        let check = |elements: &[MessagePathElement]| -> syn::Result<()> {
            for element in elements {
                if let MessagePathElement::ConfigurableOfName(name) = element {
                    let defined = self.configurables.keys().any(|configurable| *configurable == name.value());
                    if !defined {
                        return Err(syn::Error::new(
                            name.span(),
                            format!("configurable `${}` is not defined in this category's `configurables`", name.value()),
                        ));
                    }
                }
            }
            Ok(())
        };

        check(&self.base_path_elements)?;
        self.read.values().try_for_each(|message| check(message.path.elements()))?;
        self.create.values().try_for_each(|message| check(message.path.elements()))?;
        self.edit.values().try_for_each(|message| check(message.path.elements()))?;
        self.delete.values().try_for_each(|message| check(message.path.elements()))?;
        self.patch.values().try_for_each(|message| check(message.path.elements()))?;
        Ok(())
    }
}

/// One message of the category, with its method erased so all methods can be handled together
struct CategoryEntry<'a> {
    names: crate::feagi_messages::request_response::feagi_request_message::FeagiRequestMessageRustStructNames,
    alloc_only: bool,
    response_payload_type: syn::Type,
    response_error_type: proc_macro2::TokenStream,
    generate_items: Box<dyn Fn(&FeagiRequestMessageCategoryContext) -> proc_macro2::TokenStream + 'a>,
}

/// Adds the messages of one method to `entries`, in template order
fn collect_entries<'a, const METHOD: FeagiRequestMessageU8>(messages: &'a IndexMap<LitStr, FeagiRequestMessage<METHOD>>, entries: &mut Vec<CategoryEntry<'a>>) {
    for (title, message) in messages {
        let names = message.calculate_feagi_message_struct_names(format_ident!("{}", title.value()));
        let item_names = names.clone();
        entries.push(CategoryEntry {
            alloc_only: message.alloc_only,
            response_payload_type: message.response_payload_type(&names),
            response_error_type: message.response_error_type(&names),
            generate_items: Box::new(move |context| message.generate_rust_message_items(&item_names, context)),
            names,
        });
    }
}

impl Parse for FeagiRequestMessageCategory {
    /// Parse the category fields. Verb sections may be omitted, in docstring order.
    fn parse(input: ParseStream) -> syn::Result<Self> {
        /// Parse `keyword` was already consumed. Read `: { "Title": Message, ... }`.
        fn parse_bucket<Message: Parse>(input: ParseStream, section: &str) -> syn::Result<IndexMap<LitStr, Message>> {
            input.parse::<Token![:]>()?;
            let entries;
            braced!(entries in input);
            let mut messages = IndexMap::new();
            while !entries.is_empty() {
                let title: LitStr = entries.parse()?;
                entries.parse::<Token![:]>()?;
                let message: Message = entries.parse()?;
                let title_span = title.span();
                if messages.insert(title, message).is_some() {
                    return Err(syn::Error::new(title_span, format!("duplicate message title in `{section}`")));
                }
                if entries.is_empty() {
                    break;
                }
                entries.parse::<Token![,]>()?;
            }
            parse_optional_comma(input)?;
            Ok(messages)
        }

        let category_name = parse_property_colon_member::<kw::category_name, LitStr>(input)?;
        let base_path = parse_property_colon_member::<kw::base_path, LitStr>(input)?;
        let base_path_elements = if base_path.value().is_empty() { Vec::new() } else { message_path_elements(&base_path)? };
        if base_path_elements
            .iter()
            .any(|element| matches!(element, MessagePathElement::ParameterOfName(..)))
        {
            return Err(syn::Error::new(base_path.span(), "base path may only contain static segments and `$configurables`"));
        }
        let category_description = parse_property_colon_member::<kw::category_description, LitStr>(input)?;

        let configurables = if input.peek(kw::configurables) {
            input.parse::<kw::configurables>()?;
            input.parse::<Token![:]>()?;
            let entries;
            braced!(entries in input);
            let mut configurables = IndexMap::new();
            while !entries.is_empty() {
                let name: syn::Ident = entries.parse()?;
                entries.parse::<Token![:]>()?;
                let max_length: LitInt = entries.parse()?;
                max_length.base10_parse::<usize>()?;
                let span = name.span();
                if configurables.insert(name, max_length).is_some() {
                    return Err(syn::Error::new(span, "duplicate configurable name"));
                }
                if entries.is_empty() {
                    break;
                }
                entries.parse::<Token![,]>()?;
            }
            parse_optional_comma(input)?;
            configurables
        } else {
            IndexMap::new()
        };

        let read = if input.peek(kw::read) {
            input.parse::<kw::read>()?;
            parse_bucket::<FeagiRequestMessage<0>>(input, "read")?
        } else {
            IndexMap::new()
        };
        let create = if input.peek(kw::create) {
            input.parse::<kw::create>()?;
            parse_bucket::<FeagiRequestMessage<1>>(input, "create")?
        } else {
            IndexMap::new()
        };
        let edit = if input.peek(kw::edit) {
            input.parse::<kw::edit>()?;
            parse_bucket::<FeagiRequestMessage<2>>(input, "edit")?
        } else {
            IndexMap::new()
        };
        let delete = if input.peek(kw::delete) {
            input.parse::<kw::delete>()?;
            parse_bucket::<FeagiRequestMessage<3>>(input, "delete")?
        } else {
            IndexMap::new()
        };
        let patch = if input.peek(kw::patch) {
            input.parse::<kw::patch>()?;
            parse_bucket::<FeagiRequestMessage<4>>(input, "patch")?
        } else {
            IndexMap::new()
        };

        if !input.is_empty() {
            return Err(input.error("unexpected tokens in FEAGI message category"));
        }

        let category = Self {
            category_name,
            base_path,
            base_path_elements,
            category_description,
            configurables,
            read,
            create,
            edit,
            delete,
            patch,
        };
        category.validate_configurables()?;
        Ok(category)
    }
}

impl Unparse for FeagiRequestMessageCategory {
    /// Re-emit the category. Empty verb sections are left out.
    fn unparse(&self) -> proc_macro2::TokenStream {
        /// Append `section: { "Title": Message, ... },` when the map is not empty.
        fn extend_bucket<Message: Unparse>(stream: &mut proc_macro2::TokenStream, section: &str, messages: &IndexMap<LitStr, Message>) {
            if messages.is_empty() {
                return;
            }
            let section = proc_macro2::Ident::new(section, Span::call_site());
            let mut entries = proc_macro2::TokenStream::new();
            for (title, message) in messages {
                let body = message.unparse();
                entries.extend(quote!(#title: #body,));
            }
            stream.extend(quote! {
                #section: {
                    #entries
                },
            });
        }

        let category_name = &self.category_name;
        let base_path = &self.base_path;
        let category_description = &self.category_description;
        let mut stream = quote! {
            category_name: #category_name,
            base_path: #base_path,
            category_description: #category_description,
        };
        if !self.configurables.is_empty() {
            let names = self.configurables.keys();
            let max_lengths = self.configurables.values();
            stream.extend(quote!(configurables: { #( #names: #max_lengths, )* },));
        }
        extend_bucket(&mut stream, "read", &self.read);
        extend_bucket(&mut stream, "create", &self.create);
        extend_bucket(&mut stream, "edit", &self.edit);
        extend_bucket(&mut stream, "delete", &self.delete);
        extend_bucket(&mut stream, "patch", &self.patch);
        stream
    }
}

//endregion
