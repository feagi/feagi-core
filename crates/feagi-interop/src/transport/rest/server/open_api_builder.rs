//! OpenAPI 3.0.3 document of the endpoints a REST server serves, written at compile time from
//! the const descriptions of a group's categories and messages
//!
//! Configurables are written as path parameters (`{name}`), as their values are only known at
//! runtime. The document is built with `feagi_open_api_json!` where the group type is concrete,
//! as stable Rust can only size the document's array there, and handed to the server builder as
//! a `&'static str`

use feagi_basis::feagi_message::feagi_message_description::{FeagiFieldDescription, FeagiFieldKind, FeagiMessagePathElementDescription};
use feagi_basis::feagi_message::feagi_message_path::FeagiMessagePath;
use feagi_basis::feagi_message::request::feagi_request_descriptor::{FeagiMessageEnum, FeagiRequestMessageDescriptor};
use feagi_basis::feagi_message::request::feagi_request_message_category::FeagiRequestCategory;
use feagi_basis::feagi_message::request::feagi_request_message_grouping::{FeagiConfiguredCategoryRequester, NCat, NMDes, NNull};
use feagi_basis::parallelism::feagi_channel::request_response_channel::channel_backend::FeagiRequestResponseChannelBackend;

const FEAGI_INFO_TITLE: &str = "FEAGI";
const FEAGI_INFO_VERSION: &str = "0.0.0"; // TODO package var?
const FEAGI_INFO_DESCRIPTION: &str = "Feagi Server Swagger API Docs";
const OPENAPI_VERSION: &str = "3.0.3";

/// Description of configurable path parameters, which the server fixes when it starts
const CONFIGURABLE_DESCRIPTION: &str = "Configured by the server when it starts";

/// Where the server serves the document
pub const OPEN_API_JSON_PATH: &str = "/api-docs/openapi.json";

/// Methods in the order they are written under each path
const METHODS: [(FeagiMessageEnum, &str); 5] = [
    (FeagiMessageEnum::Read, "get"),
    (FeagiMessageEnum::Create, "post"),
    (FeagiMessageEnum::Edit, "put"),
    (FeagiMessageEnum::Delete, "delete"),
    (FeagiMessageEnum::Patch, "patch"),
];

/// The OpenAPI document of every message of every category of `$group`, as a `&'static str`
/// built at compile time. `$group` is the group's requesters type and `$channels` its channel
/// backend, both concrete
///
/// ```ignore
/// const OPEN_API_JSON: &str = feagi_open_api_json!(GenomeManagerRequesters<ChannelBackend>, ChannelBackend);
/// ```
#[macro_export]
macro_rules! feagi_open_api_json {
    ($group:ty, $channels:ty) => {{
        type Routes = $crate::transport::rest::server::server_basis::GroupRestRoutes<$group, $channels>;
        const LEN: usize = $crate::transport::rest::server::open_api_builder::open_api_json_len::<Routes>();
        const BYTES: [u8; LEN] = $crate::transport::rest::server::open_api_builder::open_api_json_bytes::<Routes, LEN>();
        const JSON: &str = $crate::transport::rest::server::open_api_builder::json_str(&BYTES);
        JSON
    }};
}

//region Const Descriptions

/// One message route, for the document
#[derive(Clone, Copy)]
pub struct OpenApiRoute {
    pub method: FeagiMessageEnum,
    pub description: &'static str,
    pub path_elements: &'static [FeagiMessagePathElementDescription],
    /// From the URL query for Read messages, otherwise from the json body
    pub request_payload_fields: &'static [FeagiFieldDescription],
    pub response_payload_fields: &'static [FeagiFieldDescription],
}

/// The routes of a category, as a const linked list
pub struct OpenApiRouteNode {
    pub route: OpenApiRoute,
    pub next: Option<&'static OpenApiRouteNode>,
}

/// The categories of a group, as a const linked list
pub struct OpenApiCategoryNode {
    pub name: &'static str,
    pub description: &'static str,
    pub routes: Option<&'static OpenApiRouteNode>,
    pub next: Option<&'static OpenApiCategoryNode>,
}

/// The routes of a category's messages. Implemented for the `NMDes` chain of the category
pub trait OpenApiMessageRoutes {
    const ROUTES: Option<&'static OpenApiRouteNode>;
}

