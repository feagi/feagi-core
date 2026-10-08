use std::collections::HashMap;
use crate::common::parse_property_colon_member;
use crate::template_parsing::property_descriptors::PropertyDescriptors;
use crate::template_parsing::struct_template::StructTemplate;
use crate::templates::Unparse;
use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::{braced, LitStr, Type};

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
    /// What type of error to use (FeagiError)
    pub read: HashMap<LitStr, FeagiMessageWithoutPayload>,
    pub create: HashMap<LitStr, FeagiMessageWithPayload>,
    pub edit: HashMap<LitStr, FeagiMessageWithPayload>,
    pub delete: HashMap<LitStr, FeagiMessageWithPayload>,
    pub patch: HashMap<LitStr, FeagiMessageWithPayload>,
}

impl Parse for FeagiMessageCategory {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        

    }
}

impl Unparse for FeagiMessageCategory {
    fn unparse(&self) -> TokenStream {
        todo!()
    }
}

//endregion

//region Feagi Message

/// Parses over a FeagiMessage without a payload. Formatted as the following
/// "MessageTitle": {
///     path: `FeagiMessagePath`,
///     path_item_descriptions: {
///         property_name: "Description String",
///     },
///     description: "Description String",
///     response: {
///         property_name: Type, "optional_comment",
///     }
/// }
pub struct FeagiMessageWithoutPayload {
    pub path: FeagiMessagePath,
    pub path_item_descriptions: PropertyDescriptors,
    pub description: LitStr,
    pub response: StructTemplate,
}

impl FeagiMessageWithoutPayload {

}

impl Parse for FeagiMessageWithoutPayload {
    /// Parse the braced message body. `"MessageTitle":` belongs to the parent entry.
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let members;
        braced!(members in input);

        let path = parse_property_colon_member::<kw::path, FeagiMessagePath>(&members)?;
        let path_item_descriptions = parse_property_colon_member::<kw::path_item_descriptions, PropertyDescriptors>(&members)?;
        let description = parse_property_colon_member::<kw::description, LitStr>(&members)?;
        let response = parse_property_colon_member::<kw::response, StructTemplate>(&members)?;

        if !members.is_empty() {
            return Err(members.error("unexpected tokens in FEAGI message"));
        }

        Ok(Self {
            path,
            path_item_descriptions,
            description,
            response,
        })
    }
}

impl Unparse for FeagiMessageWithoutPayload {
    /// Re-emit the braced message body so it can be parsed again.
    fn unparse(&self) -> proc_macro2::TokenStream {
        let path = self.path.unparse();
        let path_item_descriptions = self.path_item_descriptions.unparse();
        let description = &self.description;
        let response = self.response.unparse();

        quote! {
            {
                path: #path,
                path_item_descriptions: #path_item_descriptions,
                description: #description,
                response: #response,
            }
        }
    }
}



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
pub struct FeagiMessageWithPayload {
    pub path: FeagiMessagePath,
    pub path_item_descriptions: PropertyDescriptors,
    pub description: LitStr,
    pub payload: StructTemplate,
    pub response: StructTemplate,
}

impl FeagiMessageWithPayload {

}

impl Parse for FeagiMessageWithPayload {
    /// Parse the braced message body. `"MessageTitle":` belongs to the parent entry.
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let members;
        braced!(members in input);

        let path = parse_property_colon_member::<kw::path, FeagiMessagePath>(&members)?;
        let path_item_descriptions = parse_property_colon_member::<kw::path_item_descriptions, PropertyDescriptors>(&members)?;
        let description = parse_property_colon_member::<kw::description, LitStr>(&members)?;
        let payload = parse_property_colon_member::<kw::payload, StructTemplate>(&members)?;
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

impl Unparse for FeagiMessageWithPayload {
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
}

//endregion
