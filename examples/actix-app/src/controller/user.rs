use fluent::fluent_args;
use serde_json::json;
use std::time::Instant;
use zino::{prelude::*, Request, Response, Result};
use zino_model::user::User;

pub async fn new(mut req: Request) -> Result {
    let mut user = User::new();
    let mut res = req.model_validation(&mut user).await?;
    let validation = user.check_constraints().await.extract(&req)?;
    if !validation.is_success() {
        return Err(Rejection::bad_request(validation).context(&req).into());
    }

    let user_name = user.name().to_owned();
    user.insert().await.extract(&req)?;

    let args = fluent_args![
        "name" => user_name
    ];
    let user_intro = req.translate("user-intro", Some(args)).extract(&req)?;
    let data = json!({
        "method": req.request_method().as_ref(),
        "path": req.request_path(),
        "user_intro": user_intro,
    });
    res.set_code(StatusCode::CREATED);
    res.set_data(&data);
    Ok(res.into())
}

pub async fn view(req: Request) -> Result {
    let user_id = req.parse_param::<Uuid>("id")?;

    let db_query_start_time = Instant::now();
    let user = User::fetch_by_id(&user_id).await.extract(&req)?;
    let db_query_duration = db_query_start_time.elapsed();

    let data = Map::data_entry(user);
    let mut res = Response::default().context(&req);
    res.record_server_timing("db", None, Some(db_query_duration));
    res.set_data(&data);
    Ok(res.into())
}
