use convert_case::{Case, Casing};
use std::collections::BTreeMap;
use toml::Table;
use utoipa::openapi::{
    Deprecated, RefOr, Required,
    content::{Content, ContentBuilder},
    external_docs::ExternalDocs,
    header::Header,
    path::{HttpMethod, Operation, OperationBuilder, Parameter, ParameterBuilder, ParameterIn},
    request_body::{RequestBody, RequestBodyBuilder},
    response::{Response, ResponseBuilder},
    schema::{
        Array, ArrayBuilder, KnownFormat, Object, ObjectBuilder, Ref, Schema, SchemaFormat, Type,
    },
    security::{
        ApiKey, ApiKeyValue, AuthorizationCode, ClientCredentials, Flow, HttpAuthScheme,
        HttpBuilder, Implicit, OAuth2, OpenIdConnect, Password, Scopes, SecurityRequirement,
        SecurityScheme,
    },
    server::{Server, ServerVariableBuilder},
    tag::{Tag, TagBuilder},
};
use zino_core::{
    TomlValue,
    extension::{TomlTableExt, TomlValueExt},
};

/// Parses the tag.
pub(super) fn parse_tag(name: &str, config: &Table) -> Tag {
    let mut tag_builder = TagBuilder::new().name(name);
    if let Some(name) = config.get_str("name") {
        tag_builder = tag_builder.name(name);
    }
    if let Some(description) = config.get_str("description") {
        tag_builder = tag_builder.description(Some(description));
    }
    if let Some(external_docs) = config.get_table("external_docs") {
        let external_docs = parse_external_docs(external_docs);
        tag_builder = tag_builder.external_docs(Some(external_docs));
    }
    tag_builder.build()
}

/// Parses the operation.
pub(super) fn parse_operation(
    tag: &str,
    path: &str,
    config: &Table,
    ignore_securities: bool,
) -> Operation {
    let mut operation_builder = OperationBuilder::new()
        .tag(tag)
        .response("default", Ref::from_response_name("default"))
        .response("error", Ref::from_response_name("4XX"));
    if let Some(responses) = config.get_table("responses") {
        for (key, value) in responses.iter() {
            if let Some(config) = value.as_table() {
                let name = key.to_case(Case::Camel);
                let response = parse_response(config);
                operation_builder = operation_builder.response(name, response);
            } else if let Some(response_name) = value.as_str() {
                let name = key.to_case(Case::Camel);
                let response_ref = Ref::from_response_name(response_name);
                operation_builder = operation_builder.response(name, response_ref);
            }
        }
    }
    if let Some(tags) = config.get_str_array("tags") {
        let tags = tags.into_iter().map(|s| s.to_owned()).collect::<Vec<_>>();
        operation_builder = operation_builder.tags(Some(tags));
    }
    if let Some(tag) = config.get_str("tag") {
        operation_builder = operation_builder.tag(tag);
    }
    if let Some(servers) = config.get_array("servers") {
        let servers = servers
            .iter()
            .filter_map(|v| v.as_table())
            .map(parse_server)
            .collect::<Vec<_>>();
        operation_builder = operation_builder.servers(Some(servers));
    }
    if let Some(server) = config.get_table("server") {
        operation_builder = operation_builder.server(parse_server(server));
    }
    if let Some(securities) = config.get_array("securities") {
        let security_requirements = securities
            .iter()
            .filter_map(|v| v.as_table())
            .map(parse_security_requirement)
            .collect::<Vec<_>>();
        operation_builder = operation_builder.securities(Some(security_requirements));
    } else if ignore_securities {
        operation_builder = operation_builder.securities(Some(Vec::new()));
    }
    if let Some(security) = config.get_table("security") {
        let security_requirement = parse_security_requirement(security);
        operation_builder = operation_builder.security(security_requirement);
    }
    if let Some(summary) = config.get_str("summary") {
        operation_builder = operation_builder.summary(Some(summary));
    }
    if let Some(description) = config.get_str("description") {
        operation_builder = operation_builder.description(Some(description));
    }
    if let Some(operation_id) = config.get_str("operation_id") {
        operation_builder = operation_builder.operation_id(Some(operation_id));
    }
    if let Some(deprecated) = config.get_bool("deprecated") {
        let deprecated = if deprecated {
            Deprecated::True
        } else {
            Deprecated::False
        };
        operation_builder = operation_builder.deprecated(Some(deprecated));
    }
    for parameter in parse_path_parameters(path).into_iter() {
        operation_builder = operation_builder.parameter(parameter);
    }
    if let Some(query) = config.get_table("query") {
        for parameter in parse_query_parameters(query).into_iter() {
            operation_builder = operation_builder.parameter(parameter);
        }
    }
    if let Some(headers) = config.get_table("headers") {
        for parameter in parse_header_parameters(headers).into_iter() {
            operation_builder = operation_builder.parameter(parameter);
        }
    }
    if let Some(cookies) = config.get_table("cookies") {
        for parameter in parse_cookie_parameters(cookies).into_iter() {
            operation_builder = operation_builder.parameter(parameter);
        }
    }
    if let Some(body) = config.get_table("body") {
        let request_body = parse_request_body(body);
        operation_builder = operation_builder.request_body(Some(request_body));
    }
    operation_builder.build()
}

