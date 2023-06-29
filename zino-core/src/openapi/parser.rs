use crate::extension::TomlTableExt;
use convert_case::{Case, Casing};
use toml::Table;
use utoipa::openapi::{
    content::Content,
    path::{Parameter, ParameterBuilder, ParameterIn, PathItemType},
    request_body::{RequestBody, RequestBodyBuilder},
    schema::{KnownFormat, Object, ObjectBuilder, Ref, SchemaFormat, SchemaType},
    Required,
};

/// Parses the path parameters.
pub(super) fn parse_path_parameters(path: &str) -> Vec<Parameter> {
    let mut parameters = Vec::new();
    for segment in path.split('/') {
        if let Some(part) = segment.strip_prefix('{') && let Some(name) = part.strip_suffix('}') {
            let schema_name = name.to_case(Case::Camel);
            let parameter = ParameterBuilder::new()
                .name(name)
                .schema(Some(Ref::from_schema_name(schema_name)))
                .parameter_in(ParameterIn::Path)
                .required(Required::True)
                .build();
            parameters.push(parameter);
        }
    }
    parameters
}

/// Parses the query parameters.
pub(super) fn parse_query_parameters(query: &Table) -> Vec<Parameter> {
    let mut parameters = Vec::new();
    for (key, value) in query {
        let mut parameter_builder = ParameterBuilder::new()
            .name(key)
            .parameter_in(ParameterIn::Query);
        if let Some(config) = value.as_table() {
            if let Some(schema) = config.get_str("schema") {
                let schema_name = schema.to_case(Case::Camel);
                let schema_object = Ref::from_schema_name(schema_name);
                parameter_builder = parameter_builder.schema(Some(schema_object));
            } else {
                let object = parse_schema_object(config);
                parameter_builder = parameter_builder.schema(Some(object));
            };
        } else if let Some(basic_type) = value.as_str() {
            let object = Object::with_type(parse_schema_type(basic_type));
            parameter_builder = parameter_builder.schema(Some(object));
        }
        parameters.push(parameter_builder.build());
    }
    parameters
}

/// Parses the request body.
pub(super) fn parse_request_body(config: &Table) -> RequestBody {
    let mut body_builder = RequestBodyBuilder::new().required(Some(Required::True));
    if let Some(schema) = config.get_str("schema") {
        body_builder = body_builder.content(
            "application/json",
            Content::new(Ref::from_schema_name(schema)),
        );
    }
    body_builder.build()
}

/// Parses the schema.
pub(super) fn parse_schema_object(config: &Table) -> Object {
    let mut object_builder = ObjectBuilder::new();
    for (key, value) in config {
        match key.as_str() {
            "type" => {
                let schema_type = parse_schema_type(value.as_str().unwrap_or_default());
                object_builder = object_builder.schema_type(schema_type);
            }
            "format" => {
                if let Some(format) = value.as_str() {
                    let schema_format = parse_schema_format(format);
                    object_builder = object_builder.format(Some(schema_format));
                }
            }
            "description" => {
                if let Some(config) = value.as_table() {
                    let object = parse_schema_object(config);
                    object_builder = object_builder.property(key, object);
                } else if let Some(description) = value.as_str() {
                    object_builder = object_builder.description(Some(description));
                }
            }
            "required" => {
                if let Some(required_fields) = value.as_array() {
                    for field in required_fields.iter().filter_map(|v| v.as_str()) {
                        object_builder = object_builder.required(field);
                    }
                }
            }
            _ => {
                if let Some(config) = value.as_table() {
                    let object = parse_schema_object(config);
                    object_builder = object_builder.property(key, object);
                } else if let Some(basic_type) = value.as_str() {
                    let object = Object::with_type(parse_schema_type(basic_type));
                    object_builder = object_builder.property(key, object);
                }
            }
        }
    }
    object_builder.build()
}

/// Parses the path item type.
pub(super) fn parse_path_item_type(method: &str) -> PathItemType {
    match method {
        "POST" => PathItemType::Post,
        "PUT" => PathItemType::Put,
        "DELETE" => PathItemType::Delete,
        "OPTIONS" => PathItemType::Options,
        "HEAD" => PathItemType::Head,
        "PATCH" => PathItemType::Patch,
        "TRACE" => PathItemType::Trace,
        "CONNECT" => PathItemType::Connect,
        _ => PathItemType::Get,
    }
}

/// Parses the schema type.
fn parse_schema_type(basic_type: &str) -> SchemaType {
    match basic_type {
        "boolean" => SchemaType::Boolean,
        "integer" => SchemaType::Integer,
        "number" => SchemaType::Number,
        "string" => SchemaType::String,
        "array" => SchemaType::Array,
        "object" => SchemaType::Object,
        _ => SchemaType::Value,
    }
}

/// Parses the schema format.
fn parse_schema_format(format: &str) -> SchemaFormat {
    match format {
        "int8" => SchemaFormat::KnownFormat(KnownFormat::Int8),
        "int16" => SchemaFormat::KnownFormat(KnownFormat::Int16),
        "int32" => SchemaFormat::KnownFormat(KnownFormat::Int32),
        "int64" => SchemaFormat::KnownFormat(KnownFormat::Int64),
        "uint8" => SchemaFormat::KnownFormat(KnownFormat::UInt8),
        "uint16" => SchemaFormat::KnownFormat(KnownFormat::UInt16),
        "uint32" => SchemaFormat::KnownFormat(KnownFormat::UInt32),
        "uint64" => SchemaFormat::KnownFormat(KnownFormat::UInt64),
        "float" => SchemaFormat::KnownFormat(KnownFormat::Float),
        "double" => SchemaFormat::KnownFormat(KnownFormat::Double),
        "byte" => SchemaFormat::KnownFormat(KnownFormat::Byte),
        "binary" => SchemaFormat::KnownFormat(KnownFormat::Binary),
        "date" => SchemaFormat::KnownFormat(KnownFormat::Date),
        "datetime" => SchemaFormat::KnownFormat(KnownFormat::DateTime),
        "password" => SchemaFormat::KnownFormat(KnownFormat::Password),
        "uuid" => SchemaFormat::KnownFormat(KnownFormat::Uuid),
        _ => SchemaFormat::Custom(format.to_owned()),
    }
}
