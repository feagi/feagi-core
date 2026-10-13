//! Const OpenAPI 3.0.3


const EMPTY_FIELDS: &[ObjectField] = &[];
const EMPTY_REQUIRED: &[&str] = &[];
const FEAGI_INFO_TITLE: &str = "FEAGI";
const FEAGI_INFO_VERSION: &str = "0.0.0"; // TODO package var?
const FEAGI_INFO_DESCRIPTION: &str = "Feagi Server Swagger API Docs";
const OPENAPI_VERSION: &str = "3.0.3";
const SUCCESS_DESCRIPTION: &str = "Successful response";

//region Document input

/// One request category, produced by a category template macro.
#[derive(Clone, Copy, Debug)]
pub struct CategoryApi {
    name: &'static str,
    description: &'static str,
    operations: &'static [OperationApi],
}

impl CategoryApi {
    /// `operations` is the const list of endpoints in this category.
    pub const fn new(name: &'static str, description: &'static str, operations: &'static [OperationApi]) -> Self {
        Self {
            name,
            description,
            operations,
        }
    }
}

/// One endpoint inside a [`CategoryApi`].
#[derive(Clone, Copy, Debug)]
pub struct OperationApi {
    method: HttpMethod,
    path: &'static [PathSegment],
    description: &'static str,
    query: &'static [QueryParameter],
    body: Option<&'static Schema>,
    response: Option<&'static Schema>,
    tag: &'static str, // aka category
}

impl OperationApi {
    /// `path` includes the category base path. `body` and `response` are omitted with `None`.
    pub const fn new(
        method: HttpMethod,
        path: &'static [PathSegment],
        description: &'static str,
        query: &'static [QueryParameter],
        body: Option<&'static Schema>,
        response: Option<&'static Schema>,
        tag: &'static str,
    ) -> Self {
        Self {
            method,
            path,
            description,
            query,
            body,
            response,
            tag,
        }
    }
}

/// HTTP method written into the path item.
///
/// Read is GET, Create is PUT, Edit is POST, Delete is DELETE, and Patch is PATCH.
#[derive(Clone, Copy, Debug)]
#[repr(u8)]
pub enum HttpMethod {
    Get = 0,
    Post = 1,
    Put = 2,
    Delete = 3,
    Patch = 4,
}

/// One piece of a URL path.
#[derive(Clone, Copy, Debug)]
pub enum PathSegment {
    /// A fixed segment, such as `neurons`. Do not include `/`.
    Literal(&'static str),
    /// A `{name}` path parameter.
    Parameter(PathParameter),
}

/// A path parameter.
#[derive(Clone, Copy, Debug)]
pub struct PathParameter {
    name: &'static str,
    description: Option<&'static str>,
    schema: &'static Schema,
}

impl PathParameter {
    /// `name` is written into the path as `{name}`. Path parameters are required.
    pub const fn new(name: &'static str, description: Option<&'static str>, schema: &'static Schema) -> Self {
        Self { name, description, schema }
    }
}

/// A URL query parameter.
#[derive(Clone, Copy, Debug)]
pub struct QueryParameter {
    name: &'static str,
    description: Option<&'static str>,
    required: bool,
    schema: &'static Schema,
}

impl QueryParameter {
    /// Creates a query parameter. `required` is false for `Option` values from the template.
    pub const fn new(name: &'static str, description: Option<&'static str>, required: bool, schema: &'static Schema) -> Self {
        Self {
            name,
            description,
            required,
            schema,
        }
    }
}

//endregion

//region Schema

/// JSON schema stored in the const document.
#[derive(Clone, Copy, Debug)]
pub struct Schema {
    json_type: Option<&'static str>,
    description: Option<&'static str>,
    items: Option<&'static Schema>,
    properties: &'static [ObjectField],
    required: &'static [&'static str],
}

impl Schema {
    /// Boolean schema.
    pub const fn boolean(description: Option<&'static str>) -> Self {
        Self::primitive("boolean", description)
    }

    /// Integer schema.
    pub const fn integer(description: Option<&'static str>) -> Self {
        Self::primitive("integer", description)
    }

    /// Number schema.
    pub const fn number(description: Option<&'static str>) -> Self {
        Self::primitive("number", description)
    }

    /// String schema.
    pub const fn string(description: Option<&'static str>) -> Self {
        Self::primitive("string", description)
    }

    /// Array schema whose elements use `items`.
    pub const fn array(description: Option<&'static str>, items: &'static Schema) -> Self {
        Self {
            items: Some(items),
            ..Self::primitive("array", description)
        }
    }

