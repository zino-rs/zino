use fluent::fluent_args;
use serde_json::{json, Value};
use std::time::{Duration, Instant};
use zino::{
    Error, JsonObjectExt, Map, Model, Query, Request, RequestContext, Schema, Uuid,
    Validation,
};
use zino_model::{ModelAccessor, User};

pub(crate) async fn update(user_id: Uuid, body: Map) -> Result<(Validation, Value), Error> {
    let user_id = user_id.to_string();
    let mut user = User::try_get_model(&user_id).await?;
    let validation = user.read_map(&body);
    if !validation.is_success() {
        return Ok((validation, Value::Null));
    }

    let query = user.current_version_query();
    let mutation = user.next_version_mutation(body);
    User::update_one(&query, &mutation).await?;

    let data = json!({
        "user": user.next_version_filters(),
    });
    Ok((validation, data))
}

pub(crate) async fn view(req: &Request, query: &Query) -> Result<(Duration, Value), Error> {
    let db_query_start_time = Instant::now();
    let user: Map = User::find_one(query)
        .await?
        .ok_or_else(|| Error::new("user does not exist"))?;
    let db_query_duration = db_query_start_time.elapsed();

    let args = fluent_args![
        "name" => user.get_str("name").unwrap_or_default()
    ];
    let user_intro = req.translate("user-intro", Some(args))?;
    let data = json!({
        "schema": User::schema(),
        "intro": user_intro,
        "user": user,
    });
    Ok((db_query_duration, data))
}
