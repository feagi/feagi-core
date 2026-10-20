//! Turns one `FeagiMessageCategory` into a const `CategoryApi` expression.
//!
//! The expression is emitted for the call site, where the OpenAPI types are in scope.

use feagi_macro_components::feagi_messages::message_path_element::MessagePathElement;
use feagi_macro_components::feagi_messages::request_response::common_structs::{FeagiRequestMessageMethod, FeagiRequestMessageU8};
use feagi_macro_components::feagi_messages::request_response::feagi_request_message::FeagiRequestMessage;
use feagi_macro_components::feagi_messages::request_response::feagi_request_message_category::FeagiRequestMessageCategory;
use feagi_macro_components::template_parsing::property_descriptors::PropertyDescriptors;
use feagi_macro_components::template_parsing::struct_template::StructTemplate;
use proc_macro2::{Ident, Span, TokenStream};
use quote::{format_ident, quote};
use feagi_macro_components::IndexMap;
use syn::{GenericArgument, LitStr, PathArguments, Type};

/// Expression that evaluates to one `CategoryApi`.
pub fn category_api_tokens(category: &FeagiRequestMessageCategory) -> TokenStream {
    let mut operations = TokenStream::new();
    let mut operation_names = TokenStream::new();
    let mut index = 0usize;

    push_bucket(category, &category.read, &mut operations, &mut operation_names, &mut index);
    push_bucket(category, &category.create, &mut operations, &mut operation_names, &mut index);
    push_bucket(category, &category.edit, &mut operations, &mut operation_names, &mut index);
    push_bucket(category, &category.delete, &mut operations, &mut operation_names, &mut index);
    push_bucket(category, &category.patch, &mut operations, &mut operation_names, &mut index);

    let name = &category.category_name;
    let description = &category.category_description;
    if index == 0 {
        return quote! {
            CategoryApi::new(#name, #description, &[])
        };
    }

    quote! {
        {
            #operations
            const OPERATIONS: &[OperationApi] = &[ #operation_names ];
            CategoryApi::new(#name, #description, OPERATIONS)
        }
    }
}

fn push_bucket<const METHOD: FeagiRequestMessageU8>(
    category: &FeagiRequestMessageCategory,
    messages: &IndexMap<LitStr, FeagiRequestMessage<METHOD>>,
    operations: &mut TokenStream,
    operation_names: &mut TokenStream,
    index: &mut usize,
) {
    let mut entries: Vec<_> = messages.iter().collect();
    entries.sort_by(|left, right| left.0.value().cmp(&right.0.value()));
    for (title, message) in entries {
        let const_name = format_ident!("OP_{index}");
        let operation = operation_tokens(category, title, message);
        operations.extend(quote! {
            const #const_name: OperationApi = #operation;
        });
        operation_names.extend(quote!(#const_name,));
        *index += 1;
    }
}

fn operation_tokens<const METHOD: FeagiRequestMessageU8>(
    category: &FeagiRequestMessageCategory,
    _title: &LitStr,
    message: &FeagiRequestMessage<METHOD>,
) -> TokenStream {
    let method = method_tokens(METHOD);
    let description = &message.description;
    let tag = &category.category_name;
    let mut prelude = TokenStream::new();
    let mut segments = TokenStream::new();
    let mut schema_index = 0usize;

    for segment in split_literals(&category.base_path) {
        segments.extend(quote!(PathSegment::Literal(#segment),));
    }

    // TODO Read messages send their payload as the URL query, so its fields belong here. Left
    // empty until OpenAPI is built from the traits
    let query_items = TokenStream::new();
    for element in message.path.elements() {
        match element {
            MessagePathElement::StaticPath(text) => {
                segments.extend(quote!(PathSegment::Literal(#text),));
            }
            // TODO Only known at runtime. Shown by name until OpenAPI is built from the traits
            MessagePathElement::ConfigurableOfName(name) => {
                let text = LitStr::new(&format!("${}", name.value()), name.span());
                segments.extend(quote!(PathSegment::Literal(#text),));
            }
            MessagePathElement::ParameterOfName(name, ty) => {
                let const_name = format_ident!("PATH_SCHEMA_{schema_index}");
                let schema = schema_value_tokens(ty, None);
                let description = description_tokens(lookup_doc(&message.path_item_descriptions, name));
                prelude.extend(quote! {
                    const #const_name: Schema = #schema;
                });
                segments.extend(quote! {
                    PathSegment::Parameter(PathParameter::new(#name, #description, &#const_name)),
                });
                schema_index += 1;
            }
        }
    }

    let (body_const, body_expr) = optional_object("BODY", &message.payload, &message.path_item_descriptions);
    let (response_const, response_expr) = optional_object("RESPONSE", &message.response, &message.path_item_descriptions);

    quote! {
        {
            #prelude
            const PATH: &[PathSegment] = &[ #segments ];
            const QUERY: &[QueryParameter] = &[ #query_items ];
            #body_const
            #response_const
            OperationApi::new(#method, PATH, #description, QUERY, #body_expr, #response_expr, #tag)
        }
    }
}

fn optional_object(const_name: &str, template: &StructTemplate, descriptors: &PropertyDescriptors) -> (TokenStream, TokenStream) {
    if template.is_empty() {
        return (TokenStream::new(), quote!(None));
    }
    let const_name = Ident::new(const_name, Span::call_site());
    let schema = object_schema_tokens(&sorted_fields(template, descriptors));
    (
        quote! {
            const #const_name: Schema = #schema;
        },
        quote!(Some(&#const_name)),
    )
}

fn object_schema_tokens(fields: &[(Ident, Type, Option<LitStr>)]) -> TokenStream {
    let mut consts = TokenStream::new();
    let mut properties = TokenStream::new();
    let mut required = TokenStream::new();
    for (index, (name, ty, doc)) in fields.iter().enumerate() {
        let const_name = format_ident!("FIELD_{index}");
        let (inner, is_required) = peel_option(ty);
        let schema = schema_value_tokens(inner, doc.as_ref());
        let name_lit = LitStr::new(&name.to_string(), name.span());
        consts.extend(quote! {
            const #const_name: Schema = #schema;
        });
        properties.extend(quote! {
            ObjectField::new(#name_lit, &#const_name),
        });
        if is_required {
            required.extend(quote!(#name_lit,));
        }
    }
    quote! {
        {
            #consts
            const PROPERTIES: &[ObjectField] = &[ #properties ];
            const REQUIRED_FIELDS: &[&str] = &[ #required ];
            Schema::object(None, PROPERTIES, REQUIRED_FIELDS)
        }
    }
}

/// Maps a template type onto a `Schema` expression.
///
/// `bool`, integer types, `f32`/`f64`, `String`/`str`/`char`, `Vec<T>`, slices, and arrays are mapped.
/// `Option<T>` uses the schema of `T`. Any other type becomes `Schema::opaque`.
fn schema_value_tokens(ty: &Type, description: Option<&LitStr>) -> TokenStream {
    if let Type::Reference(reference) = ty {
        return schema_value_tokens(&reference.elem, description);
    }
    if let Some(inner) = generic_inner(ty, "Option") {
        return schema_value_tokens(inner, description);
    }

    let desc = description_tokens(description.cloned());
    if is_plain_name(ty, "bool") {
        return quote!(Schema::boolean(#desc));
    }
    if is_integer(ty) {
        return quote!(Schema::integer(#desc));
    }
    if is_plain_name(ty, "f32") || is_plain_name(ty, "f64") {
        return quote!(Schema::number(#desc));
    }
    if is_plain_name(ty, "String") || is_plain_name(ty, "str") || is_plain_name(ty, "char") {
        return quote!(Schema::string(#desc));
    }
    if let Some(inner) = generic_inner(ty, "Vec") {
        let items = schema_value_tokens(inner, None);
        return quote! {
            {
                const ITEMS: Schema = #items;
                Schema::array(#desc, &ITEMS)
            }
        };
    }
    if let Type::Array(array) = ty {
        let items = schema_value_tokens(&array.elem, None);
        return quote! {
            {
                const ITEMS: Schema = #items;
                Schema::array(#desc, &ITEMS)
            }
        };
    }
    if let Type::Slice(slice) = ty {
        let items = schema_value_tokens(&slice.elem, None);
        return quote! {
            {
                const ITEMS: Schema = #items;
                Schema::array(#desc, &ITEMS)
            }
        };
    }

    let rendered = quote!(#ty).to_string();
    let rendered = LitStr::new(&rendered, Span::call_site());
    match description {
        Some(text) => quote!(Schema::opaque(Some(#text))),
        None => quote!(Schema::opaque(Some(#rendered))),
    }
}

fn method_tokens(method: FeagiRequestMessageU8) -> TokenStream {
    match FeagiRequestMessageMethod::from_u8(method) {
        FeagiRequestMessageMethod::Read => quote!(HttpMethod::Get),
        FeagiRequestMessageMethod::Create => quote!(HttpMethod::Put),
        FeagiRequestMessageMethod::Edit => quote!(HttpMethod::Post),
        FeagiRequestMessageMethod::Delete => quote!(HttpMethod::Delete),
        FeagiRequestMessageMethod::Patch => quote!(HttpMethod::Patch),
    }
}

fn sorted_fields(template: &StructTemplate, descriptors: &PropertyDescriptors) -> Vec<(Ident, Type, Option<LitStr>)> {
    let mut fields = Vec::new();
    for (name, (ty, doc)) in template.get_map() {
        let description = descriptors.try_get_description(name).cloned().or_else(|| doc.clone());
        fields.push((name.clone(), ty.clone(), description));
    }
    fields.sort_by(|left, right| left.0.to_string().cmp(&right.0.to_string()));
    fields
}

fn lookup_doc(descriptors: &PropertyDescriptors, name: &LitStr) -> Option<LitStr> {
    let ident = Ident::new(&name.value(), name.span());
    descriptors.try_get_description(&ident).cloned()
}

fn split_literals(path: &LitStr) -> Vec<LitStr> {
    path.value()
        .split('/')
        .filter(|segment| !segment.is_empty())
        .map(|segment| LitStr::new(segment, path.span()))
        .collect()
}

fn description_tokens(description: Option<LitStr>) -> TokenStream {
    match description {
        Some(text) => quote!(Some(#text)),
        None => quote!(None),
    }
}

fn peel_option(ty: &Type) -> (&Type, bool) {
    if let Some(inner) = generic_inner(ty, "Option") {
        (inner, false)
    } else {
        (ty, true)
    }
}

fn is_integer(ty: &Type) -> bool {
    const NAMES: &[&str] = &["i8", "i16", "i32", "i64", "i128", "isize", "u8", "u16", "u32", "u64", "u128", "usize"];
    NAMES.iter().any(|name| is_plain_name(ty, name))
}

fn is_plain_name(ty: &Type, name: &str) -> bool {
    let Type::Path(path) = ty else {
        return false;
    };
    if path.qself.is_some() || path.path.segments.len() != 1 {
        return false;
    }
    let segment = &path.path.segments[0];
    matches!(segment.arguments, PathArguments::None) && segment.ident == name
}

fn generic_inner<'a>(ty: &'a Type, name: &str) -> Option<&'a Type> {
    let Type::Path(path) = ty else {
        return None;
    };
    if path.qself.is_some() {
        return None;
    }
    let segment = path.path.segments.last()?;
    if segment.ident != name {
        return None;
    }
    let PathArguments::AngleBracketed(arguments) = &segment.arguments else {
        return None;
    };
    let mut inner = None;
    for argument in &arguments.args {
        if let GenericArgument::Type(ty) = argument {
            if inner.is_some() {
                return None;
            }
            inner = Some(ty);
        }
    }
    inner
}