    /// Object schema. Field documentation lives on each field schema.
    pub const fn object(description: Option<&'static str>, properties: &'static [ObjectField], required: &'static [&'static str]) -> Self {
        Self {
            properties,
            required,
            ..Self::primitive("object", description)
        }
    }

    /// Object schema for a Rust type this generator does not map to a JSON primitive.
    pub const fn opaque(description: Option<&'static str>) -> Self {
        Self::object(description, EMPTY_FIELDS, EMPTY_REQUIRED)
    }

    const fn primitive(json_type: &'static str, description: Option<&'static str>) -> Self {
        Self {
            json_type: Some(json_type),
            description,
            items: None,
            properties: EMPTY_FIELDS,
            required: EMPTY_REQUIRED,
        }
    }
}

/// One property of an object schema.
#[derive(Clone, Copy, Debug)]
pub struct ObjectField {
    name: &'static str,
    schema: &'static Schema,
}

impl ObjectField {
    /// `schema` is the const schema for this property.
    pub const fn new(name: &'static str, schema: &'static Schema) -> Self {
        Self { name, schema }
    }
}

//endregion

//region OpenApiDocument

/// OpenAPI document assembled from const category data.
#[derive(Clone, Copy, Debug)]
pub struct OpenApiDocument {
    categories: &'static [CategoryApi],
}

impl OpenApiDocument {
    /// Stores the categories that the JSON writer walks.
    pub const fn from_categories(categories: &'static [CategoryApi]) -> Self {
        Self { categories }
    }

    /// Number of UTF-8 bytes `write_json` will produce.
    pub const fn json_len(&self) -> usize {
        let mut dummy = [0u8; 0];
        let mut writer = JsonWriter {
            buf: &mut dummy,
            pos: 0,
            counting: true,
        };
        self.emit(&mut writer);
        writer.pos
    }

    /// Writes the document into a buffer of length [`Self::json_len`].
    pub const fn write_json<const N: usize>(&self) -> [u8; N] {
        let mut buf = [0u8; N];
        let mut writer = JsonWriter {
            buf: &mut buf,
            pos: 0,
            counting: false,
        };
        self.emit(&mut writer);
        if writer.pos != N {
            panic!("OpenAPI JSON byte length did not match json_len");
        }
        buf
    }

