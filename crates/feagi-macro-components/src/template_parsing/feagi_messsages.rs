use crate::common::{lit_str_to_ident, parse_optional_comma, parse_property_colon_member, RustVisibility};
use crate::template_parsing::property_descriptors::PropertyDescriptors;
use crate::template_parsing::struct_template::StructTemplate;
use crate::templates::Unparse;
use proc_macro2::{Span};
use quote::{format_ident, quote, TokenStreamExt};
use std::collections::HashMap;
use syn::parse::{Parse, ParseStream};
use syn::{braced, LitStr, Token, Type};

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

    syn::custom_keyword!(path);
    syn::custom_keyword!(path_item_descriptions);
    syn::custom_keyword!(description);
    syn::custom_keyword!(payload);
    syn::custom_keyword!(response);
}

//region Feagi Message Category

/// Holds all messages of a given category.  Formatted as the following:
/// category_name: "category_name",
/// base_path: "base_path",
/// category_description: "category_description",
/// (following sections are optional)
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
pub struct FeagiMessageCategory {
    pub category_name: LitStr,
    /// Any additional path to this request that is appended to all defined
    pub base_path: LitStr,
    /// Comment Description of this category
    pub category_description: LitStr,
    pub read: HashMap<LitStr, FeagiMessage<true>>,
    pub create: HashMap<LitStr, FeagiMessage<false>>,
    pub edit: HashMap<LitStr, FeagiMessage<false>>,
    pub delete: HashMap<LitStr, FeagiMessage<false>>,
    pub patch: HashMap<LitStr, FeagiMessage<false>>,
}

impl Parse for FeagiMessageCategory {
    /// Parse the category fields. Verb sections may be omitted, in docstring order.
    fn parse(input: ParseStream) -> syn::Result<Self> {
        /// Parse `keyword` was already consumed. Read `: { "Title": Message, ... }`.
        fn parse_bucket<Message: Parse>(input: ParseStream, section: &str) -> syn::Result<HashMap<LitStr, Message>> {
            input.parse::<Token![:]>()?;
            let entries;
            braced!(entries in input);
            let mut messages = HashMap::new();
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
        let category_description = parse_property_colon_member::<kw::category_description, LitStr>(input)?;

        let read = if input.peek(kw::read) {
            input.parse::<kw::read>()?;
            parse_bucket::<FeagiMessage<true>>(input, "read")?
        } else {
            HashMap::new()
        };
        let create = if input.peek(kw::create) {
            input.parse::<kw::create>()?;
            parse_bucket::<FeagiMessage<false>>(input, "create")?
        } else {
            HashMap::new()
        };
        let edit = if input.peek(kw::edit) {
            input.parse::<kw::edit>()?;
            parse_bucket::<FeagiMessage<false>>(input, "edit")?
        } else {
            HashMap::new()
        };
        let delete = if input.peek(kw::delete) {
            input.parse::<kw::delete>()?;
            parse_bucket::<FeagiMessage<false>>(input, "delete")?
        } else {
            HashMap::new()
        };
        let patch = if input.peek(kw::patch) {
            input.parse::<kw::patch>()?;
            parse_bucket::<FeagiMessage<false>>(input, "patch")?
        } else {
            HashMap::new()
        };

        if !input.is_empty() {
            return Err(input.error("unexpected tokens in FEAGI message category"));
        }

        Ok(Self {
            category_name,
            base_path,
            category_description,
            read,
            create,
            edit,
            delete,
            patch,
        })
    }
}

impl Unparse for FeagiMessageCategory {
    /// Re-emit the category. Empty verb sections are left out.
    fn unparse(&self) -> proc_macro2::TokenStream {
        /// Append `section: { "Title": Message, ... },` when the map is not empty.
        fn extend_bucket<Message: Unparse>(stream: &mut proc_macro2::TokenStream, section: &str, messages: &HashMap<LitStr, Message>) {
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
        extend_bucket(&mut stream, "read", &self.read);
        extend_bucket(&mut stream, "create", &self.create);
        extend_bucket(&mut stream, "edit", &self.edit);
        extend_bucket(&mut stream, "delete", &self.delete);
        extend_bucket(&mut stream, "patch", &self.patch);
        stream
    }
}

//endregion

//region Feagi Message

/// Parses over a FeagiMessage with a payload. Formatted as the following
/// "MessageTitle": {
///     path: `FeagiMessagePath`,
///     path_item_descriptions: {
///         property_name: "Description String",
///     },
///     description: "Description String",
///     payload: {
///         property_name: Type, "optional_comment",
///     },
///     response: {
///         property_name: Type, "optional_comment",
///     }
/// }
/// Note that the payload is skipped for Read Messages
pub struct FeagiMessage<const NO_PAYLOAD: bool> {
    pub path: FeagiMessagePath,
    pub path_item_descriptions: PropertyDescriptors,
    pub description: LitStr,
    pub payload: StructTemplate,
    pub response: StructTemplate,
}

impl<const NO_PAYLOAD: bool> FeagiMessage<NO_PAYLOAD> {

