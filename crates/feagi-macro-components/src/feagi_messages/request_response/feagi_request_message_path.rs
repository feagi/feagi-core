use crate::common::{basis_path, lit_str_to_ident};
use crate::feagi_messages::message_path_element::MessagePathElement;
use crate::feagi_messages::request_response::common_structs::{FeagiRequestMessageMethod, FeagiRequestMessageU8};
use crate::feagi_messages::request_response::feagi_request_message_category::FeagiRequestMessageCategoryContext;
use crate::template_parsing::struct_template::StructTemplate;
use crate::templates::Unparse;
use proc_macro2::Span;
use quote::{format_ident, quote};
use syn::parse::{Parse, ParseStream};
use syn::{Index, LitStr, Type};

//region Feagi Request Message Path Specification Struct Names

/// Contains the rust struct name generated from a FeagiRequestMessagePath.
/// Whether a parameter struct is actually emitted is decided by
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
/// "static_1/$configurable_1/:parameter_1<ParameterType1>/static_3"
pub struct FeagiRequestMessagePath<const METHOD_U8: FeagiRequestMessageU8> {
    leaf: Vec<MessagePathElement>,
}

impl<const METHOD: FeagiRequestMessageU8> FeagiRequestMessagePath<METHOD> {
    /// Whether this path defines a parameter struct.
    pub fn is_parameter_struct_generated(&self) -> bool {
        !self.as_parameter_struct_template().is_empty()
    }

    /// Struct names this path will generate. Existence of each named struct
    /// is queried from this path, not from the name set.
    pub fn calculate_feagi_message_path_struct_names(&self, name_base: syn::Ident) -> FeagiRequestMessagePathRustStructNames {
        let _ = self;
        FeagiRequestMessagePathRustStructNames::new(name_base, FeagiRequestMessageMethod::from_u8(METHOD))
    }

    pub fn full_message_path(&self, category: &LitStr) -> LitStr {
        let path = self.as_lit_str();
        LitStr::new(&format!("{}/{}", category.value(), path.value()), Span::call_site())
    }

    pub fn as_lit_str(&self) -> LitStr {
        LitStr::new(&path_elements_to_string(&self.leaf), Span::call_site())
    }

    /// Path elements in template order.
    pub fn elements(&self) -> &[MessagePathElement] {
        &self.leaf
    }

    /// Outputs the parameters as a StructTemplate (No comments)
    pub fn as_parameter_struct_template(&self) -> StructTemplate {
        let mut output = StructTemplate::new_empty();
        for element in &self.leaf {
            if let MessagePathElement::ParameterOfName(n, p) = element {
                let ident = lit_str_to_ident(n).unwrap();
                output.insert_property(ident, p.clone(), None);
            }
        }
        output
    }

