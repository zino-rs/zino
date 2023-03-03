use fluent::fluent_args;
use serde_json::{json, Value};
use std::time::{Duration, Instant};
use zino::{BoxError, JsonObjectExt, Map, Query, Request, RequestContext, Schema};
use zino_model::User;

pub(crate) async fn view_profile(
    req: &Request,
    query: &Query,
) -> Result<(Duration, Value), BoxError> {
    let db_query_start_time = Instant::now();
    let user: Map = User::find_one(&query).await?.ok_or("user does not exist")?;
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