/// Parses the response.
pub(super) fn parse_response(config: &Table) -> Response {
    let mut response_builder = ResponseBuilder::new();
    if let Some(description) = config.get_str("description") {
        response_builder = response_builder.description(description);
    }
    if let Some(content) = config.get_table("content") {
        let content_type = config.get_str("content_type").unwrap_or("application/json");
        let content_schema = if let Some(schema) = content.get_str("schema") {
            parse_schema_reference(schema)
        } else {
            parse_schema(content).into()
        };
        let mut content_builder = ContentBuilder::new().schema(Some(content_schema));
        if let Some(example) = config.get("example") {
            content_builder = content_builder.example(Some(example.to_json_value()));
        }
        response_builder = response_builder.content(content_type, content_builder.build());
    }
    if let Some(headers) = config.get_table("headers") {
        for (key, value) in headers.iter() {
            if let Some(config) = value.as_table() {
                let name = key.to_case(Case::Kebab);
                let mut header = Header::new(parse_schema(config));
                if let Some(description) = config.get_str("description") {
                    header.description = Some(description.to_owned());
                }
                response_builder = response_builder.header(name, header);
            } else if let Some(schema_type) = value.as_str() {
                let name = key.to_case(Case::Kebab);
                let schema = Object::with_type(parse_schema_type(schema_type));
                let header = Header::new(schema);
                response_builder = response_builder.header(name, header);
            }
        }
    }
    response_builder.build()
}

/// Parses the schema.
pub(super) fn parse_schema(config: &Table) -> Schema {
    const SPECIAL_KEYS: [&str; 3] = ["type", "items", "content_type"];

    let schema_type_name = config.get_str("type").unwrap_or("object");
    let mut is_array_object = false;
    if schema_type_name == "array" {
        if config.get_str("items") == Some("object") {
            is_array_object = true;
        } else {
            return Schema::Array(parse_array_schema(config));
        }
    }

    let schema_type = if is_array_object {
        Type::Object
    } else {
        parse_schema_type(schema_type_name)
    };
    let mut object_builder = ObjectBuilder::new().schema_type(schema_type);
    for (key, value) in config {
        if key == "default" {
            let default_value = value.to_json_value();
            object_builder = object_builder
                .default(Some(default_value.clone()))
                .examples(Some(default_value));
        } else if key == "example" {
            object_builder = object_builder.examples(Some(value.to_json_value()));
        } else {
            match value {
                TomlValue::String(value) => match key.as_str() {
                    "format" => {
                        let format = parse_schema_format(value);
                        object_builder = object_builder.format(Some(format));
                    }
                    "title" => {
                        object_builder = object_builder.title(Some(value));
                    }
                    "description" => {
                        object_builder = object_builder.description(Some(value));
                    }
                    "pattern" => {
                        object_builder = object_builder.pattern(Some(value));
                    }
                    _ => {
                        if !SPECIAL_KEYS.contains(&key.as_str()) {
                            let object = Object::with_type(parse_schema_type(value));
                            object_builder = object_builder.property(key, object);
                        }
                    }
                },
                TomlValue::Integer(value) => match key.as_str() {
                    "max_length" => {
                        object_builder = object_builder.max_length(usize::try_from(*value).ok());
                    }
                    "min_length" => {
                        object_builder = object_builder.min_length(usize::try_from(*value).ok());
                    }
                    "max_properties" => {
                        object_builder =
                            object_builder.max_properties(usize::try_from(*value).ok());
                    }
                    "min_properties" => {
                        object_builder =
                            object_builder.min_properties(usize::try_from(*value).ok());
                    }
                    _ => (),
                },
                TomlValue::Float(value) => match key.as_str() {
                    "multiple_of" => {
                        object_builder = object_builder.multiple_of(Some(*value));
                    }
                    "maximum" => {
                        object_builder = object_builder.maximum(Some(*value));
                    }
                    "minimum" => {
                        object_builder = object_builder.minimum(Some(*value));
                    }
                    "exclusive_maximum" => {
                        object_builder = object_builder.exclusive_maximum(Some(*value));
                    }
                    "exclusive_minimum" => {
                        object_builder = object_builder.exclusive_minimum(Some(*value));
                    }
                    _ => (),
                },
                TomlValue::Boolean(value) => match key.as_str() {
                    "write_only" => {
                        object_builder = object_builder.write_only(Some(*value));
                    }
                    "read_only" => {
                        object_builder = object_builder.read_only(Some(*value));
                    }
                    "deprecated" => {
                        let deprecated = if *value {
                            Deprecated::True
                        } else {
                            Deprecated::False
                        };
                        object_builder = object_builder.deprecated(Some(deprecated));
                    }
                    _ => (),
                },
                TomlValue::Array(vec) => match key.as_str() {
                    "required" => {
                        for field in vec.iter().filter_map(|v| v.as_str()) {
                            object_builder = object_builder.required(field);
                        }
                    }
                    "enum" => {
                        let values = vec.iter().filter_map(|v| v.as_str());
                        object_builder = object_builder.enum_values(Some(values));
                    }
                    "examples" => {
                        for example in vec.iter() {
                            object_builder = object_builder.examples(Some(example.to_json_value()));
                        }
                    }
                    _ => (),
                },
                TomlValue::Table(config) => {
                    let object = parse_schema(config);
                    object_builder = object_builder.property(key, object);
                }
                _ => (),
            }
        }
    }
    if is_array_object {
        Schema::Array(object_builder.to_array_builder().build())
    } else {
        Schema::Object(object_builder.build())
    }
}