    /// Generate the path struct and its `FeagiMessagePath` impl. The category's base path is
    /// prepended to this path. `parameters_type` is the message's parameters struct, or `()`
    pub fn generate_feagi_message_path_struct_and_impls(
        &self,
        path_struct_names: &FeagiRequestMessagePathRustStructNames,
        parameters_type: &syn::Type,
        category: &FeagiRequestMessageCategoryContext,
    ) -> proc_macro2::TokenStream {
        let basis = basis_path();
        let path_module = quote!(#basis::feagi_message::feagi_message_path);
        let static_element = quote!(#path_module::configurables::FeagiMessagePathStaticElement);
        let parameter_element = quote!(#path_module::parameters::FeagiMessagePathParameterElement);

        let mut full_types = Vec::new();
        let mut full_values = Vec::new();
        let mut slotted_types = Vec::new();
        let mut slotted_values = Vec::new();
        let mut write_segments = Vec::new();
        let mut parse_segments = Vec::new();
        let mut parameter_fields = Vec::new();
        let mut uses_configurables = false;
        let mut uses_parameters = false;

        for (index, element) in category.base_path_elements.iter().chain(self.leaf.iter()).enumerate() {
            let tuple_index = Index::from(index);
            match element {
                MessagePathElement::StaticPath(text) => {
                    full_types.push(quote!(&'static str));
                    full_values.push(quote!(#text));
                    slotted_types.push(quote!(&'static str));
                    slotted_values.push(quote!(full.#tuple_index));
                    write_segments.push(quote!(out.write_str(#text)?;));
                    parse_segments.push(quote! {
                        if segments.next()? != #text {
                            return None;
                        }
                    });
                }
                MessagePathElement::ConfigurableOfName(name) => {
                    let field = lit_str_to_ident(name).unwrap();
                    uses_configurables = true;
                    full_types.push(quote!(#path_module::configurables::ConfigurableSlot));
                    full_values.push(quote!(#path_module::configurables::ConfigurableSlot::new(#name)));
                    slotted_types.push(quote!(&'static str));
                    slotted_values.push(quote!(#static_element::to_str_slice(&configurables.#field)));
                    write_segments.push(quote!(out.write_str(#static_element::to_str_slice(&configurables.#field))?;));
                    parse_segments.push(quote! {
                        if segments.next()? != #static_element::to_str_slice(&configurables.#field) {
                            return None;
                        }
                    });
                }
                MessagePathElement::ParameterOfName(name, parameter_type) => {
                    let field = lit_str_to_ident(name).unwrap();
                    uses_parameters = true;
                    let slot = quote!(#path_module::parameters::MessagePathSlotElement<#parameter_type>);
                    full_types.push(slot.clone());
                    full_values.push(quote!(#path_module::parameters::MessagePathSlotElement::new(#name)));
                    slotted_types.push(slot);
                    slotted_values.push(quote!(full.#tuple_index));
                    write_segments.push(quote!(#parameter_element::write_path_segment(&parameters.#field, out)?;));
                    parse_segments.push(quote! {
                        let #field: #parameter_type = ::core::str::FromStr::from_str(segments.next()?).ok()?;
                    });
                    parameter_fields.push(field);
                }
            }
        }

        let mut write_body = proc_macro2::TokenStream::new();
        for (index, segment) in write_segments.iter().enumerate() {
            if index != 0 {
                write_body.extend(quote!(out.write_str("/")?;));
            }
            write_body.extend(segment.clone());
        }

        let consume_configurables = if uses_configurables { quote!() } else { quote!(let _ = configurables;) };
        let consume_parameters = if uses_parameters { quote!() } else { quote!(let _ = parameters;) };
        let parsed_parameters = if uses_parameters {
            quote!(#parameters_type { #( #parameter_fields, )* })
        } else {
            quote!(())
        };

        let full_struct_name = path_struct_names.feagi_message_path_struct_name();
        let configurables_struct_name = &category.configurables_struct_name;

        quote! {
            #[derive(Clone, Copy, Debug)]
            pub struct #full_struct_name;

            impl #path_module::FeagiMessagePath for #full_struct_name {
                type Configurables = #configurables_struct_name;
                type Parameters = #parameters_type;
                type FullMessagePathBaseTuple = ( #( #full_types, )* );
                const FULL_MESSAGE_PATH: Self::FullMessagePathBaseTuple = ( #( #full_values, )* );
                type SlottedParameterMessagePath = ( #( #slotted_types, )* );

                fn to_slotted_parameter_message_path(configurables: &'static Self::Configurables) -> Self::SlottedParameterMessagePath {
                    #[allow(unused_variables)]
                    let full = Self::FULL_MESSAGE_PATH;
                    #consume_configurables
                    ( #( #slotted_values, )* )
                }

                fn write_message_path<W: ::core::fmt::Write>(
                    configurables: &Self::Configurables,
                    parameters: &Self::Parameters,
                    out: &mut W,
                ) -> ::core::fmt::Result {
                    #consume_configurables
                    #consume_parameters
                    #write_body
                    Ok(())
                }

                fn parse_message_path<'a, I: ::core::iter::Iterator<Item = &'a str>>(
                    configurables: &Self::Configurables,
                    mut segments: I,
                ) -> ::core::option::Option<Self::Parameters> {
                    #consume_configurables
                    #( #parse_segments )*
                    if segments.next().is_some() {
                        return None;
                    }
                    Some(#parsed_parameters)
                }
            }
        }
    }
}

impl<const METHOD: FeagiRequestMessageU8> Parse for FeagiRequestMessagePath<METHOD> {
    /// Parse `"static/$configurable/:parameter<Type>/static"` into path elements.
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

/// Rebuild the template string of path elements
pub fn path_elements_to_string(elements: &[MessagePathElement]) -> String {
    elements.iter().map(MessagePathElement::to_string).collect::<Vec<_>>().join("/")
}

/// Split a path literal into static segments, configurables, and parameters.
///
/// `/` separates elements only outside `<...>`, so a type such as `Vec<&str>` stays inside the
/// element it belongs to. `>` in `->` is not a closing angle bracket.
pub fn message_path_elements(path_str: &LitStr) -> syn::Result<Vec<MessagePathElement>> {
    let span = path_str.span();
    let raw = path_str.value();
    if raw.is_empty() {
        return Err(syn::Error::new(span, "FEAGI message path must not be empty"));
    }

    let mut leaf = Vec::new();
    let mut depth = 0usize;
    let mut prev: Option<char> = None;
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
                push_path_part(&mut leaf, &raw[part_start..index], span)?;
                part_start = index + ch.len_utf8();
            }
            '?' | '&' if depth == 0 => {
                return Err(syn::Error::new(
                    span,
                    "FEAGI message paths have no queryables. Put the field in the message `payload` instead (Read messages send it as the URL query in REST)",
                ));
            }
            _ => {}
        }
        prev = Some(ch);
    }

    if depth != 0 {
        return Err(syn::Error::new(span, "FEAGI message path has an unclosed '<'"));
    }
    push_path_part(&mut leaf, &raw[part_start..], span)?;
    Ok(leaf)
}

/// Append one `/` segment
fn push_path_part(leaf: &mut Vec<MessagePathElement>, part: &str, span: Span) -> syn::Result<()> {
    if part.is_empty() {
        return Err(syn::Error::new(span, "FEAGI message path contains an empty segment"));
    }
    leaf.push(parse_path_segment(part, span)?);
    Ok(())
}

/// Parse `static_name`, `$configurable_name`, or `:parameter_name<Type>`.
fn parse_path_segment(segment: &str, span: Span) -> syn::Result<MessagePathElement> {
    if let Some(spec) = segment.strip_prefix(':') {
        let (name, parameter_type) = parse_name_and_type(spec, span)?;
        return Ok(MessagePathElement::ParameterOfName(LitStr::new(name, span), parameter_type));
    }

    if let Some(name) = segment.strip_prefix('$') {
        syn::parse_str::<syn::Ident>(name).map_err(|_| syn::Error::new(span, format!("`{name}` is not a valid configurable name")))?;
        return Ok(MessagePathElement::ConfigurableOfName(LitStr::new(name, span)));
    }

    if segment.chars().any(|ch| matches!(ch, '<' | '>' | ':' | '$') || ch.is_whitespace()) {
        return Err(syn::Error::new(
            span,
            format!("static path segment `{segment}` contains path syntax; parameters use `:name<Type>` and configurables use `$name`"),
        ));
    }

    Ok(MessagePathElement::StaticPath(LitStr::new(segment, span)))
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