impl OpenApiMessageRoutes for NNull {
    const ROUTES: Option<&'static OpenApiRouteNode> = None;
}

impl<Descriptor, Tail> OpenApiMessageRoutes for NMDes<Descriptor, Tail>
where
    Descriptor: FeagiRequestMessageDescriptor,
    Tail: OpenApiMessageRoutes,
{
    const ROUTES: Option<&'static OpenApiRouteNode> = Some(&OpenApiRouteNode {
        route: OpenApiRoute {
            method: Descriptor::METHOD,
            description: Descriptor::MESSAGE_DESCRIPTION,
            path_elements: <Descriptor::MessagePath as FeagiMessagePath>::PATH_ELEMENTS,
            request_payload_fields: Descriptor::REQUEST_PAYLOAD_FIELDS,
            response_payload_fields: Descriptor::RESPONSE_PAYLOAD_FIELDS,
        },
        next: Tail::ROUTES,
    });
}

/// The categories of a group. Implemented for the `NCat` chain a group's routes are served from
pub trait OpenApiCategories {
    const CATEGORIES: Option<&'static OpenApiCategoryNode>;
}

impl OpenApiCategories for NNull {
    const CATEGORIES: Option<&'static OpenApiCategoryNode> = None;
}

impl<Category, Channels, Tail> OpenApiCategories for NCat<FeagiConfiguredCategoryRequester<Category, Channels>, Tail>
where
    Category: FeagiRequestCategory,
    Category::NestedMessageDescriptors: OpenApiMessageRoutes,
    Channels: FeagiRequestResponseChannelBackend,
    Tail: OpenApiCategories,
{
    const CATEGORIES: Option<&'static OpenApiCategoryNode> = Some(&OpenApiCategoryNode {
        name: Category::CATEGORY_NAME,
        description: Category::CATEGORY_DESCRIPTION,
        routes: <Category::NestedMessageDescriptors as OpenApiMessageRoutes>::ROUTES,
        next: Tail::CATEGORIES,
    });
}

//endregion

//region Document

/// Length of the document of `Routes`, the size `open_api_json_bytes` needs
pub const fn open_api_json_len<Routes: OpenApiCategories>() -> usize {
    // Nothing fits, so this only counts
    let mut out = ConstJsonWriter::new(&mut []);
    write_open_api_json(Routes::CATEGORIES, &mut out);
    out.len
}

/// The document of `Routes`. `LEN` must be `open_api_json_len::<Routes>()`
pub const fn open_api_json_bytes<Routes: OpenApiCategories, const LEN: usize>() -> [u8; LEN] {
    let mut buffer = [0u8; LEN];
    let mut out = ConstJsonWriter::new(&mut buffer);
    write_open_api_json(Routes::CATEGORIES, &mut out);
    assert!(out.len == LEN, "LEN is not the length of the OpenAPI document");
    buffer
}

/// The document's bytes as a str. Always valid, as only str contents are written
pub const fn json_str(bytes: &[u8]) -> &str {
    match core::str::from_utf8(bytes) {
        Ok(json) => json,
        Err(_) => panic!("the OpenAPI document is not utf-8"),
    }
}

/// Writes the document of every route of `categories`. Each category is a tag, and messages
/// sharing a path are written as the methods of that one path
const fn write_open_api_json(categories: Option<&'static OpenApiCategoryNode>, out: &mut ConstJsonWriter) {
    out.raw("{\"openapi\":");
    out.string(OPENAPI_VERSION);
    out.raw(",\"info\":{\"title\":");
    out.string(FEAGI_INFO_TITLE);
    out.raw(",\"version\":");
    out.string(FEAGI_INFO_VERSION);
    out.raw(",\"description\":");
    out.string(FEAGI_INFO_DESCRIPTION);
    out.raw("}");
    write_tags(categories, out);
    write_paths(categories, out);
    out.raw("}");
}