/// Parses the Array schema.
fn parse_array_schema(config: &Table) -> Array {
    let item_schema = if let Some(items) = config.get_table("items") {
        if let Some(schema) = items.get_str("schema") {
            parse_schema_reference(schema)
        } else {
            parse_schema(items).into()
        }
    } else {
        let items = config.get_str("items").unwrap_or("string");
        Object::with_type(parse_schema_type(items)).into()
    };
    let mut array_builder = ArrayBuilder::new().items(item_schema);
    for (key, value) in config {
        if key == "default" {
            let default_value = value.to_json_value();
            array_builder = array_builder
                .default(Some(default_value.clone()))
                .examples(Some(default_value));
        } else if key == "example" {
            array_builder = array_builder.examples(Some(value.to_json_value()));
        } else {
            match value {
                TomlValue::String(value) => match key.as_str() {
                    "title" => {
                        array_builder = array_builder.title(Some(value));
                    }
                    "description" => {
                        array_builder = array_builder.description(Some(value));
                    }
                    _ => (),
                },
                TomlValue::Integer(value) => match key.as_str() {
                    "max_items" => {
                        array_builder = array_builder.max_items(usize::try_from(*value).ok());
                    }
                    "min_items" => {
                        array_builder = array_builder.min_items(usize::try_from(*value).ok());
                    }
                    _ => (),
                },
                TomlValue::Boolean(value) => match key.as_str() {
                    "unique_items" => {
                        array_builder = array_builder.unique_items(*value);
                    }
                    "deprecated" => {
                        let deprecated = if *value {
                            Deprecated::True
                        } else {
                            Deprecated::False
                        };
                        array_builder = array_builder.deprecated(Some(deprecated));
                    }
                    _ => (),
                },
                _ => (),
            }
        }
    }
    array_builder.build()
}

/// Parses the schema reference.
fn parse_schema_reference(schema: &str) -> RefOr<Schema> {
    let schema_ref = if schema.starts_with('/') || schema.contains(':') {
        Ref::new(schema)
    } else {
        let schema_name = schema.to_case(Case::Camel);
        Ref::from_schema_name(schema_name)
    };
    RefOr::Ref(schema_ref)
}

/// Parses the HTTP method.
pub(super) fn parse_http_method(method: &str) -> HttpMethod {
    match method {
        "POST" => HttpMethod::Post,
        "PUT" => HttpMethod::Put,
        "DELETE" => HttpMethod::Delete,
        "OPTIONS" => HttpMethod::Options,
        "HEAD" => HttpMethod::Head,
        "PATCH" => HttpMethod::Patch,
        "TRACE" => HttpMethod::Trace,
        _ => HttpMethod::Get,
    }
}

