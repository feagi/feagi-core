use serde::Serialize;
use std::collections::BTreeMap;

const OPENAPI_VERSION: &str = "3.0.3";
const DEFAULT_INFO_VERSION: &str = "0.0.0";

//region OpenApiDocument

/// OpenAPI document
#[derive(Clone, Debug, Serialize)]
pub struct OpenApiDocument {
    openapi: &'static str,
    /// Document title, version, description
    pub info: Info,
    /// Document-level tags
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<Tag>,
    paths: BTreeMap<String, PathItem>,
    #[serde(skip_serializing_if = "Option::is_none")]
    components: Option<Components>,
}

impl OpenApiDocument {

    /// Creates an empty document
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            openapi: OPENAPI_VERSION,
            info: Info {
                title: title.into(),
                version: DEFAULT_INFO_VERSION.to_string(),
                description: None,
            },
            tags: Vec::new(),
            paths: BTreeMap::new(),
            components: None,
        }
    }

    /// Registers a tag used to categorize operations
    pub fn add_tag(&mut self, tag: Tag) -> &mut Self {
        if let Some(existing) = self.tags.iter_mut().find(|existing| existing.name == tag.name) {
            *existing = tag;
        } else {
            self.tags.push(tag);
        }
        self
    }

    /// Registers a named schema under `components.schemas`
    pub fn add_schema(&mut self, name: impl Into<String>, schema: Schema) -> &mut Self {
        self.components
            .get_or_insert_with(Components::default)
            .schemas
            .insert(name.into(), schema);
        self
    }

    /// Registers the schema produced by `source`
    pub fn add_schema_for(&mut self, source: &impl OpenApiSchema) -> &mut Self {
        self.add_schema(source.schema_name(), source.open_api_schema())
    }

    /// Adds a GET operation. GET has no body
    pub fn add_get(&mut self, endpoint: GetEndpoint) -> &mut Self {
        self.add_operation(
            HttpMethod::Get,
            endpoint.path,
            endpoint.description,
            endpoint.tags,
            endpoint.query_parameters,
            endpoint.responses,
            None,
        )
    }

    /// Adds a POST operation
    pub fn add_post(&mut self, endpoint: PayloadEndpoint) -> &mut Self {
        self.add_operation(
            HttpMethod::Post,
            endpoint.path,
            endpoint.description,
            endpoint.tags,
            endpoint.query_parameters,
            endpoint.responses,
            endpoint.request_body,
        )
    }

    /// Adds a PUT operation
    pub fn add_put(&mut self, endpoint: PayloadEndpoint) -> &mut Self {
        self.add_operation(
            HttpMethod::Put,
            endpoint.path,
            endpoint.description,
            endpoint.tags,
            endpoint.query_parameters,
            endpoint.responses,
            endpoint.request_body,
        )
    }

    /// Adds a DELETE operation
    pub fn add_delete(&mut self, endpoint: PayloadEndpoint) -> &mut Self {
        self.add_operation(
            HttpMethod::Delete,
            endpoint.path,
            endpoint.description,
            endpoint.tags,
            endpoint.query_parameters,
            endpoint.responses,
            endpoint.request_body,
        )
    }

    /// Adds a PATCH operation
    pub fn add_patch(&mut self, endpoint: PayloadEndpoint) -> &mut Self {
        self.add_operation(
            HttpMethod::Patch,
            endpoint.path,
            endpoint.description,
            endpoint.tags,
            endpoint.query_parameters,
            endpoint.responses,
            endpoint.request_body,
        )
    }

    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    pub fn to_json_pretty(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Inserts one operation, creating the path item on first use
    fn add_operation(
        &mut self,
        method: HttpMethod,
        path: Vec<PathSegment>,
        description: Option<String>,
        tags: Vec<String>,
        query_parameters: Vec<QueryParameter>,
        responses: BTreeMap<String, ResponseObject>,
        request_body: Option<RequestBody>,
    ) -> &mut Self {
        let (path_key, mut parameters) = compile_path(path);
        parameters.extend(query_parameters.into_iter().map(Parameter::query));
        let operation = Operation {
            tags,
            description,
            parameters,
            request_body,
            responses,
        };

        let item = self.paths.entry(path_key).or_default();
        match method {
            HttpMethod::Get => item.get = Some(operation),
            HttpMethod::Post => item.post = Some(operation),
            HttpMethod::Put => item.put = Some(operation),
            HttpMethod::Delete => item.delete = Some(operation),
            HttpMethod::Patch => item.patch = Some(operation),
        }
        self
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct Info {
    pub title: String,
    pub version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

#[derive(Clone, Debug, Default, Serialize)]
struct Components {
    schemas: BTreeMap<String, Schema>,
}

//endregion

//region Tag

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Tag {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

impl Tag {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: None,
        }
    }
}

//endregion

//region Path

/// One piece of a URL path
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PathSegment {
    /// A fixed segment, such as `neurons`. Do not include `/`
    Literal(String),
    /// A `{name}` path parameter
    Parameter(PathParameter),
}

/// A path parameter
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PathParameter {
    /// Parameter name, without braces. Written into the path as `{name}`
    pub name: String,
    pub description: Option<String>,
    pub schema: Schema,
}

impl PathParameter {
    pub fn new(name: impl Into<String>, schema: Schema) -> Self {
        Self {
            name: name.into(),
            description: None,
            schema,
        }
    }
}

//endregion

//region Query Parameter

/// A URL query parameter
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct QueryParameter {
    pub name: String,
    pub description: Option<String>,
    pub required: bool,
    pub schema: Schema,
}

//endregion

//region Request Body

/// JSON request body
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct RequestBody {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub required: bool,
    content: JsonContent,
}

impl RequestBody {
    /// Creates a required `application/json` body
    pub fn json(schema: Schema) -> Self {
        Self {
            description: None,
            required: true,
            content: JsonContent::new(schema),
        }
    }
}

//endregion

/// `content` object with a single JSON media type
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
struct JsonContent {
    #[serde(rename = "application/json")]
    json: JsonMediaSchema,
}

impl JsonContent {
    fn new(schema: Schema) -> Self {
        Self {
            json: JsonMediaSchema { schema },
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
struct JsonMediaSchema {
    schema: Schema,
}

//region Response

/// One HTTP response. The status code becomes the OpenAPI responses key
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResponseSpec {
    pub status_code: u16,
    pub description: String,
    /// JSON body schema. `None` omits `content`
    pub schema: Option<Schema>,
}

impl ResponseSpec {
    pub fn new(status_code: u16, description: impl Into<String>) -> Self {
        Self {
            status_code,
            description: description.into(),
            schema: None,
        }
    }

    pub fn json(status_code: u16, description: impl Into<String>, schema: Schema) -> Self {
        Self {
            status_code,
            description: description.into(),
            schema: Some(schema),
        }
    }

    fn into_object(self) -> ResponseObject {
        ResponseObject {
            description: self.description,
            content: self.schema.map(JsonContent::new),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
struct ResponseObject {
    description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    content: Option<JsonContent>,
}

//endregion

//region Endpoints

/// GET operation. There is no request-body method on this type
#[derive(Clone, Debug)]
pub struct GetEndpoint {
    pub path: Vec<PathSegment>,
    pub description: Option<String>,
    pub tags: Vec<String>,
    pub query_parameters: Vec<QueryParameter>,
    responses: BTreeMap<String, ResponseObject>,
}

impl GetEndpoint {
    pub fn new(path: Vec<PathSegment>) -> Self {
        Self {
            path,
            description: None,
            tags: Vec::new(),
            query_parameters: Vec::new(),
            responses: BTreeMap::new(),
        }
    }

    pub fn response(mut self, response: ResponseSpec) -> Self {
        upsert_response(&mut self.responses, response);
        self
    }
}

#[derive(Clone, Debug)]
pub struct PayloadEndpoint {
    pub path: Vec<PathSegment>,
    pub description: Option<String>,
    pub tags: Vec<String>,
    pub query_parameters: Vec<QueryParameter>,
    pub request_body: Option<RequestBody>,
    responses: BTreeMap<String, ResponseObject>,
}

impl PayloadEndpoint {
    pub fn new(path: Vec<PathSegment>) -> Self {
        Self {
            path,
            description: None,
            tags: Vec::new(),
            query_parameters: Vec::new(),
            request_body: None,
            responses: BTreeMap::new(),
        }
    }

    /// Adds a response
    pub fn response(mut self, response: ResponseSpec) -> Self {
        upsert_response(&mut self.responses, response);
        self
    }
}

//endregion

//region Schema

/// A type that can produce the OpenAPI schema registered for it
pub trait OpenApiSchema {
    /// Component name under components.schemas
    fn schema_name(&self) -> &str;

    /// OpenAPI schema for this object
    fn open_api_schema(&self) -> Schema;
}

/// A JSON schema used
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Schema {
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    json_type: Option<&'static str>,
    #[serde(rename = "$ref", skip_serializing_if = "Option::is_none")]
    reference: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    items: Option<Box<Schema>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    properties: Option<BTreeMap<String, Schema>>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    required: Vec<String>,
    /// OpenAPI 3.0 ignores keywords beside `$ref`, so a described reference
    /// stores the pointer here
    #[serde(rename = "allOf", skip_serializing_if = "Option::is_none")]
    all_of: Option<Vec<SchemaPointer>>,
}

impl Schema {
    /// Boolean schema with no description
    pub fn boolean() -> Self {
        Self::primitive("boolean")
    }

    /// Integer schema with no description
    pub fn integer() -> Self {
        Self::primitive("integer")
    }

    /// Number schema with no description
    pub fn number() -> Self {
        Self::primitive("number")
    }

    /// String schema with no description
    pub fn string() -> Self {
        Self::primitive("string")
    }

    /// Array schema whose elements use `items`
    pub fn array(items: Schema) -> Self {
        Self {
            items: Some(Box::new(items)),
            ..Self::primitive("array")
        }
    }

    /// Object schema. Field documentation lives on each field's schema, or
    /// on [`ObjectField::description`]
    pub fn object(fields: Vec<ObjectField>) -> Self {
        let mut properties = BTreeMap::new();
        let mut required = Vec::new();
        for field in fields {
            if field.required {
                required.push(field.name.clone());
            }
            properties.insert(field.name, field.schema);
        }
        Self {
            properties: Some(properties),
            required,
            ..Self::primitive("object")
        }
    }

    /// Reference to a named component schema
    pub fn reference(name: impl Into<String>) -> Self {
        Self {
            reference: Some(schema_pointer(&name.into())),
            ..Self::empty()
        }
    }

    /// Sets the description on this schema
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        if let Some(pointer) = self.reference.take() {
            self.all_of = Some(vec![SchemaPointer { pointer }]);
        }
        self.description = Some(description.into());
        self
    }

    fn primitive(json_type: &'static str) -> Self {
        Self {
            json_type: Some(json_type),
            ..Self::empty()
        }
    }

    fn empty() -> Self {
        Self {
            json_type: None,
            reference: None,
            description: None,
            items: None,
            properties: None,
            required: Vec::new(),
            all_of: None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
struct SchemaPointer {
    #[serde(rename = "$ref")]
    pointer: String,
}

/// One property of an object schema
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ObjectField {
    pub name: String,
    pub required: bool,
    pub schema: Schema,
}

impl ObjectField {
    pub fn required(name: impl Into<String>, schema: Schema) -> Self {
        Self {
            name: name.into(),
            required: true,
            schema,
        }
    }

    pub fn optional(name: impl Into<String>, schema: Schema) -> Self {
        Self {
            name: name.into(),
            required: false,
            schema,
        }
    }

    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.schema = self.schema.with_description(description);
        self
    }
}

//endregion

//region Internal

#[derive(Clone, Copy, Debug)]
enum HttpMethod {
    Get,
    Post,
    Put,
    Delete,
    Patch,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
enum ParameterLocation {
    #[serde(rename = "path")]
    Path,
    #[serde(rename = "query")]
    Query,
}

#[derive(Clone, Debug, Default, Serialize)]
struct PathItem {
    #[serde(skip_serializing_if = "Option::is_none")]
    get: Option<Operation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    post: Option<Operation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    put: Option<Operation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    delete: Option<Operation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    patch: Option<Operation>,
}

#[derive(Clone, Debug, Serialize)]
struct Operation {
    #[serde(skip_serializing_if = "Vec::is_empty")]
    tags: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    parameters: Vec<Parameter>,
    #[serde(rename = "requestBody", skip_serializing_if = "Option::is_none")]
    request_body: Option<RequestBody>,
    responses: BTreeMap<String, ResponseObject>,
}

/// A parameter stored on an operation, including ones taken from the path
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
struct Parameter {
    name: String,
    #[serde(rename = "in")]
    location: ParameterLocation,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    required: bool,
    schema: Schema,
}

impl Parameter {
    fn query(parameter: QueryParameter) -> Self {
        Self {
            name: parameter.name,
            location: ParameterLocation::Query,
            description: parameter.description,
            required: parameter.required,
            schema: parameter.schema,
        }
    }
}

/// Replaces a response with the same status code, or inserts it
fn upsert_response(responses: &mut BTreeMap<String, ResponseObject>, response: ResponseSpec) {
    responses.insert(response.status_code.to_string(), response.into_object());
}

fn compile_path(segments: Vec<PathSegment>) -> (String, Vec<Parameter>) {
    if segments.is_empty() {
        return ("/".to_string(), Vec::new());
    }

    let mut path = String::new();
    let mut parameters = Vec::new();
    for segment in segments {
        path.push('/');
        match segment {
            PathSegment::Literal(literal) => path.push_str(&literal),
            PathSegment::Parameter(parameter) => {
                path.push('{');
                path.push_str(&parameter.name);
                path.push('}');
                parameters.push(Parameter {
                    name: parameter.name,
                    location: ParameterLocation::Path,
                    description: parameter.description,
                    required: true,
                    schema: parameter.schema,
                });
            }
        }
    }
    (path, parameters)
}


fn schema_pointer(name: &str) -> String {
    let mut reference = String::from("#/components/schemas/");
    for character in name.chars() {
        match character {
            '~' => reference.push_str("~0"),
            '/' => reference.push_str("~1"),
            other => reference.push(other),
        }
    }
    reference
}

//endregion