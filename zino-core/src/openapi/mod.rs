//! OpenAPI specification and API documentation.

use crate::{application, extension::TomlTableExt, response::WebHook, LazyLock, Uuid};
use ahash::{HashMap, HashMapExt};
use convert_case::{Case, Casing};
use serde_json::json;
use std::{collections::BTreeMap, fs, io::ErrorKind, sync::OnceLock};
use toml::Table;
use utoipa::openapi::{
    content::ContentBuilder,
    external_docs::ExternalDocs,
    info::{Contact, Info, License},
    path::{PathItem, Paths, PathsBuilder},
    response::ResponseBuilder,
    schema::{
        Components, ComponentsBuilder, KnownFormat, Object, ObjectBuilder, Ref, SchemaFormat,
        SchemaType,
    },
    security::SecurityRequirement,
    server::Server,
    tag::Tag,
};

mod model;
mod parser;
mod webhook;

pub(crate) use model::translate_model_entry;
pub(crate) use webhook::get_webhook;

/// Constructs the OpenAPI `Info` object.
pub(crate) fn openapi_info(title: &str, version: &str) -> Info {
    let mut info = Info::new(title, version);
    if let Some(config) = OPENAPI_INFO.get() {
        if let Some(title) = config.get_str("title") {
            title.clone_into(&mut info.title);
        }
        if let Some(description) = config.get_str("description") {
            info.description = Some(description.to_owned());
        }
        if let Some(terms_of_service) = config.get_str("terms_of_service") {
            info.terms_of_service = Some(terms_of_service.to_owned());
        }
        if let Some(contact_config) = config.get_table("contact") {
            let mut contact = Contact::new();
            if let Some(contact_name) = contact_config.get_str("name") {
                contact.name = Some(contact_name.to_owned());
            }
            if let Some(contact_url) = contact_config.get_str("url") {
                contact.url = Some(contact_url.to_owned());
            }
            if let Some(contact_email) = contact_config.get_str("email") {
                contact.email = Some(contact_email.to_owned());
            }
            info.contact = Some(contact);
        }
        if let Some(license) = config.get_str("license") {
            info.license = Some(License::new(license));
        } else if let Some(license_config) = config.get_table("license") {
            let license_name = license_config.get_str("name").unwrap_or_default();
            let mut license = License::new(license_name);
            if let Some(license_url) = license_config.get_str("url") {
                license.url = Some(license_url.to_owned());
            }
            info.license = Some(license);
        }
        if let Some(version) = config.get_str("version") {
            version.clone_into(&mut info.version);
        }
    }
    info
}

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
    let request_id_example = Uuid::now_v7();
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

    // 4XX error response
    let model_id_example = Uuid::now_v7();
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
        .insert("4XX".to_owned(), error_response.into());

    components
}

/// Returns the default OpenAPI tags.
pub(crate) fn default_tags() -> Vec<Tag> {
    OPENAPI_TAGS.get_or_init(Vec::new).clone()
}

/// Returns the default OpenAPI servers.
pub(crate) fn default_servers() -> Vec<Server> {
    OPENAPI_SERVERS
        .get_or_init(|| vec![Server::new("/")])
        .clone()
}

/// Returns the default OpenAPI security requirements.
pub(crate) fn default_securities() -> Vec<SecurityRequirement> {
    OPENAPI_SECURITIES.get_or_init(Vec::new).clone()
}

/// Returns the default OpenAPI external docs.
pub(crate) fn default_external_docs() -> Option<ExternalDocs> {
    OPENAPI_EXTERNAL_DOCS.get().cloned()
}