    /// Interprets a buffer previously filled by [`Self::write_json`] as JSON text.
    pub const fn json_str(bytes: &'static [u8]) -> &'static str {
        match core::str::from_utf8(bytes) {
            Ok(text) => text,
            Err(_) => panic!("OpenAPI JSON is not valid UTF-8"),
        }
    }


    const fn emit(&self, writer: &mut JsonWriter) {
        writer.push_byte(b'{');
        writer.push_str("\"openapi\":");
        writer.push_json_string(OPENAPI_VERSION);
        writer.push_str(",\"info\":{\"title\":");
        writer.push_json_string(FEAGI_INFO_TITLE);
        writer.push_str(",\"version\":");
        writer.push_json_string(FEAGI_INFO_VERSION);
        writer.push_str(",\"description\":");
        writer.push_json_string(FEAGI_INFO_DESCRIPTION);
        writer.push_byte(b'}');
        self.emit_tags(writer);
        self.emit_paths(writer);
        writer.push_byte(b'}');
    }

    const fn emit_tags(&self, writer: &mut JsonWriter) {
        if self.categories.is_empty() {
            return;
        }
        writer.push_str(",\"tags\":[");
        let mut index = 0;
        while index < self.categories.len() {
            if index > 0 {
                writer.push_byte(b',');
            }
            let category = &self.categories[index];
            writer.push_str("{\"name\":");
            writer.push_json_string(category.name);
            writer.push_str(",\"description\":");
            writer.push_json_string(category.description);
            writer.push_byte(b'}');
            index += 1;
        }
        writer.push_byte(b']');
    }

    const fn emit_paths(&self, writer: &mut JsonWriter) {
        writer.push_str(",\"paths\":{");
        let mut wrote_path = false;
        let mut cat_index = 0;
        while cat_index < self.categories.len() {
            let operations = self.categories[cat_index].operations;
            let mut op_index = 0;
            while op_index < operations.len() {
                if self.is_first_path(cat_index, op_index) {
                    if wrote_path {
                        writer.push_byte(b',');
                    }
                    wrote_path = true;
                    self.emit_one_path(writer, operations[op_index].path);
                }
                op_index += 1;
            }
            cat_index += 1;
        }
        writer.push_byte(b'}');
    }

    const fn is_first_path(&self, cat_index: usize, op_index: usize) -> bool {
        let path = self.categories[cat_index].operations[op_index].path;
        let mut earlier_cat = 0;
        while earlier_cat <= cat_index {
            let operations = self.categories[earlier_cat].operations;
            let limit = if earlier_cat == cat_index { op_index } else { operations.len() };
            let mut earlier_op = 0;
            while earlier_op < limit {
                if paths_equal(operations[earlier_op].path, path) {
                    return false;
                }
                earlier_op += 1;
            }
            earlier_cat += 1;
        }
        true
    }

    const fn emit_one_path(&self, writer: &mut JsonWriter, path: &'static [PathSegment]) {
        push_path_string(writer, path);
        writer.push_byte(b':');
        writer.push_byte(b'{');
        let mut wrote = false;
        wrote = self.emit_method_if_present(writer, path, HttpMethod::Get, "get", wrote);
        wrote = self.emit_method_if_present(writer, path, HttpMethod::Post, "post", wrote);
        wrote = self.emit_method_if_present(writer, path, HttpMethod::Put, "put", wrote);
        wrote = self.emit_method_if_present(writer, path, HttpMethod::Delete, "delete", wrote);
        let _ = self.emit_method_if_present(writer, path, HttpMethod::Patch, "patch", wrote);
        writer.push_byte(b'}');
    }

    const fn emit_method_if_present(
        &self,
        writer: &mut JsonWriter,
        path: &'static [PathSegment],
        method: HttpMethod,
        key: &str,
        wrote: bool,
    ) -> bool {
        let mut cat_index = 0;
        while cat_index < self.categories.len() {
            let operations = self.categories[cat_index].operations;
            let mut op_index = 0;
            while op_index < operations.len() {
                let operation = &operations[op_index];
                if method_eq(operation.method, method) && paths_equal(operation.path, path) {
                    if wrote {
                        writer.push_byte(b',');
                    }
                    writer.push_json_string(key);
                    writer.push_byte(b':');
                    emit_operation(writer, operation);
                    return true;
                }
                op_index += 1;
            }
            cat_index += 1;
        }
        wrote
    }
}

const fn emit_operation(writer: &mut JsonWriter, operation: &OperationApi) {
    writer.push_str("{\"tags\":[");
    writer.push_json_string(operation.tag);
    writer.push_str("],\"description\":");
    writer.push_json_string(operation.description);
    if operation_has_parameters(operation) {
        writer.push_str(",\"parameters\":[");
        let mut wrote = false;
        let mut index = 0;
        while index < operation.path.len() {
            if let PathSegment::Parameter(parameter) = operation.path[index] {
                if wrote {
                    writer.push_byte(b',');
                }
                wrote = true;
                emit_parameter(writer, parameter.name, "path", parameter.description, true, parameter.schema);
            }
            index += 1;
        }
        index = 0;
        while index < operation.query.len() {
            if wrote {
                writer.push_byte(b',');
            }
            wrote = true;
            let parameter = &operation.query[index];
            emit_parameter(
                writer,
                parameter.name,
                "query",
                parameter.description,
                parameter.required,
                parameter.schema,
            );
            index += 1;
        }
        writer.push_byte(b']');
    }
    if let Some(body) = operation.body {
        writer.push_str(",\"requestBody\":{\"required\":true,\"content\":{\"application/json\":{\"schema\":");
        emit_schema(writer, body);
        writer.push_str("}}}");
    }
    writer.push_str(",\"responses\":{\"200\":{\"description\":");
    writer.push_json_string(SUCCESS_DESCRIPTION);
    if let Some(response) = operation.response {
        writer.push_str(",\"content\":{\"application/json\":{\"schema\":");
        emit_schema(writer, response);
        writer.push_str("}}");
    }
    writer.push_str("}}}");
}

const fn operation_has_parameters(operation: &OperationApi) -> bool {
    if !operation.query.is_empty() {
        return true;
    }
    let mut index = 0;
    while index < operation.path.len() {
        if let PathSegment::Parameter(_) = operation.path[index] {
            return true;
        }
        index += 1;
    }
    false
}

const fn emit_parameter(writer: &mut JsonWriter, name: &str, location: &str, description: Option<&str>, required: bool, schema: &Schema) {
    writer.push_str("{\"name\":");
    writer.push_json_string(name);
    writer.push_str(",\"in\":");
    writer.push_json_string(location);
    if let Some(description) = description {
        writer.push_str(",\"description\":");
        writer.push_json_string(description);
    }
    writer.push_str(",\"required\":");
    if required {
        writer.push_str("true");
    } else {
        writer.push_str("false");
    }
    writer.push_str(",\"schema\":");
    emit_schema(writer, schema);
    writer.push_byte(b'}');
}

const fn emit_schema(writer: &mut JsonWriter, schema: &Schema) {
    writer.push_byte(b'{');
    let mut wrote = false;
    if let Some(json_type) = schema.json_type {
        writer.push_str("\"type\":");
        writer.push_json_string(json_type);
        wrote = true;
    }
    if let Some(description) = schema.description {
        if wrote {
            writer.push_byte(b',');
        }
        writer.push_str("\"description\":");
        writer.push_json_string(description);
        wrote = true;
    }
    if let Some(items) = schema.items {
        if wrote {
            writer.push_byte(b',');
        }
        writer.push_str("\"items\":");
        emit_schema(writer, items);
        wrote = true;
    }
    if !schema.properties.is_empty() {
        if wrote {
            writer.push_byte(b',');
        }
        writer.push_str("\"properties\":{");
        let mut index = 0;
        while index < schema.properties.len() {
            if index > 0 {
                writer.push_byte(b',');
            }
            let field = &schema.properties[index];
            writer.push_json_string(field.name);
            writer.push_byte(b':');
            emit_schema(writer, field.schema);
            index += 1;
        }
        writer.push_byte(b'}');
        wrote = true;
    }
    if !schema.required.is_empty() {
        if wrote {
            writer.push_byte(b',');
        }
        writer.push_str("\"required\":[");
        let mut index = 0;
        while index < schema.required.len() {
            if index > 0 {
                writer.push_byte(b',');
            }
            writer.push_json_string(schema.required[index]);
            index += 1;
        }
        writer.push_byte(b']');
    }
    writer.push_byte(b'}');
}

const fn method_eq(left: HttpMethod, right: HttpMethod) -> bool {
    (left as u8) == (right as u8)
}

const fn paths_equal(left: &[PathSegment], right: &[PathSegment]) -> bool {
    if left.len() != right.len() {
        return false;
    }
    let mut index = 0;
    while index < left.len() {
        if !segment_eq(&left[index], &right[index]) {
            return false;
        }
        index += 1;
    }
    true
}

const fn segment_eq(left: &PathSegment, right: &PathSegment) -> bool {
    match (left, right) {
        (PathSegment::Literal(left), PathSegment::Literal(right)) => str_eq(left, right),
        (PathSegment::Parameter(left), PathSegment::Parameter(right)) => str_eq(left.name, right.name),
        _ => false,
    }
}

const fn str_eq(left: &str, right: &str) -> bool {
    let left = left.as_bytes();
    let right = right.as_bytes();
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

const fn push_path_string(writer: &mut JsonWriter, segments: &[PathSegment]) {
    writer.push_byte(b'"');
    if segments.is_empty() {
        writer.push_byte(b'/');
    } else {
        let mut index = 0;
        while index < segments.len() {
            writer.push_byte(b'/');
            match segments[index] {
                PathSegment::Literal(text) => writer.push_escaped(text),
                PathSegment::Parameter(parameter) => {
                    writer.push_byte(b'{');
                    writer.push_escaped(parameter.name);
                    writer.push_byte(b'}');
                }
            }
            index += 1;
        }
    }
    writer.push_byte(b'"');
}

//endregion

//region JSON writer

struct JsonWriter<'a> {
    buf: &'a mut [u8],
    pos: usize,
    counting: bool,
}

impl JsonWriter<'_> {
    const fn push_byte(&mut self, byte: u8) {
        if !self.counting {
            self.buf[self.pos] = byte;
        }
        self.pos += 1;
    }

    const fn push_str(&mut self, text: &str) {
        let bytes = text.as_bytes();
        let mut index = 0;
        while index < bytes.len() {
            self.push_byte(bytes[index]);
            index += 1;
        }
    }

    const fn push_json_string(&mut self, text: &str) {
        self.push_byte(b'"');
        self.push_escaped(text);
        self.push_byte(b'"');
    }

    const fn push_escaped(&mut self, text: &str) {
        let bytes = text.as_bytes();
        let mut index = 0;
        while index < bytes.len() {
            let byte = bytes[index];
            match byte {
                b'"' => self.push_str("\\\""),
                b'\\' => self.push_str("\\\\"),
                b'\n' => self.push_str("\\n"),
                b'\r' => self.push_str("\\r"),
                b'\t' => self.push_str("\\t"),
                byte if byte < 0x20 => self.push_control(byte),
                byte => self.push_byte(byte),
            }
            index += 1;
        }
    }

    const fn push_control(&mut self, byte: u8) {
        const HEX: &[u8; 16] = b"0123456789abcdef";
        self.push_str("\\u00");
        self.push_byte(HEX[(byte >> 4) as usize]);
        self.push_byte(HEX[(byte & 0x0f) as usize]);
    }
}

//endregion