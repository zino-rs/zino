use serde_json::json;
use zino::prelude::*;
use zino_model::User;

pub async fn generate_token(body: Map) -> Result<(Uuid, String, String), Error> {
    let account = body
        .get_str("account")
        .ok_or_else(|| Error::new("403 Forbidden: the user `account` shoud be specified"))?;
    let passowrd = body
        .get_str("password")
        .ok_or_else(|| Error::new("403 Forbidden: the user `password` shoud be specified"))?;
    let mut query = Query::default();
    query.allow_fields(&["id", "password", "roles"]);
    query.add_filter("status", json!({ "$nin": ["Locked", "Deleted"] }));
    query.add_filter("account", account);

    let user: Map = User::find_one(&mut query)
        .await?
        .ok_or_else(|| Error::new("404 Not Found: invalid user account or password"))?;
    let encrypted_password = user.get_str("password").unwrap_or_default();
    if User::verify_password(passowrd, encrypted_password)? {
        let user_id = user.get_str("id").unwrap_or_default();
        let mut claims = JwtClaims::new(user_id);
        claims.add_data_entry("roles", user.parse_str_array("roles"));

        let user_id = user_id.parse()?;
        let refresh_token = claims.refresh_token()?;
        let access_token = claims.access_token()?;
        Ok((user_id, access_token, refresh_token))
    } else {
        Err(Error::new("fail to generate access token"))
    }
}

pub async fn refresh_token(claims: &JwtClaims) -> Result<String, Error> {
    if !claims.data().is_empty() {
        return Err(Error::new("the JWT token is not a refresh token"));
    }

    let Some(user_id) = claims.subject() else {
        return Err(Error::new("the JWT token does not have a subject"));
    };

    let mut query = Query::default();
    query.allow_fields(&["id", "roles"]);
    query.add_filter("id", user_id);
    query.add_filter("status", json!({ "$nin": ["Locked", "Deleted"] }));

    let user: Map = User::find_one(&mut query).await?.ok_or_else(|| {
        let message = format!("404 Not Found: the user `{user_id}` does not exist");
        Error::new(message)
    })?;
    let mut claims = JwtClaims::new(user_id);
    claims.add_data_entry("roles", user.parse_str_array("roles"));
    claims.access_token()
}
