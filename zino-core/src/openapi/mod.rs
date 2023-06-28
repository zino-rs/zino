//! OpenAPI specification and API documentation.

use crate::{application, extension::TomlTableExt};
use convert_case::{Case, Casing};
use std::{
    collections::btree_map::BTreeMap,
    fs,
    io::ErrorKind,
    sync::{LazyLock, OnceLock},
};
use toml::Table;
use utoipa::openapi::{
    content::Content,
    path::{OperationBuilder, PathItem, Paths, PathsBuilder},
    response::ResponseBuilder,
    schema::{
        Components, ComponentsBuilder, KnownFormat, Object, ObjectBuilder, Ref, SchemaFormat,
        SchemaType,
    },
};

mod parser;

/// Returns the default OpenAPI paths.
pub(crate) fn default_paths() -> Paths {
    let mut paths_builder = PathsBuilder::new();
    for (path, item) in OPENAPI_PATHS.iter() {
        paths_builder = paths_builder.path(path, item.clone());
    }
    paths_builder.build()
}

/// Returns the default OpenAPI components.
pub(crate) fn default_components() -> Components {
    let mut components = OPENAPI_COMPONENTS.get_or_init(Components::new).clone();
    let status_schema = ObjectBuilder::new()
        .schema_type(SchemaType::Integer)
        .example(Some(200.into()))
        .build();
    let message_schema = ObjectBuilder::new()
        .schema_type(SchemaType::String)
        .example(Some("OK".into()))
        .build();
    let success_schema = ObjectBuilder::new()
        .schema_type(SchemaType::Boolean)
        .example(Some(true.into()))
        .build();
    let request_id_schema = ObjectBuilder::new()
        .schema_type(SchemaType::String)
        .format(Some(SchemaFormat::KnownFormat(KnownFormat::Uuid)))
        .build();
    let response_object = ObjectBuilder::new()
        .schema_type(SchemaType::Object)
        .property("status", status_schema)
        .property("success", success_schema)
        .property("message", message_schema)
        .property("request_id", request_id_schema)
        .property("data", Object::new())
        .build();
    components
        .schemas
        .insert("defaultResponse".to_owned(), response_object.into());
    components
}

/// OpenAPI paths.
static OPENAPI_PATHS: LazyLock<BTreeMap<String, PathItem>> = LazyLock::new(|| {
    let mut paths: BTreeMap<String, PathItem> = BTreeMap::new();
    let openapi_dir = application::PROJECT_DIR.join("./config/openapi");
    match fs::read_dir(openapi_dir) {
        Ok(entries) => {
            let mut components_builder = ComponentsBuilder::new();
            let files = entries.filter_map(|entry| entry.ok());
            for file in files {
                let openapi_file = file.path();
                let openapi_config = fs::read_to_string(&openapi_file)
                    .unwrap_or_else(|err| {
                        let openapi_file = openapi_file.to_string_lossy();
                        panic!("fail to read the OpenAPI file `{openapi_file}`: {err}");
                    })
                    .parse::<Table>()
                    .expect("fail to parse the OpenAPI file as a TOML table");
                if let Some(endpoints) = openapi_config.get_array("endpoints") {
                    for endpoint in endpoints.iter().filter_map(|v| v.as_table()) {
                        let path = endpoint.get_str("path").unwrap_or("/");
                        let method = endpoint
                            .get_str("method")
                            .unwrap_or_default()
                            .to_ascii_uppercase();
                        let path_item_type = parser::parse_path_item_type(&method);
                        let response = ResponseBuilder::new()
                            .content(
                                "application/json",
                                Content::new(Ref::from_schema_name("defaultResponse")),
                            )
                            .build();
                        let mut operation_builder =
                            OperationBuilder::new().response("default", response);
                        for parameter in parser::parse_path_parameters(path).into_iter() {
                            operation_builder = operation_builder.parameter(parameter);
                        }
                        if let Some(query) = endpoint.get_table("query") {
                            for parameter in parser::parse_query_parameters(query).into_iter() {
                                operation_builder = operation_builder.parameter(parameter);
                            }
                        }
                        if let Some(body) = endpoint.get_table("requestBody") {
                            let request_body = parser::parse_request_body(body);
                            operation_builder = operation_builder.request_body(Some(request_body));
                        }
                        if let Some(item) = paths.get_mut(path) {
                            item.operations
                                .insert(path_item_type, operation_builder.into());
                        } else {
                            let path_item = PathItem::new(path_item_type, operation_builder);
                            paths.insert(path.to_owned(), path_item);
                        }
                    }
                }
                if let Some(schemas) = openapi_config.get_table("schemas") {
                    for (key, value) in schemas.iter() {
                        if let Some(config) = value.as_table() {
                            let schema_name = key.to_case(Case::Camel);
                            let schema = parser::parse_schema_object(config);
                            components_builder = components_builder.schema(schema_name, schema);
                        }
                    }
                }
            }
            if OPENAPI_COMPONENTS.set(components_builder.build()).is_err() {
                panic!("fail to set OpenAPI components");
            }
        }
        Err(err) => {
            if err.kind() != ErrorKind::NotFound {
                tracing::error!("{err}");
            }
        }
    }
    paths
});

/// OpenAPI components.
static OPENAPI_COMPONENTS: OnceLock<Components> = OnceLock::new();