/// Parses the schema type.
fn parse_schema_type(basic_type: &str) -> Type {
    match basic_type {
        "boolean" => Type::Boolean,
        "integer" => Type::Integer,
        "number" => Type::Number,
        "string" => Type::String,
        "array" => Type::Array,
        "object" => Type::Object,
        _ => Type::Null,
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
        "time" => SchemaFormat::KnownFormat(KnownFormat::Time),
        "date" => SchemaFormat::KnownFormat(KnownFormat::Date),
        "date-time" => SchemaFormat::KnownFormat(KnownFormat::DateTime),
        "duration" => SchemaFormat::KnownFormat(KnownFormat::Duration),
        "password" => SchemaFormat::KnownFormat(KnownFormat::Password),
        "uuid" => SchemaFormat::KnownFormat(KnownFormat::Uuid),
        "ulid" => SchemaFormat::KnownFormat(KnownFormat::Ulid),
        "uri" => SchemaFormat::KnownFormat(KnownFormat::Uri),
        "uri-reference" => SchemaFormat::KnownFormat(KnownFormat::UriReference),
        "email" => SchemaFormat::KnownFormat(KnownFormat::Email),
        "idn-email" => SchemaFormat::KnownFormat(KnownFormat::IdnEmail),
        "hostname" => SchemaFormat::KnownFormat(KnownFormat::Hostname),
        "idn-hostname" => SchemaFormat::KnownFormat(KnownFormat::IdnHostname),
        "ipv4" => SchemaFormat::KnownFormat(KnownFormat::Ipv4),
        "ipv6" => SchemaFormat::KnownFormat(KnownFormat::Ipv6),
        "uri-template" => SchemaFormat::KnownFormat(KnownFormat::UriTemplate),
        "json-pointer" => SchemaFormat::KnownFormat(KnownFormat::JsonPointer),
        "relative-json-pointer" => SchemaFormat::KnownFormat(KnownFormat::RelativeJsonPointer),
        "regex" => SchemaFormat::KnownFormat(KnownFormat::Regex),
        _ => SchemaFormat::Custom(format.to_owned()),
    }
}