/// One tag per category name, in the order they first appear
const fn write_tags(categories: Option<&'static OpenApiCategoryNode>, out: &mut ConstJsonWriter) {
    out.raw(",\"tags\":[");
    let mut wrote_tag = false;
    let mut category = categories;
    let mut index = 0;
    while let Some(current) = category {
        let mut first_of_name = true;
        let mut earlier = categories;
        let mut earlier_index = 0;
        while earlier_index < index {
            let Some(earlier_category) = earlier else {
                break;
            };
            if str_eq(earlier_category.name, current.name) {
                first_of_name = false;
                break;
            }
            earlier = earlier_category.next;
            earlier_index += 1;
        }
        if first_of_name {
            if wrote_tag {
                out.raw(",");
            }
            wrote_tag = true;
            out.raw("{\"name\":");
            out.string(current.name);
            out.raw(",\"description\":");
            out.string(current.description);
            out.raw("}");
        }
        category = current.next;
        index += 1;
    }
    out.raw("]");
}

/// Each distinct path once, with every method of the routes sharing it
const fn write_paths(categories: Option<&'static OpenApiCategoryNode>, out: &mut ConstJsonWriter) {
    out.raw(",\"paths\":{");
    let mut wrote_path = false;
    let mut cursor = RouteCursor::start(categories);
    let mut index = 0;
    while let Some((_, route)) = cursor.current() {
        // Only the first route of a path writes it
        let mut first_of_path = true;
        let mut earlier = RouteCursor::start(categories);
        let mut earlier_index = 0;
        while earlier_index < index {
            if let Some((_, earlier_route)) = earlier.current() {
                if same_path(earlier_route, route) {
                    first_of_path = false;
                    break;
                }
            }
            earlier = earlier.next();
            earlier_index += 1;
        }

        if first_of_path {
            if wrote_path {
                out.raw(",");
            }
            wrote_path = true;
            write_path_key(out, route);
            out.raw(":{");
            let mut wrote_method = false;
            let mut method_index = 0;
            while method_index < METHODS.len() {
                let (method, method_key) = METHODS[method_index];
                // The first route of this method at this path. Routes before this one are at other
                // paths, as this is the first route at this path
                let mut candidate = cursor;
                while let Some((category, candidate_route)) = candidate.current() {
                    if candidate_route.method as u8 == method as u8 && same_path(candidate_route, route) {
                        if wrote_method {
                            out.raw(",");
                        }
                        wrote_method = true;
                        out.string(method_key);
                        out.raw(":");
                        write_operation(out, category, candidate_route);
                        break;
                    }
                    candidate = candidate.next();
                }
                method_index += 1;
            }
            out.raw("}");
        }
        cursor = cursor.next();
        index += 1;
    }
    out.raw("}");
}

const fn write_operation(out: &mut ConstJsonWriter, category: &OpenApiCategoryNode, route: &OpenApiRoute) {
    let payload_in_query = route.method.is_payload_in_url();

    out.raw("{\"tags\":[");
    out.string(category.name);
    out.raw("],\"description\":");
    out.string(route.description);

    out.raw(",\"parameters\":[");
    let mut wrote_parameter = false;
    let mut element_index = 0;
    while element_index < route.path_elements.len() {
        match route.path_elements[element_index] {
            FeagiMessagePathElementDescription::Static(_) => {}
            FeagiMessagePathElementDescription::Configurable { name } => {
                if wrote_parameter {
                    out.raw(",");
                }
                wrote_parameter = true;
                write_parameter(out, name, "path", Some(CONFIGURABLE_DESCRIPTION), true, FeagiFieldKind::String);
            }
            FeagiMessagePathElementDescription::Parameter { name, description, kind } => {
                if wrote_parameter {
                    out.raw(",");
                }
                wrote_parameter = true;
                write_parameter(out, name, "path", description, true, kind);
            }
        }
        element_index += 1;
    }
    if payload_in_query {
        let mut field_index = 0;
        while field_index < route.request_payload_fields.len() {
            let field = &route.request_payload_fields[field_index];
            if wrote_parameter {
                out.raw(",");
            }
            wrote_parameter = true;
            write_parameter(out, field.name, "query", field.description, field.required, field.kind);
            field_index += 1;
        }
    }
    out.raw("]");

    if !payload_in_query && !route.request_payload_fields.is_empty() {
        out.raw(",\"requestBody\":{\"required\":true,\"content\":{\"application/json\":{\"schema\":");
        write_object_schema(out, route.request_payload_fields);
        out.raw("}}}");
    }

    out.raw(",\"responses\":{\"200\":{\"description\":\"Successful response\"");
    if !route.response_payload_fields.is_empty() {
        out.raw(",\"content\":{\"application/json\":{\"schema\":");
        write_object_schema(out, route.response_payload_fields);
        out.raw("}}");
    }
    out.raw("}");
    out.raw(",\"400\":{\"description\":\"The request payload could not be parsed, or the message responded with its error\"}");
    out.raw(",\"503\":{\"description\":\"The request could not be sent to, or answered by, the worker serving its category\"}");
    out.raw("}}");
}