    pub fn generate_feagi_message_structs(&self, name_base: syn::Ident) -> proc_macro2::TokenStream {
        let mut output: proc_macro2::TokenStream = proc_macro2::TokenStream::new();
        let name_path = format_ident!("{}{}", name_base, "FeagiMessagePath");
        let name_message = format_ident!("{}{}", name_base, "FeagiMessage");
        let name_null = format_ident!("{}", "NullMessageSpecification");
        let specification_derives = vec![format_ident!("{}", "Clone"), format_ident!("{}", "Debug"), format_ident!("{}", "Serialize"), format_ident!("{}", "DeserializeOwned")];

        let tokens_message_path = self.path.generate_feagi_message_path_struct_and_impls(&name_path, &name_message);

        output.extend(tokens_message_path);

        let mut all_request_fields = StructTemplate::new_empty();

        fn generate_specification(
            mut struct_template: StructTemplate,
            path_item_descriptions: &PropertyDescriptors,
            name_base: &syn::Ident,
            name_suffix: &str,
            name_null: &syn::Ident,
            specification_derives: &Vec<syn::Ident>,
            all_request_fields: &mut StructTemplate,
            include_in_message: bool,
        ) -> (syn::Ident, proc_macro2::TokenStream, Vec<syn::Ident>) {
            struct_template.overwrite_descriptions_from_property_descriptors(path_item_descriptions);
            let (name, tokens) = if struct_template.is_empty() {
                (name_null.clone(), proc_macro2::TokenStream::new())
            } else {
                let name = format_ident!("{}{}", name_base, name_suffix);
                let mut tokens = struct_template.generate_rust_struct(
                    name.clone(), None, specification_derives, RustVisibility::Public
                );
                let trait_name = format_ident!("{}", name_suffix);
                tokens.extend(quote! { impl #trait_name for #name });
                (name, tokens)
            };
            if include_in_message {
                for (k, (t, d)) in struct_template.get_map() {
                    all_request_fields.insert_property(k.clone(), t.clone(), d.clone());
                }
            }
            let field_names: Vec<syn::Ident> = struct_template.get_map().keys().cloned().collect();
            (name, tokens, field_names)
        }

        fn moved_fields(owner: &syn::Ident, fields: &[syn::Ident]) -> proc_macro2::TokenStream {
            let mut tokens = proc_macro2::TokenStream::new();
            for field in fields {
                tokens.extend(quote! { #field: #owner.#field, });
            }
            tokens
        }

        let (name_parameters, tokens_parameters, parameter_fields) = generate_specification(
            self.path.as_parameter_struct_template(),
            &self.path_item_descriptions,
            &name_base,
            "FeagiMessageParameters",
            &name_null,
            &specification_derives,
            &mut all_request_fields,
            true,
        );
        output.extend(tokens_parameters);

        let (name_queryables, tokens_queryables, queryable_fields) = generate_specification(
            self.path.as_queryable_struct_template(),
            &self.path_item_descriptions,
            &name_base,
            "FeagiMessageQueryables",
            &name_null,
            &specification_derives,
            &mut all_request_fields,
            true,
        );
        output.extend(tokens_queryables);

        let (name_payload, tokens_payload, payload_fields) = generate_specification(
            self.payload.clone(),
            &self.path_item_descriptions,
            &name_base,
            "FeagiMessagePayload",
            &name_null,
            &specification_derives,
            &mut all_request_fields,
            true,
        );
        output.extend(tokens_payload);

        let (name_response, tokens_response, _) = generate_specification(
            self.response.clone(),
            &self.path_item_descriptions,
            &name_base,
            "FeagiMessageResponse",
            &name_null,
            &specification_derives,
            &mut all_request_fields,
            false,
        );
        output.extend(tokens_response);


        let feagi_message_name = format_ident!("{}{}", name_base, "FeagiMessage");
        let feagi_message_derives = vec![format_ident!("{}", "Clone"), format_ident!("{}", "Debug")]; // doesn't need serialization
        let feagi_message_tokens = all_request_fields.generate_rust_struct(
            feagi_message_name.clone(), Some(self.description.clone()), &feagi_message_derives, RustVisibility::Public
        );

        output.extend(feagi_message_tokens);

        let parameters_binding = format_ident!("parameters");
        let queryables_binding = format_ident!("queryables");
        let payload_binding = format_ident!("payload");
        let self_binding = format_ident!("self");
        let from_parameters = moved_fields(&parameters_binding, &parameter_fields);
        let from_queryables = moved_fields(&queryables_binding, &queryable_fields);
        let from_payload = moved_fields(&payload_binding, &payload_fields);
        let to_parameters = moved_fields(&self_binding, &parameter_fields);
        let to_queryables = moved_fields(&self_binding, &queryable_fields);
        let to_payload = moved_fields(&self_binding, &payload_fields);
        // An empty specification is `NullMessageSpecification` and adds no fields.
        // Name the unused binding so the generated function does not warn.
        let consume_parameters = if parameter_fields.is_empty() { quote!(let _ = parameters;) } else { quote!() };
        let consume_queryables = if queryable_fields.is_empty() { quote!(let _ = queryables;) } else { quote!() };
        let consume_payload = if payload_fields.is_empty() { quote!(let _ = payload;) } else { quote!() };
        let consume_self = if parameter_fields.is_empty() && queryable_fields.is_empty() && payload_fields.is_empty() {
            quote!(let _ = self;)
        } else {
            quote!()
        };

        let mut new_arguments = proc_macro2::TokenStream::new();
        let mut new_members = proc_macro2::TokenStream::new();
        for (name, (field_type, _)) in all_request_fields.get_map() {
            new_arguments.extend(quote! { #name: #field_type, });
            new_members.extend(quote! { #name, });
        }

        output.extend(quote!{

            impl #feagi_message_name {
                pub fn new(#new_arguments) -> Self {
                    Self {
                        #new_members
                    }
                }
            }

            impl FeagiMessage for #feagi_message_name {
                type Path = #name_path;
                type Parameters = #name_parameters;
                type Queryables = #name_queryables;
                type Payload = #name_payload;

                fn from_message_data(parameters: Self::Parameters, queryables: Self::Queryables, payload: Self::Payload) -> Self {
                    #consume_parameters
                    #consume_queryables
                    #consume_payload
                    Self {
                        #from_parameters
                        #from_queryables
                        #from_payload
                    }
                }

                fn to_message_data(self) -> (Self::Parameters, Self::Queryables, Self::Payload) {
                    #consume_self
                    (
                        Self::Parameters { #to_parameters },
                        Self::Queryables { #to_queryables },
                        Self::Payload { #to_payload },
                    )
                }
            }

            impl FeagiMessageWithResponse for #feagi_message_name {
                type Response = #name_response;
            }

        });

        output
    }

}

impl<const NO_PAYLOAD: bool> Parse for FeagiMessage<NO_PAYLOAD> {
    /// Parse the braced message body. `"MessageTitle":` belongs to the parent entry.
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let members;
        braced!(members in input);

        let path = parse_property_colon_member::<kw::path, FeagiMessagePath>(&members)?;
        let path_item_descriptions = parse_property_colon_member::<kw::path_item_descriptions, PropertyDescriptors>(&members)?;
        let description = parse_property_colon_member::<kw::description, LitStr>(&members)?;
        let payload;
        if !NO_PAYLOAD {
            payload = parse_property_colon_member::<kw::payload, StructTemplate>(&members)?;
        } else {
            payload = StructTemplate::new_empty();
        }

        let response = parse_property_colon_member::<kw::response, StructTemplate>(&members)?;

        if !members.is_empty() {
            return Err(members.error("unexpected tokens in FEAGI message"));
        }

        Ok(Self {
            path,
            path_item_descriptions,
            description,
            payload,
            response,
        })
    }
}

impl<const NO_PAYLOAD: bool> Unparse for FeagiMessage<NO_PAYLOAD> {
    /// Re-emit the braced message body so it can be parsed again.
    fn unparse(&self) -> proc_macro2::TokenStream {
        let path = self.path.unparse();
        let path_item_descriptions = self.path_item_descriptions.unparse();
        let description = &self.description;
        let payload = self.payload.unparse();
        let response = self.response.unparse();

        quote! {
            {
                path: #path,
                path_item_descriptions: #path_item_descriptions,
                description: #description,
                payload: #payload,
                response: #response,
            }
        }
    }
}

//endregion

//region FeagiMessagePath

/// Parses over a single string to get the pathing out, with the formatting:
/// "static_1/static_2/:parameter_1<ParameterType1>/static_3?query_1<QueryType1>&query_2<QueryType2>"
pub struct FeagiMessagePath {
    leaf: Vec<MessagePathElement>,
}

impl FeagiMessagePath {
    pub fn full_message_path(&self, category: &LitStr) -> LitStr {
        let path = self.as_lit_str();
        LitStr::new(&format!("{}/{}", category.value(), path.value()), Span::call_site())
    }

    pub fn as_lit_str(&self) -> LitStr {
        let string_out: String = self.leaf.iter().map(|l| l.to_string()).collect::<Vec<_>>().join("/");
        LitStr::new(&string_out, Span::call_site())
    }

    /// Outputs the parameters as a StructTemplate (No comments)
    pub fn as_parameter_struct_template(&self) -> StructTemplate {
        let mut output = StructTemplate::new_empty();
        for element in &self.leaf {
            match element {
                MessagePathElement::ParameterOfName(n, p) => {
                    let ident = lit_str_to_ident(n).unwrap();
                    output.insert_property(ident, p.clone(), None);
                }
                _ => {} // do nothing
            }
        };
        output
    }

    /// Outputs the queryable as a StructTemplate (No comments)
    pub fn as_queryable_struct_template(&self) -> StructTemplate {
        let mut output = StructTemplate::new_empty();
        for element in &self.leaf {
            match element {
                MessagePathElement::QueryableOfName(n, q) => {
                    let ident = lit_str_to_ident(n).unwrap();
                    output.insert_property(ident, q.clone(), None);
                }
                _ => {} // do nothing
            }
        };
        output
    }

    /// Generate the FeagiMessagePath impl struct rust code for this path
    pub fn generate_feagi_message_path_struct_and_impls(&self, full_struct_name: &syn::Ident, name_of_message_struct: &syn::Ident) -> proc_macro2::TokenStream {
        let mut output = proc_macro2::TokenStream::new();
        let elements: Vec<proc_macro2::TokenStream> = self.leaf.iter().map(MessagePathElement::as_rust_message_path_element_enum).collect();

        output.extend(quote! {

            ##[derive(Clone, Copy, Debug)]
            pub struct #full_struct_name;

            impl FeagiMessagePath for #full_struct_name {
                const PATH: &'static [MessagePathElement] = &[ #( #elements )* ];

                type Message = #name_of_message_struct;
            }
        });

        output

    }
}

impl Parse for FeagiMessagePath {
    /// Parse `"static/:parameter<Type>/static?query<Type>&query<Type>"` into path elements.
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let path_str: LitStr = input.parse()?;
        let leaf = message_path_elements(&path_str)?;
        Ok(Self { leaf })
    }
}

/// Split a path literal into static segments, parameters, and queries.
///
/// `/`, `?`, and `&` separate elements only outside `<...>`, so a type such as
/// `Vec<&str>` stays inside the element it belongs to. `>` in `->` is not a
/// closing angle bracket.
fn message_path_elements(path_str: &LitStr) -> syn::Result<Vec<MessagePathElement>> {
    let span = path_str.span();
    let raw = path_str.value();
    if raw.is_empty() {
        return Err(syn::Error::new(span, "FEAGI message path must not be empty"));
    }

    let mut leaf = Vec::new();
    let mut depth = 0usize;
    let mut prev: Option<char> = None;
    let mut in_query = false;
    let mut part_start = 0usize;

    for (index, ch) in raw.char_indices() {
        match ch {
            '<' => depth += 1,
            '>' if prev != Some('-') => {
                if depth == 0 {
                    return Err(syn::Error::new(span, "FEAGI message path has '>' without a matching '<'"));
                }
                depth -= 1;
            }
            '/' if depth == 0 => {
                if in_query {
                    return Err(syn::Error::new(span, "FEAGI message path query string cannot contain '/'"));
                }
                push_path_part(&mut leaf, &raw[part_start..index], false, span)?;
                part_start = index + ch.len_utf8();
            }
            '?' if depth == 0 => {
                if in_query {
                    return Err(syn::Error::new(span, "FEAGI message path has more than one '?'"));
                }
                push_path_part(&mut leaf, &raw[part_start..index], false, span)?;
                in_query = true;
                part_start = index + ch.len_utf8();
            }
            '&' if depth == 0 => {
                if !in_query {
                    return Err(syn::Error::new(span, "FEAGI message path uses '&' only between query items"));
                }
                push_path_part(&mut leaf, &raw[part_start..index], true, span)?;
                part_start = index + ch.len_utf8();
            }
            _ => {}
        }
        prev = Some(ch);
    }

    if depth != 0 {
        return Err(syn::Error::new(span, "FEAGI message path has an unclosed '<'"));
    }
    push_path_part(&mut leaf, &raw[part_start..], in_query, span)?;
    Ok(leaf)
}

/// Append one `/` segment, or one query item when `in_query` is set.
fn push_path_part(leaf: &mut Vec<MessagePathElement>, part: &str, in_query: bool, span: Span) -> syn::Result<()> {
    if part.is_empty() {
        let message = if in_query {
            "FEAGI message path contains an empty query"
        } else {
            "FEAGI message path contains an empty segment"
        };
        return Err(syn::Error::new(span, message));
    }
    if in_query {
        leaf.push(parse_query(part, span)?);
    } else {
        leaf.push(parse_path_segment(part, span)?);
    }
    Ok(())
}

/// Parse `static_name` or `:parameter_name<Type>`.
fn parse_path_segment(segment: &str, span: Span) -> syn::Result<MessagePathElement> {
    if let Some(spec) = segment.strip_prefix(':') {
        let (name, parameter_type) = parse_name_and_type(spec, span)?;
        return Ok(MessagePathElement::ParameterOfName(LitStr::new(name, span), parameter_type));
    }

    if segment.chars().any(|ch| matches!(ch, '<' | '>' | '&' | ':') || ch.is_whitespace()) {
        return Err(syn::Error::new(
            span,
            format!("static path segment `{segment}` contains path syntax; parameters use `:name<Type>`"),
        ));
    }

    Ok(MessagePathElement::StaticPath(LitStr::new(segment, span)))
}

/// Parse `query_name<Type>`.
fn parse_query(spec: &str, span: Span) -> syn::Result<MessagePathElement> {
    if spec.starts_with(':') {
        return Err(syn::Error::new(
            span,
            format!("query `{spec}` must be `name<Type>` without a leading ':'"),
        ));
    }
    let (name, query_type) = parse_name_and_type(spec, span)?;
    Ok(MessagePathElement::QueryableOfName(LitStr::new(name, span), query_type))
}

/// Split `name<Type>` and parse both sides. `Type` may itself contain angle brackets.
fn parse_name_and_type(spec: &str, span: Span) -> syn::Result<(&str, Type)> {
    let (name, type_src) = split_name_and_type(spec, span)?;
    syn::parse_str::<syn::Ident>(name).map_err(|_| syn::Error::new(span, format!("`{name}` is not a valid path name")))?;
    let parsed_type =
        syn::parse_str::<Type>(type_src).map_err(|err| syn::Error::new(span, format!("invalid type `{type_src}` in FEAGI message path: {err}")))?;
    Ok((name, parsed_type))
}

/// Return the name and the source inside the outer `<...>` pair.
fn split_name_and_type(spec: &str, span: Span) -> syn::Result<(&str, &str)> {
    let Some(open) = spec.find('<') else {
        return Err(syn::Error::new(span, format!("`{spec}` is missing a <Type>")));
    };
    let name = &spec[..open];
    if name.is_empty() {
        return Err(syn::Error::new(span, format!("`{spec}` is missing a name before <Type>")));
    }

    let mut depth = 0usize;
    let mut close_at = None;
    let mut prev: Option<char> = None;
    for (offset, ch) in spec[open..].char_indices() {
        match ch {
            '<' => depth += 1,
            '>' if prev != Some('-') => {
                depth -= 1;
                if depth == 0 {
                    close_at = Some(open + offset);
                    break;
                }
            }
            _ => {}
        }
        prev = Some(ch);
    }

    let Some(close_at) = close_at else {
        return Err(syn::Error::new(span, format!("`{spec}` is missing a closing '>'")));
    };
    if close_at + 1 != spec.len() {
        return Err(syn::Error::new(span, format!("`{spec}` has tokens after the type")));
    }
    let type_src = &spec[open + 1..close_at];
    if type_src.is_empty() {
        return Err(syn::Error::new(span, format!("`{spec}` has an empty type")));
    }
    Ok((name, type_src))
}

impl Unparse for FeagiMessagePath {
    fn unparse(&self) -> proc_macro2::TokenStream {
        let string_out = self.as_lit_str();
        quote! { #string_out }
    }
}

pub enum MessagePathElement {
    StaticPath(LitStr),
    ParameterOfName(LitStr, Type),
    QueryableOfName(LitStr, Type),
}

impl MessagePathElement {
    pub fn to_string(&self) -> String {
        match self {
            MessagePathElement::StaticPath(s) => s.value(),
            MessagePathElement::ParameterOfName(s, t) => {
                format!("{}<{}>", s.value(), quote!(#t).to_string())
            }
            MessagePathElement::QueryableOfName(s, t) => {
                format!("{}<{}>", s.value(), quote!(#t).to_string())
            }
        }
    }

    pub fn as_rust_message_path_element_enum(&self) -> proc_macro2::TokenStream {
        let mut output = proc_macro2::TokenStream::new();

        match self {
            MessagePathElement::StaticPath(s) => {
                output.extend(quote!{ MessagePathElement::StaticPath(#s), });
                output
            }
            MessagePathElement::ParameterOfName(s, _) => {
                output.extend(quote!{ MessagePathElement::ParameterOfName(#s), });
                output
            }
            MessagePathElement::QueryableOfName(s, _) => {
                output.extend(quote!{ MessagePathElement::QueryableOfName(#s), });
                output
            }
        }

    }
}

//endregion
