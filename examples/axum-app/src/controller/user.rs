use fluent::fluent_args;
use serde_json::json;
use std::time::Instant;
use zino::{
    ExtractRejection, JsonObjectExt, Map, Model, Query, Request, RequestContext, Response, Schema,
    Uuid,
};
use zino_model::User;

pub(crate) async fn new(mut req: Request) -> zino::Result {
    let mut user = User::new();
    let mut res: Response = req.model_validation(&mut user).await?;

    let rows = user.upsert().await.extract_with_context(&req)?;
    let data = json!({
        "method": req.request_method().as_ref(),
        "path": req.request_path(),
        "rows": rows,
    });
    res.set_data(&data);
    Ok(res.into())
}

pub(crate) async fn update(mut req: Request) -> zino::Result {
    let user_id: Uuid = req.parse_param("id")?;
    let mut user = User::try_get_model(&user_id.to_string())
        .await
        .extract_with_context(&req)?;
    let validation = req.parse_body().await.map(|body| user.read_map(body))?;
    user.update().await.extract_with_context(&req)?;

    let res = Response::from(validation).provide_context(&req);
    Ok(res.into())
}

pub(crate) async fn list(req: Request) -> zino::Result {
    let mut query = Query::new();
    let mut res: Response = req.query_validation(&mut query)?;
    let users: Vec<User> = User::find_as(&query).await.extract_with_context(&req)?;
    let data = json!({
        "users": users,
    });
    res.set_data(&data);
    Ok(res.into())
}

pub(crate) async fn view(mut req: Request) -> zino::Result {
    let locale_cookie = req.new_cookie("locale", "en-US", None);
    req.add_cookie(locale_cookie);

    let user_id: Uuid = req.parse_param("id")?;
    let mut query = Query::new();
    let mut res: Response = req.query_validation(&mut query)?;
    query.insert_filter("id", user_id.to_string());

    let message = json!({
        "path": req.request_path(),
    });
    let event = req.cloud_event("message", message);
    req.try_send(event)?;

    let db_query_start_time = Instant::now();
    let user: Map = User::find_one_as(&query).await.extract_with_context(&req)?;
    res.record_server_timing("db", None, Some(db_query_start_time.elapsed()));

    let args = fluent_args![
        "name" => user.get_str("name").unwrap_or_default()
    ];
    let user_intro = req
        .translate("user-intro", Some(args))
        .extract_with_context(&req)?;
    let data = json!({
        "schema": User::schema(),
        "intro": user_intro,
        "user": user,
    });
    res.set_data(&data);
    Ok(res.into())
}
