use crate::common::lit_str_to_ident;
use crate::feagi_messages::message_path_element::MessagePathElement;
use crate::feagi_messages::request_response::common_structs::{FeagiRequestMessageMethod, FeagiRequestMessageU8};
use crate::template_parsing::struct_template::StructTemplate;
use crate::templates::Unparse;
use proc_macro2::Span;
use quote::{format_ident, quote};
use syn::parse::{Parse, ParseStream};
use syn::{LitStr, Type};

//region Feagi Request Message Path Specification Struct Names

/// Contains the rust struct name generated from a FeagiRequestMessagePath.
/// Whether parameter or queryable structs are actually emitted is decided by
/// [`FeagiRequestMessagePath`], which holds the path-element context.
#[derive(Clone)]
pub struct FeagiRequestMessagePathRustStructNames {
    pub base_name: syn::Ident,
    pub method: FeagiRequestMessageMethod,
}

impl FeagiRequestMessagePathRustStructNames {

    /// Build the name set from the message title and HTTP-equivalent method.
    pub fn new(base_name: syn::Ident, method: FeagiRequestMessageMethod) -> Self {
        Self { base_name, method }
    }

    pub fn feagi_message_path_struct_name(&self) -> syn::Ident {
        format_ident!("{}{}{}", self.method.as_str(), self.base_name, "FeagiRequestMessagePath")
    }
}

//endregion

//region Feagi Request Message Path

/// Parses over a single string to get the pathing out, with the formatting:
/// "static_1/static_2/:parameter_1<ParameterType1>/static_3?query_1<QueryType1>&query_2<QueryType2>"
pub struct FeagiRequestMessagePath<const METHOD_U8: FeagiRequestMessageU8> {
    leaf: Vec<MessagePathElement>,
}

impl<const METHOD: FeagiRequestMessageU8> FeagiRequestMessagePath<METHOD> {

    /// Whether this path defines a parameter struct.
    pub fn is_parameter_struct_generated(&self) -> bool {
        !self.as_parameter_struct_template().is_empty()
    }

    /// Whether this path defines a queryable struct.
    pub fn is_queryable_struct_generated(&self) -> bool {
        !self.as_queryable_struct_template().is_empty()
    }

    /// Struct names this path will generate. Existence of each named struct
    /// is queried from this path, not from the name set.
    pub fn calculate_feagi_message_path_struct_names(&self, name_base: syn::Ident) -> FeagiRequestMessagePathRustStructNames {
        let _ = self;
        FeagiRequestMessagePathRustStructNames::new(
            name_base,
            FeagiRequestMessageMethod::from_u8(METHOD),
        )
    }

    pub fn full_message_path(&self, category: &LitStr) -> LitStr {
        let path = self.as_lit_str();
        LitStr::new(&format!("{}/{}", category.value(), path.value()), Span::call_site())
    }

    // TODO full_message_path_with_method(&self, category: &LitStr) -> LitStr should append the method at the end

    pub fn as_lit_str(&self) -> LitStr {
        let string_out: String = self.leaf.iter().map(|l| l.to_string()).collect::<Vec<_>>().join("/");
        LitStr::new(&string_out, Span::call_site())
    }

    /// Path and query elements in template order.
    pub fn elements(&self) -> &[MessagePathElement] {
        &self.leaf
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
        }
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
        }
        output
    }

    /// Generate the FeagiMessagePath impl struct rust code for this path
    pub fn generate_feagi_message_path_struct_and_impls(
        &self,
        path_struct_names: &FeagiRequestMessagePathRustStructNames,
        name_of_message_struct: &syn::Ident,
    ) -> proc_macro2::TokenStream {
        let mut output = proc_macro2::TokenStream::new();
        let elements: Vec<proc_macro2::TokenStream> = self.leaf.iter().map(MessagePathElement::as_rust_message_path_element_enum).collect();

        let full_struct_name = path_struct_names.feagi_message_path_struct_name();
        let method_name = format_ident!("{}", FeagiRequestMessageMethod::from_u8(METHOD).as_str());

        output.extend(quote! {

            #[derive(Clone, Copy, Debug)]
            pub struct #full_struct_name;

            impl FeagiMessagePath for #full_struct_name {
                const PATH: &'static [MessagePathElement] = &[ #( #elements )* ];

                type Message = #name_of_message_struct;
            }

            impl FeagiRequestMessagePath for #full_struct_name {
                const METHOD: FeagiMessageRequestMethod = FeagiMessageRequestMethod::#method_name;
            }
        });

        output
    }
}

impl<const METHOD: FeagiRequestMessageU8> Parse for FeagiRequestMessagePath<METHOD> {
    /// Parse `"static/:parameter<Type>/static?query<Type>&query<Type>"` into path elements.
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let path_str: LitStr = input.parse()?;
        let leaf = message_path_elements(&path_str)?;
        Ok(Self { leaf })
    }
}

impl<const METHOD: FeagiRequestMessageU8> Unparse for FeagiRequestMessagePath<METHOD> {
    fn unparse(&self) -> proc_macro2::TokenStream {
        let string_out = self.as_lit_str();
        quote! { #string_out }
    }
}

//endregion

//region Internal

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

//endregion