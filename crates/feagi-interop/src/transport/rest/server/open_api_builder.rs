//! OpenAPI 3.0.3 document of the endpoints a REST server serves, written from the descriptions
//! of its routes
//!
//! Written once when the server starts rather than at compile time, as the paths contain the
//! values of the categories' configurables, which are only known at runtime. Everything else
//! comes from const descriptions generated with the messages

use core::fmt::{Result, Write};
use feagi_basis::feagi_message::feagi_message_description::{FeagiFieldDescription, FeagiFieldKind, FeagiMessagePathElementDescription};
use feagi_basis::feagi_message::request::feagi_request_descriptor::FeagiMessageEnum;

use super::server_basis::routing::{RestCategoryRoutes, RouteDescription};

const FEAGI_INFO_TITLE: &str = "FEAGI";
const FEAGI_INFO_VERSION: &str = "0.0.0"; // TODO package var?
const FEAGI_INFO_DESCRIPTION: &str = "Feagi Server Swagger API Docs";
const OPENAPI_VERSION: &str = "3.0.3";

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

/// Writes the OpenAPI document of every route in `routes`. Each category is a tag, and messages
/// sharing a path are written as the methods of that one path
pub fn write_open_api_json<Routes: RestCategoryRoutes, W: Write>(routes: &Routes, out: &mut W) -> Result {
    out.write_str("{\"openapi\":")?;
    write_json_string(out, OPENAPI_VERSION)?;
    out.write_str(",\"info\":{\"title\":")?;
    write_json_string(out, FEAGI_INFO_TITLE)?;
    out.write_str(",\"version\":")?;
    write_json_string(out, FEAGI_INFO_VERSION)?;
    out.write_str(",\"description\":")?;
    write_json_string(out, FEAGI_INFO_DESCRIPTION)?;
    out.write_str("}")?;
    write_tags(routes, out)?;
    write_paths(routes, out)?;
    out.write_str("}")
}

/// One tag per category, in the order categories first appear
fn write_tags<Routes: RestCategoryRoutes, W: Write>(routes: &Routes, out: &mut W) -> Result {
    out.write_str(",\"tags\":[")?;
    let mut wrote_tag = false;
    let mut index = 0;
    while let Some(route) = routes.route_description(index) {
        let first_of_category = (0..index).all(|earlier| {
            routes
                .route_description(earlier)
                .is_none_or(|earlier| earlier.category_name != route.category_name)
        });
        if first_of_category {
            if wrote_tag {
                out.write_str(",")?;
            }
            wrote_tag = true;
            out.write_str("{\"name\":")?;
            write_json_string(out, route.category_name)?;
            out.write_str(",\"description\":")?;
            write_json_string(out, route.category_description)?;
            out.write_str("}")?;
        }
        index += 1;
    }
    out.write_str("]")
}

/// Each distinct path once, with every method of the routes sharing it
fn write_paths<Routes: RestCategoryRoutes, W: Write>(routes: &Routes, out: &mut W) -> Result {
    out.write_str(",\"paths\":{")?;
    let mut wrote_path = false;
    let mut index = 0;
    while let Some(route) = routes.route_description(index) {
        let first_of_path = (0..index).all(|earlier| routes.route_description(earlier).is_none_or(|earlier| !same_path(&earlier, &route)));
        if first_of_path {
            if wrote_path {
                out.write_str(",")?;
            }
            wrote_path = true;
            write_path_key(out, &route)?;
            out.write_str(":{")?;
            let mut wrote_method = false;
            for (method, method_key) in METHODS {
                let mut other = 0;
                while let Some(candidate) = routes.route_description(other) {
                    if candidate.method == method && same_path(&candidate, &route) {
                        if wrote_method {
                            out.write_str(",")?;
                        }
                        wrote_method = true;
                        write_json_string(out, method_key)?;
                        out.write_str(":")?;
                        write_operation(out, &candidate)?;
                        break;
                    }
                    other += 1;
                }
            }
            out.write_str("}")?;
        }
        index += 1;
    }
    out.write_str("}")
}

fn write_operation<W: Write>(out: &mut W, route: &RouteDescription) -> Result {
    let payload_in_query = route.method == FeagiMessageEnum::Read;

    out.write_str("{\"tags\":[")?;
    write_json_string(out, route.category_name)?;
    out.write_str("],\"description\":")?;
    write_json_string(out, route.description)?;

    out.write_str(",\"parameters\":[")?;
    let mut wrote_parameter = false;
    for element in route.path_elements {
        if let FeagiMessagePathElementDescription::Parameter { name, description, kind } = element {
            if wrote_parameter {
                out.write_str(",")?;
            }
            wrote_parameter = true;
            write_parameter(out, name, "path", *description, true, *kind)?;
        }
    }
    if payload_in_query {
        for field in route.request_payload_fields {
            if wrote_parameter {
                out.write_str(",")?;
            }
            wrote_parameter = true;
            write_parameter(out, field.name, "query", field.description, field.required, field.kind)?;
        }
    }
    out.write_str("]")?;

    if !payload_in_query && !route.request_payload_fields.is_empty() {
        out.write_str(",\"requestBody\":{\"required\":true,\"content\":{\"application/json\":{\"schema\":")?;
        write_object_schema(out, route.request_payload_fields)?;
        out.write_str("}}}")?;
    }

    out.write_str(",\"responses\":{\"200\":{\"description\":\"Successful response\"")?;
    if !route.response_payload_fields.is_empty() {
        out.write_str(",\"content\":{\"application/json\":{\"schema\":")?;
        write_object_schema(out, route.response_payload_fields)?;
        out.write_str("}}")?;
    }
    out.write_str("}")?;
    out.write_str(",\"400\":{\"description\":\"The request payload could not be parsed, or the message responded with its error\"}")?;
    out.write_str(",\"503\":{\"description\":\"The request could not be sent to, or answered by, the worker serving its category\"}")?;
    out.write_str("}}")
}