const fn write_parameter(out: &mut ConstJsonWriter, name: &str, location: &str, description: Option<&str>, required: bool, kind: FeagiFieldKind) {
    out.raw("{\"name\":");
    out.string(name);
    out.raw(",\"in\":");
    out.string(location);
    if let Some(description) = description {
        out.raw(",\"description\":");
        out.string(description);
    }
    out.raw(if required { ",\"required\":true" } else { ",\"required\":false" });
    out.raw(",\"schema\":");
    write_kind_schema(out, kind, None);
    out.raw("}");
}

const fn write_object_schema(out: &mut ConstJsonWriter, fields: &[FeagiFieldDescription]) {
    out.raw("{\"type\":\"object\",\"properties\":{");
    let mut any_required = false;
    let mut index = 0;
    while index < fields.len() {
        if index != 0 {
            out.raw(",");
        }
        out.string(fields[index].name);
        out.raw(":");
        write_kind_schema(out, fields[index].kind, fields[index].description);
        any_required |= fields[index].required;
        index += 1;
    }
    out.raw("}");
    if any_required {
        out.raw(",\"required\":[");
        let mut wrote_required = false;
        let mut index = 0;
        while index < fields.len() {
            if fields[index].required {
                if wrote_required {
                    out.raw(",");
                }
                wrote_required = true;
                out.string(fields[index].name);
            }
            index += 1;
        }
        out.raw("]");
    }
    out.raw("}");
}

const fn write_kind_schema(out: &mut ConstJsonWriter, kind: FeagiFieldKind, description: Option<&str>) {
    out.raw(match kind {
        FeagiFieldKind::Boolean => "{\"type\":\"boolean\"",
        FeagiFieldKind::Integer => "{\"type\":\"integer\"",
        FeagiFieldKind::Number => "{\"type\":\"number\"",
        FeagiFieldKind::String => "{\"type\":\"string\"",
        FeagiFieldKind::Array => "{\"type\":\"array\",\"items\":{}",
        FeagiFieldKind::Object => "{\"type\":\"object\"",
    });
    if let Some(description) = description {
        out.raw(",\"description\":");
        out.string(description);
    }
    out.raw("}");
}

//endregion

//region Routes

/// Walks every route of every category in order
#[derive(Clone, Copy)]
struct RouteCursor {
    category: Option<&'static OpenApiCategoryNode>,
    route: Option<&'static OpenApiRouteNode>,
}

impl RouteCursor {
    /// At the first route of `categories`
    const fn start(categories: Option<&'static OpenApiCategoryNode>) -> Self {
        let routes = match categories {
            Some(category) => category.routes,
            None => None,
        };
        Self::skip_empty(categories, routes)
    }

    /// At the route after this one
    const fn next(self) -> Self {
        match self.route {
            Some(route) => Self::skip_empty(self.category, route.next),
            None => self,
        }
    }

    /// The route the cursor is at, with its category, or `None` past the last one
    const fn current(&self) -> Option<(&'static OpenApiCategoryNode, &'static OpenApiRoute)> {
        match (self.category, self.route) {
            (Some(category), Some(route)) => Some((category, &route.route)),
            _ => None,
        }
    }

    /// Moves past categories with no routes left
    const fn skip_empty(mut category: Option<&'static OpenApiCategoryNode>, mut route: Option<&'static OpenApiRouteNode>) -> Self {
        while route.is_none() {
            let Some(current) = category else {
                break;
            };
            category = current.next;
            route = match category {
                Some(next) => next.routes,
                None => None,
            };
        }
        RouteCursor { category, route }
    }
}

/// Whether two routes are at the same path. Configurables and parameters are both written as
/// `{name}`, so only their names are compared
const fn same_path(left: &OpenApiRoute, right: &OpenApiRoute) -> bool {
    if left.path_elements.len() != right.path_elements.len() {
        return false;
    }
    let mut index = 0;
    while index < left.path_elements.len() {
        let same_element = match (segment(&left.path_elements[index]), segment(&right.path_elements[index])) {
            (PathSegment::Text(left), PathSegment::Text(right)) | (PathSegment::Parameter(left), PathSegment::Parameter(right)) => str_eq(left, right),
            _ => false,
        };
        if !same_element {
            return false;
        }
        index += 1;
    }
    true
}

