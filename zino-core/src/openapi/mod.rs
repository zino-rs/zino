//! OpenAPI specification and API documentation.

use crate::{application, extension::TomlTableExt, model::Translation, Uuid};
use convert_case::{Case, Casing};
use serde_json::json;
use std::{
    collections::{BTreeMap, HashMap},
    fs,
    io::ErrorKind,
    sync::{LazyLock, OnceLock},
};
use toml::Table;
use utoipa::openapi::{
    content::ContentBuilder,
    path::{PathItem, Paths, PathsBuilder},
    response::ResponseBuilder,
    schema::{
        Components, ComponentsBuilder, KnownFormat, Object, ObjectBuilder, Ref, SchemaFormat,
        SchemaType,
    },
    tag::Tag,
};

mod model;
mod parser;

pub(crate) use model::translate_model_entry;

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

    // Request ID
    let request_id_example = Uuid::new_v4();
    let request_id_schema = ObjectBuilder::new()
        .schema_type(SchemaType::String)
        .format(Some(SchemaFormat::KnownFormat(KnownFormat::Uuid)))
        .build();

    // Default response
    let status_schema = ObjectBuilder::new()
        .schema_type(SchemaType::Integer)
        .example(Some(200.into()))
        .build();
    let success_schema = ObjectBuilder::new()
        .schema_type(SchemaType::Boolean)
        .example(Some(true.into()))
        .build();
    let message_schema = ObjectBuilder::new()
        .schema_type(SchemaType::String)
        .example(Some("OK".into()))
        .build();
    let default_response_schema = ObjectBuilder::new()
        .schema_type(SchemaType::Object)
        .property("status", status_schema)
        .property("success", success_schema)
        .property("message", message_schema)
        .property("request_id", request_id_schema.clone())
        .property("data", Object::new())
        .required("status")
        .required("success")
        .required("message")
        .required("request_id")
        .build();
    let default_response_example = json!({
        "status": 200,
        "success": true,
        "message": "OK",
        "request_id": request_id_example,
        "data": {},
    });
    let default_response_content = ContentBuilder::new()
        .schema(Ref::from_schema_name("defaultResponse"))
        .example(Some(default_response_example))
        .build();
    let default_response = ResponseBuilder::new()
        .content("application/json", default_response_content)
        .build();
    components
        .schemas
        .insert("defaultResponse".to_owned(), default_response_schema.into());
    components
        .responses
        .insert("default".to_owned(), default_response.into());

    // Error response
    let model_id_example = Uuid::new_v4();
    let detail_example = format!("404 Not Found: cannot find the model `{model_id_example}`");
    let instance_example = format!("/model/{model_id_example}/view");
    let status_schema = ObjectBuilder::new()
        .schema_type(SchemaType::Integer)
        .example(Some(404.into()))
        .build();
    let success_schema = ObjectBuilder::new()
        .schema_type(SchemaType::Boolean)
        .example(Some(false.into()))
        .build();
    let title_schema = ObjectBuilder::new()
        .schema_type(SchemaType::String)
        .example(Some("NotFound".into()))
        .build();
    let detail_schema = ObjectBuilder::new()
        .schema_type(SchemaType::String)
        .example(Some(detail_example.as_str().into()))
        .build();
    let instance_schema = ObjectBuilder::new()
        .schema_type(SchemaType::String)
        .example(Some(instance_example.as_str().into()))
        .build();
    let error_response_schema = ObjectBuilder::new()
        .schema_type(SchemaType::Object)
        .property("status", status_schema)
        .property("success", success_schema)
        .property("title", title_schema)
        .property("detail", detail_schema)
        .property("instance", instance_schema)
        .property("request_id", request_id_schema)
        .required("status")
        .required("success")
        .required("title")
        .required("detail")
        .required("instance")
        .required("request_id")
        .build();
    let error_response_example = json!({
        "status": 404,
        "success": false,
        "title": "NotFound",
        "detail": detail_example,
        "instance": instance_example,
        "request_id": request_id_example,
    });
    let error_response_content = ContentBuilder::new()
        .schema(Ref::from_schema_name("errorResponse"))
        .example(Some(error_response_example))
        .build();
    let error_response = ResponseBuilder::new()
        .content("application/json", error_response_content)
        .build();
    components
        .schemas
        .insert("errorResponse".to_owned(), error_response_schema.into());
    components
        .responses
        .insert("error".to_owned(), error_response.into());

    components
}