/// OpenAPI paths.
static OPENAPI_PATHS: LazyLock<BTreeMap<String, PathItem>> = LazyLock::new(|| {
    let mut paths: BTreeMap<String, PathItem> = BTreeMap::new();
    let openapi_dir = application::PROJECT_DIR.join("./config/openapi");
    match fs::read_dir(openapi_dir) {
        Ok(entries) => {
            let mut openapi_tags = Vec::new();
            let mut model_definitions = HashMap::new();
            let mut webhook_definitions = HashMap::new();
            let mut components_builder = ComponentsBuilder::new();
            let files = entries.filter_map(|entry| entry.ok());
            for file in files {
                let openapi_file = file.path();
                let openapi_config = fs::read_to_string(&openapi_file)
                    .unwrap_or_else(|err| {
                        let openapi_file = openapi_file.display();
                        panic!("fail to read the OpenAPI file `{openapi_file}`: {err}");
                    })
                    .parse::<Table>()
                    .expect("fail to parse the OpenAPI file as a TOML table");
                if file.file_name() == "OPENAPI.toml" {
                    if let Some(info_config) = openapi_config.get_table("info") {
                        if OPENAPI_INFO.set(info_config.clone()).is_err() {
                            panic!("fail to set OpenAPI info");
                        }
                    }
                    if let Some(servers) = openapi_config.get_array("servers") {
                        let servers = servers
                            .iter()
                            .filter_map(|v| v.as_table())
                            .map(parser::parse_server)
                            .collect::<Vec<_>>();
                        if OPENAPI_SERVERS.set(servers).is_err() {
                            panic!("fail to set OpenAPI servers");
                        }
                    }
                    if let Some(security_schemes) = openapi_config.get_table("security_schemes") {
                        for (name, scheme) in security_schemes {
                            if let Some(scheme_config) = scheme.as_table() {
                                let scheme = parser::parse_security_scheme(scheme_config);
                                components_builder =
                                    components_builder.security_scheme(name, scheme);
                            }
                        }
                    }
                    if let Some(securities) = openapi_config.get_array("securities") {
                        let security_requirements = securities
                            .iter()
                            .filter_map(|v| v.as_table())
                            .map(parser::parse_security_requirement)
                            .collect::<Vec<_>>();
                        if OPENAPI_SECURITIES.set(security_requirements).is_err() {
                            panic!("fail to set OpenAPI security requirements");
                        }
                    }
                    if let Some(external_docs) = openapi_config.get_table("external_docs") {
                        let external_docs = parser::parse_external_docs(external_docs);
                        if OPENAPI_EXTERNAL_DOCS.set(external_docs).is_err() {
                            panic!("fail to set OpenAPI external docs");
                        }
                    }
                    continue;
                }

                let name = openapi_config
                    .get_str("name")
                    .map(|s| s.to_owned())
                    .unwrap_or_else(|| {
                        file.file_name()
                            .to_string_lossy()
                            .trim_end_matches(".toml")
                            .to_owned()
                    });
                let ignore_securities = openapi_config
                    .get_array("securities")
                    .is_some_and(|v| v.is_empty());
                if let Some(endpoints) = openapi_config.get_array("endpoints") {
                    for endpoint in endpoints.iter().filter_map(|v| v.as_table()) {
                        let path = endpoint.get_str("path").unwrap_or("/");
                        let method = endpoint
                            .get_str("method")
                            .unwrap_or_default()
                            .to_ascii_uppercase();
                        let path_item_type = parser::parse_path_item_type(&method);
                        let operation =
                            parser::parse_operation(&name, path, endpoint, ignore_securities);
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
                            let name = key.to_case(Case::Camel);
                            let schema = parser::parse_schema(config);
                            components_builder = components_builder.schema(name, schema);
                        }
                    }
                }
                if let Some(responses) = openapi_config.get_table("responses") {
                    for (key, value) in responses.iter() {
                        if let Some(config) = value.as_table() {
                            let name = key.to_case(Case::Camel);
                            let response = parser::parse_response(config);
                            components_builder = components_builder.response(name, response);
                        }
                    }
                }
                if let Some(models) = openapi_config.get_table("models") {
                    for (model_name, model_fields) in models {
                        if let Some(fields) = model_fields.as_table() {
                            let model_name = model_name.to_owned().leak() as &'static str;
                            model_definitions.insert(model_name, fields.to_owned());
                        }
                    }
                }
                if let Some(webhooks) = openapi_config.get_table("webhooks") {
                    for (webhook_name, webhook_request) in webhooks {
                        if let Some(request) = webhook_request.as_table() {
                            if let Ok(webhook) = WebHook::try_new(request) {
                                let webhook_name = webhook_name.to_owned().leak() as &'static str;
                                webhook_definitions.insert(webhook_name, webhook);
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
            if MODEL_DEFINITIONS.set(model_definitions).is_err() {
                panic!("fail to set model definitions");
            }
            if WEBHOOK_DEFINITIONS.set(webhook_definitions).is_err() {
                panic!("fail to set webhook definitions");
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

/// OpenAPI info.
static OPENAPI_INFO: OnceLock<Table> = OnceLock::new();

/// OpenAPI components.
static OPENAPI_COMPONENTS: OnceLock<Components> = OnceLock::new();

/// OpenAPI tags.
static OPENAPI_TAGS: OnceLock<Vec<Tag>> = OnceLock::new();

/// OpenAPI servers.
static OPENAPI_SERVERS: OnceLock<Vec<Server>> = OnceLock::new();

/// OpenAPI securities.
static OPENAPI_SECURITIES: OnceLock<Vec<SecurityRequirement>> = OnceLock::new();

/// OpenAPI external docs.
static OPENAPI_EXTERNAL_DOCS: OnceLock<ExternalDocs> = OnceLock::new();

/// Model definitions.
static MODEL_DEFINITIONS: OnceLock<HashMap<&str, Table>> = OnceLock::new();

/// WebHook definitions.
static WEBHOOK_DEFINITIONS: OnceLock<HashMap<&str, WebHook>> = OnceLock::new();