/// One segment of a route's path as written in the document
enum PathSegment {
    Text(&'static str),
    Parameter(&'static str),
}

const fn segment(element: &FeagiMessagePathElementDescription) -> PathSegment {
    match *element {
        FeagiMessagePathElementDescription::Static(text) => PathSegment::Text(text),
        FeagiMessagePathElementDescription::Configurable { name } | FeagiMessagePathElementDescription::Parameter { name, .. } => PathSegment::Parameter(name),
    }
}

/// The path as the json key, with configurables and parameters as `{name}`
const fn write_path_key(out: &mut ConstJsonWriter, route: &OpenApiRoute) {
    out.raw("\"");
    let mut index = 0;
    while index < route.path_elements.len() {
        out.raw("/");
        match segment(&route.path_elements[index]) {
            PathSegment::Text(text) => out.escaped(text),
            PathSegment::Parameter(name) => {
                out.raw("{");
                out.escaped(name);
                out.raw("}");
            }
        }
        index += 1;
    }
    out.raw("\"");
}

//endregion

//region JSON

const fn str_eq(left: &str, right: &str) -> bool {
    let (left, right) = (left.as_bytes(), right.as_bytes());
    if left.len() != right.len() {
        return false;
    }
    let mut index = 0;
    while index < left.len() {
        if left[index] != right[index] {
            return false;
        }
        index += 1;
    }
    true
}

/// Writes json into a buffer in const code. Counts every byte written, including those past
/// the end of the buffer, so writing into an empty buffer gives the length needed
struct ConstJsonWriter<'a> {
    buffer: &'a mut [u8],
    len: usize,
}

impl<'a> ConstJsonWriter<'a> {
    const fn new(buffer: &'a mut [u8]) -> Self {
        ConstJsonWriter { buffer, len: 0 }
    }

    const fn byte(&mut self, byte: u8) {
        if self.len < self.buffer.len() {
            self.buffer[self.len] = byte;
        }
        self.len += 1;
    }

    /// Writes `text` as it is
    const fn raw(&mut self, text: &str) {
        self.bytes(text.as_bytes());
    }

    /// Copies `bytes` in at once, as const evaluation is slow per byte
    const fn bytes(&mut self, bytes: &[u8]) {
        if self.len + bytes.len() <= self.buffer.len() {
            let (_, unwritten) = self.buffer.split_at_mut(self.len);
            let (destination, _) = unwritten.split_at_mut(bytes.len());
            destination.copy_from_slice(bytes);
        }
        self.len += bytes.len();
    }

    /// Writes `text` as a json string
    const fn string(&mut self, text: &str) {
        self.byte(b'"');
        self.escaped(text);
        self.byte(b'"');
    }

    /// Writes `text` escaped for use inside a json string. Bytes of multibyte characters are
    /// all at least 0x80, so they are written as they are
    const fn escaped(&mut self, text: &str) {
        const HEX: &[u8; 16] = b"0123456789abcdef";
        let bytes = text.as_bytes();
        // Most text needs no escaping, and is copied at once
        let mut needs_escaping = false;
        let mut index = 0;
        while index < bytes.len() {
            if bytes[index] == b'"' || bytes[index] == b'\\' || bytes[index] < 0x20 {
                needs_escaping = true;
                break;
            }
            index += 1;
        }
        if !needs_escaping {
            self.bytes(bytes);
            return;
        }

        let mut index = 0;
        while index < bytes.len() {
            match bytes[index] {
                b'"' => self.raw("\\\""),
                b'\\' => self.raw("\\\\"),
                b'\n' => self.raw("\\n"),
                b'\r' => self.raw("\\r"),
                b'\t' => self.raw("\\t"),
                control if control < 0x20 => {
                    self.raw("\\u00");
                    self.byte(HEX[(control >> 4) as usize]);
                    self.byte(HEX[(control & 0xf) as usize]);
                }
                byte => self.byte(byte),
            }
            index += 1;
        }
    }
}

//endregion