fn write_parameter<W: Write>(out: &mut W, name: &str, location: &str, description: Option<&str>, required: bool, kind: FeagiFieldKind) -> Result {
    out.write_str("{\"name\":")?;
    write_json_string(out, name)?;
    out.write_str(",\"in\":")?;
    write_json_string(out, location)?;
    if let Some(description) = description {
        out.write_str(",\"description\":")?;
        write_json_string(out, description)?;
    }
    out.write_str(if required { ",\"required\":true" } else { ",\"required\":false" })?;
    out.write_str(",\"schema\":")?;
    write_kind_schema(out, kind, None)?;
    out.write_str("}")
}

fn write_object_schema<W: Write>(out: &mut W, fields: &[FeagiFieldDescription]) -> Result {
    out.write_str("{\"type\":\"object\",\"properties\":{")?;
    for (index, field) in fields.iter().enumerate() {
        if index != 0 {
            out.write_str(",")?;
        }
        write_json_string(out, field.name)?;
        out.write_str(":")?;
        write_kind_schema(out, field.kind, field.description)?;
    }
    out.write_str("}")?;
    if fields.iter().any(|field| field.required) {
        out.write_str(",\"required\":[")?;
        let mut wrote_required = false;
        for field in fields.iter().filter(|field| field.required) {
            if wrote_required {
                out.write_str(",")?;
            }
            wrote_required = true;
            write_json_string(out, field.name)?;
        }
        out.write_str("]")?;
    }
    out.write_str("}")
}

fn write_kind_schema<W: Write>(out: &mut W, kind: FeagiFieldKind, description: Option<&str>) -> Result {
    out.write_str(match kind {
        FeagiFieldKind::Boolean => "{\"type\":\"boolean\"",
        FeagiFieldKind::Integer => "{\"type\":\"integer\"",
        FeagiFieldKind::Number => "{\"type\":\"number\"",
        FeagiFieldKind::String => "{\"type\":\"string\"",
        FeagiFieldKind::Array => "{\"type\":\"array\",\"items\":{}",
        FeagiFieldKind::Object => "{\"type\":\"object\"",
    })?;
    if let Some(description) = description {
        out.write_str(",\"description\":")?;
        write_json_string(out, description)?;
    }
    out.write_str("}")
}

//region Paths

/// One segment of a route's path, with configurables replaced by their values
#[derive(PartialEq)]
enum ResolvedSegment<'a> {
    Text(&'a str),
    Parameter(&'a str),
}

fn resolved_segments<'a>(route: &'a RouteDescription) -> impl Iterator<Item = ResolvedSegment<'a>> {
    route.path_elements.iter().map(|element| match element {
        FeagiMessagePathElementDescription::Static(text) => ResolvedSegment::Text(text),
        // A configurable without a value is shown as a parameter, as it has to be filled in
        FeagiMessagePathElementDescription::Configurable { name } => match route.configurables.configurable_str(name) {
            Some(value) => ResolvedSegment::Text(value),
            None => ResolvedSegment::Parameter(name),
        },
        FeagiMessagePathElementDescription::Parameter { name, .. } => ResolvedSegment::Parameter(name),
    })
}

fn same_path(left: &RouteDescription, right: &RouteDescription) -> bool {
    resolved_segments(left).eq(resolved_segments(right))
}

/// The path as the json key, with parameters as `{name}`
fn write_path_key<W: Write>(out: &mut W, route: &RouteDescription) -> Result {
    out.write_str("\"")?;
    for segment in resolved_segments(route) {
        out.write_str("/")?;
        match segment {
            ResolvedSegment::Text(text) => write_escaped(out, text)?,
            ResolvedSegment::Parameter(name) => {
                out.write_str("{")?;
                write_escaped(out, name)?;
                out.write_str("}")?;
            }
        }
    }
    out.write_str("\"")
}

//endregion

//region JSON

fn write_json_string<W: Write>(out: &mut W, text: &str) -> Result {
    out.write_str("\"")?;
    write_escaped(out, text)?;
    out.write_str("\"")
}

fn write_escaped<W: Write>(out: &mut W, text: &str) -> Result {
    for character in text.chars() {
        match character {
            '"' => out.write_str("\\\"")?,
            '\\' => out.write_str("\\\\")?,
            '\n' => out.write_str("\\n")?,
            '\r' => out.write_str("\\r")?,
            '\t' => out.write_str("\\t")?,
            character if (character as u32) < 0x20 => write!(out, "\\u{:04x}", character as u32)?,
            character => out.write_char(character)?,
        }
    }
    Ok(())
}

//endregion
