use crate::service::user;
use fluent::fluent_args;
use serde_json::json;
use std::time::Instant;
use zino::{
    ExtractRejection, JsonObjectExt, Map, Model, Request, RequestContext, Response, Result, Schema,
    Uuid,
};
use zino_model::User;

pub(crate) async fn new(mut req: Request) -> Result {
    let mut user = User::new();
    let mut res: Response = req.model_validation(&mut user).await?;

    user.upsert().await.extract(&req)?;
    let data = json!({
        "method": req.request_method().as_ref(),
        "path": req.request_path(),
    });
    res.set_data(&data);
    Ok(res.into())
}

pub(crate) async fn update(mut req: Request) -> Result {
    let user_id: Uuid = req.parse_param("id")?;
    let body: Map = req.parse_body().await?;
    let (validation, user_info) = user::update(user_id, body).await.extract(&req)?;
    let mut res = Response::from(validation).context(&req);
    let data = json!({
        "user": user_info,
    });
    res.set_data(&data);
    Ok(res.into())
}

pub(crate) async fn list(req: Request) -> Result {
    let mut query = User::default_query();
    let mut res: Response = req.query_validation(&mut query)?;
    let users: Vec<Map> = User::find(&query).await.extract(&req)?;
    let data = json!({
        "users": users,
    });
    res.set_data(&data);
    Ok(res.into())
}

pub(crate) async fn view(req: Request) -> Result {
    let user_id: Uuid = req.parse_param("id")?;
    let mut query = User::default_query();
    let mut res: Response = req.query_validation(&mut query)?;
    query.add_filter("id", user_id.to_string());

    let db_query_start_time = Instant::now();
    let user: Map = User::find_one(&query).await.extract(&req)?;
    let db_query_duration = db_query_start_time.elapsed();

    let args = fluent_args![
        "name" => user.get_str("name").unwrap_or_default()
    ];
    let user_intro = req.translate("user-intro", Some(args)).extract(&req)?;
    let data = json!({
        "columns": User::columns(),
        "intro": user_intro,
        "user": user,
    });
    res.record_server_timing("db", None, Some(db_query_duration));
    res.set_data(&data);
    Ok(res.into())
}