/// Returns the default OpenAPI tags.
pub(crate) fn default_tags() -> Vec<Tag> {
    OPENAPI_TAGS.get_or_init(Vec::new).clone()
}

/// OpenAPI paths.
static OPENAPI_PATHS: LazyLock<BTreeMap<String, PathItem>> = LazyLock::new(|| {
    let mut paths: BTreeMap<String, PathItem> = BTreeMap::new();
    let openapi_dir = application::PROJECT_DIR.join("./config/openapi");
    match fs::read_dir(openapi_dir) {
        Ok(entries) => {
            let mut model_translation_keys = Vec::new();
            let mut model_translations = HashMap::new();
            let mut openapi_tags = Vec::new();
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
                let name = openapi_config
                    .get_str("name")
                    .map(|s| s.to_owned())
                    .unwrap_or_else(|| {
                        file.file_name()
                            .to_string_lossy()
                            .trim_end_matches(".toml")
                            .to_owned()
                    });
                if let Some(endpoints) = openapi_config.get_array("endpoints") {
                    for endpoint in endpoints.iter().filter_map(|v| v.as_table()) {
                        let path = endpoint.get_str("path").unwrap_or("/");
                        let method = endpoint
                            .get_str("method")
                            .unwrap_or_default()
                            .to_ascii_uppercase();
                        let path_item_type = parser::parse_path_item_type(&method);
                        let operation = parser::parse_operation(&name, path, endpoint);
                        if let Some(item) = paths.get_mut(path) {
                            item.operations.insert(path_item_type, operation);
                        } else {
                            let path_item = PathItem::new(path_item_type, operation);
                            paths.insert(path.to_owned(), path_item);
                        }
                    }
                }
                if let Some(schemas) = openapi_config.get_table("schemas") {
                    for (key, value) in schemas.iter() {
                        if let Some(config) = value.as_table() {
                            let schema_name = key.to_case(Case::Camel);
                            let schema = parser::parse_schema(config);
                            components_builder = components_builder.schema(schema_name, schema);
                        }
                    }
                }
                if let Some(models) = openapi_config.get_table("models") {
                    for (model_name, model_fields) in models {
                        if let Some(fields) = model_fields.as_table() {
                            for (field, value) in fields {
                                let translation = value.as_table().map(Translation::with_config);
                                if let Some(translation) = translation && translation.is_ready() {
                                    let model_name = model_name.to_case(Case::Snake);
                                    let model_key = format!("{model_name}.{field}.translations");
                                    let key: &'static str = model_key.leak();
                                    model_translation_keys.push(key);
                                    model_translations.insert(key, translation);
                                }
                            }
                        }
                    }
                }
                openapi_tags.push(parser::parse_tag(&name, &openapi_config))
            }
            if OPENAPI_COMPONENTS.set(components_builder.build()).is_err() {
                panic!("fail to set OpenAPI components");
            }
            if OPENAPI_TAGS.set(openapi_tags).is_err() {
                panic!("fail to set OpenAPI tags");
            }
            if !model_translation_keys.is_empty() {
                if model::MODEL_TRANSLATION_KEYS
                    .set(model_translation_keys)
                    .is_err()
                {
                    panic!("fail to set model translation keys");
                }
                if model::MODEL_TRANSLATIONS.set(model_translations).is_err() {
                    panic!("fail to set model translations");
                }
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

/// OpenAPI tags.
static OPENAPI_TAGS: OnceLock<Vec<Tag>> = OnceLock::new();