/// Parses the path parameters.
fn parse_path_parameters(path: &str) -> Vec<Parameter> {
    let mut parameters = Vec::new();
    for segment in path.split('/') {
        if let Some(name) = segment.strip_prefix('{').and_then(|s| s.strip_suffix('}')) {
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
fn parse_query_parameters(query: &Table) -> Vec<Parameter> {
    let mut parameters = Vec::new();
    for (key, value) in query {
        let mut parameter_builder = ParameterBuilder::new()
            .name(key.to_case(Case::Snake))
            .parameter_in(ParameterIn::Query);
        if let Some(config) = value.as_table() {
            let schema = if let Some(schema) = config.get_str("schema") {
                parse_schema_reference(schema)
            } else {
                parse_schema(config).into()
            };
            parameter_builder = parameter_builder
                .schema(Some(schema))
                .description(config.get_str("description"))
                .explode(config.get_bool("explode"))
                .allow_reserved(config.get_bool("allow_reserved"));
        } else if let Some(basic_type) = value.as_str() {
            let object = Object::with_type(parse_schema_type(basic_type));
            parameter_builder = parameter_builder.schema(Some(object));
        }
        parameters.push(parameter_builder.build());
    }
    parameters
}

/// Parses the header parameters.
fn parse_header_parameters(headers: &Table) -> Vec<Parameter> {
    let mut parameters = Vec::new();
    for (key, value) in headers {
        let mut parameter_builder = ParameterBuilder::new()
            .name(key.to_case(Case::Kebab))
            .parameter_in(ParameterIn::Header);
        if let Some(config) = value.as_table() {
            let schema = if let Some(schema) = config.get_str("schema") {
                parse_schema_reference(schema)
            } else {
                parse_schema(config).into()
            };
            parameter_builder = parameter_builder
                .schema(Some(schema))
                .description(config.get_str("description"))
                .explode(config.get_bool("explode"))
                .allow_reserved(config.get_bool("allow_reserved"));
        } else if let Some(basic_type) = value.as_str() {
            let object = Object::with_type(parse_schema_type(basic_type));
            parameter_builder = parameter_builder.schema(Some(object));
        }
        parameters.push(parameter_builder.build());
    }
    parameters
}

/// Parses the cookie parameters.
fn parse_cookie_parameters(cookies: &Table) -> Vec<Parameter> {
    let mut parameters = Vec::new();
    for (key, value) in cookies {
        let mut parameter_builder = ParameterBuilder::new()
            .name(key)
            .parameter_in(ParameterIn::Cookie);
        if let Some(config) = value.as_table() {
            let schema = if let Some(schema) = config.get_str("schema") {
                parse_schema_reference(schema)
            } else {
                parse_schema(config).into()
            };
            parameter_builder = parameter_builder
                .schema(Some(schema))
                .description(config.get_str("description"))
                .explode(config.get_bool("explode"))
                .allow_reserved(config.get_bool("allow_reserved"));
        } else if let Some(basic_type) = value.as_str() {
            let object = Object::with_type(parse_schema_type(basic_type));
            parameter_builder = parameter_builder.schema(Some(object));
        }
        parameters.push(parameter_builder.build());
    }
    parameters
}

/// Parses the request body.
fn parse_request_body(config: &Table) -> RequestBody {
    let schema = if let Some(schema) = config.get_str("schema") {
        parse_schema_reference(schema)
    } else {
        parse_schema(config).into()
    };
    let required = if config.get_bool("required") == Some(false) {
        Required::False
    } else {
        Required::True
    };
    let content_type = config.get_str("content_type").unwrap_or("application/json");
    RequestBodyBuilder::new()
        .description(config.get_str("description"))
        .required(Some(required))
        .content(content_type, Content::new(Some(schema)))
        .build()
}

/// Parses the security scheme.
pub(super) fn parse_security_scheme(config: &Table) -> SecurityScheme {
    let schema_type = config.get_str("type").unwrap_or("unkown");
    match schema_type {
        "oauth2" => {
            let mut flows = Vec::new();
            if let Some(config) = config.get_table("flows") {
                for (key, value) in config {
                    let Some(config) = value.as_table() else {
                        continue;
                    };
                    let scopes = if let Some(scopes) = config.get_table("scopes") {
                        let scopes_list = scopes
                            .iter()
                            .filter_map(|(key, value)| value.as_str().map(|s| (key.as_str(), s)));
                        Scopes::from_iter(scopes_list)
                    } else {
                        Scopes::new()
                    };
                    let flow = match key.as_str() {
                        "password" => {
                            let token_url = config.get_str("token_url").unwrap_or("/oauth2/token");
                            let flow = if let Some(refresh_url) = config.get_str("refresh_url") {
                                Password::with_refresh_url(token_url, scopes, refresh_url)
                            } else {
                                Password::new(token_url, scopes)
                            };
                            Flow::Password(flow)
                        }
                        "clientCredentials" => {
                            let token_url = config.get_str("token_url").unwrap_or("/oauth2/token");
                            let flow = if let Some(refresh_url) = config.get_str("refresh_url") {
                                ClientCredentials::with_refresh_url(token_url, scopes, refresh_url)
                            } else {
                                ClientCredentials::new(token_url, scopes)
                            };
                            Flow::ClientCredentials(flow)
                        }
                        "authorizationCode" => {
                            let authorization_url = config
                                .get_str("authorization_url")
                                .unwrap_or("/oauth2/auth");
                            let token_url = config.get_str("token_url").unwrap_or("/oauth2/token");
                            let flow = if let Some(refresh_url) = config.get_str("refresh_url") {
                                AuthorizationCode::with_refresh_url(
                                    authorization_url,
                                    token_url,
                                    scopes,
                                    refresh_url,
                                )
                            } else {
                                AuthorizationCode::new(authorization_url, token_url, scopes)
                            };
                            Flow::AuthorizationCode(flow)
                        }
                        _ => {
                            let authorization_url = config
                                .get_str("authorization_url")
                                .unwrap_or("/oauth2/auth");
                            let flow = if let Some(refresh_url) = config.get_str("refresh_url") {
                                Implicit::with_refresh_url(authorization_url, scopes, refresh_url)
                            } else {
                                Implicit::new(authorization_url, scopes)
                            };
                            Flow::Implicit(flow)
                        }
                    };
                    flows.push(flow);
                }
            }

            let oauth2 = if let Some(description) = config.get_str("description") {
                OAuth2::with_description(flows, description)
            } else {
                OAuth2::new(flows)
            };
            SecurityScheme::OAuth2(oauth2)
        }
        "apiKey" => {
            let name = config.get_str("name").unwrap_or("api_key");
            let location = config.get_str("in").unwrap_or("query");
            let api_key_value = if let Some(description) = config.get_str("description") {
                ApiKeyValue::with_description(name, description)
            } else {
                ApiKeyValue::new(name)
            };
            let api_key = match location {
                "header" => ApiKey::Header(api_key_value),
                "cookie" => ApiKey::Cookie(api_key_value),
                _ => ApiKey::Query(api_key_value),
            };
            SecurityScheme::ApiKey(api_key)
        }
        "http" => {
            let mut http_builder = HttpBuilder::new();
            if let Some(scheme) = config.get_str("scheme") {
                let http_auth_scheme = match scheme {
                    "bearer" => HttpAuthScheme::Bearer,
                    "digest" => HttpAuthScheme::Digest,
                    "hoba" => HttpAuthScheme::Hoba,
                    "mutual" => HttpAuthScheme::Mutual,
                    "negotiate" => HttpAuthScheme::Negotiate,
                    "oauth" => HttpAuthScheme::OAuth,
                    "scram-sha1" => HttpAuthScheme::ScramSha1,
                    "scram-sha256" => HttpAuthScheme::ScramSha256,
                    "vapid" => HttpAuthScheme::Vapid,
                    _ => HttpAuthScheme::Basic,
                };
                http_builder = http_builder.scheme(http_auth_scheme);
            }
            if let Some(bearer_format) = config.get_str("bearer_format") {
                http_builder = http_builder.bearer_format(bearer_format);
            }
            if let Some(description) = config.get_str("description") {
                http_builder = http_builder.description(Some(description.to_owned()));
            }
            SecurityScheme::Http(http_builder.build())
        }
        "openIdConnect" => {
            let open_id_connect_url = config.get_str("open_id_connect_url").unwrap_or("/openid");
            let open_id_connect = if let Some(description) = config.get_str("description") {
                OpenIdConnect::with_description(open_id_connect_url, description)
            } else {
                OpenIdConnect::new(open_id_connect_url)
            };
            SecurityScheme::OpenIdConnect(open_id_connect)
        }
        _ => SecurityScheme::MutualTls {
            description: config.get_str("description").map(|s| s.to_owned()),
            extensions: None,
        },
    }
}

/// Parses the security requirement.
pub(super) fn parse_security_requirement(config: &Table) -> SecurityRequirement {
    if let Some(name) = config.get_str("name") {
        let scopes = config.get_str_array("scopes").unwrap_or_default();
        SecurityRequirement::new(name, scopes)
    } else {
        SecurityRequirement::default()
    }
}

/// Parses the server.
pub(super) fn parse_server(config: &Table) -> Server {
    if let Some(url) = config.get_str("url") {
        let mut server = Server::new(url);
        if let Some(description) = config.get_str("description") {
            server.description = Some(description.to_owned());
        }
        if let Some(variables) = config.get_table("variables") {
            let mut server_variables = BTreeMap::new();
            for (name, value) in variables {
                let mut variable_builder = ServerVariableBuilder::new();
                match value {
                    TomlValue::String(s) => {
                        variable_builder = variable_builder.default_value(s);
                    }
                    TomlValue::Array(vec) => {
                        let enum_values = vec.iter().filter_map(|v| v.as_str());
                        variable_builder = variable_builder.enum_values(Some(enum_values));
                    }
                    TomlValue::Table(table) => {
                        if let Some(value) = table.get_str("default") {
                            variable_builder = variable_builder.default_value(value);
                        }
                        if let Some(description) = table.get_str("description") {
                            variable_builder = variable_builder.description(Some(description));
                        }
                        if let Some(values) = table.get_str_array("enum") {
                            variable_builder = variable_builder.enum_values(Some(values));
                        }
                    }
                    _ => (),
                }
                server_variables.insert(name.to_owned(), variable_builder.build());
            }
            server.variables = Some(server_variables);
        }
        server
    } else {
        Server::default()
    }
}

/// Parses the external docs.
pub(super) fn parse_external_docs(config: &Table) -> ExternalDocs {
    if let Some(url) = config.get_str("url") {
        let mut external_docs = ExternalDocs::new(url);
        if let Some(description) = config.get_str("description") {
            external_docs.description = Some(description.to_owned());
        }
        external_docs
    } else {
        ExternalDocs::default()
    }
}
