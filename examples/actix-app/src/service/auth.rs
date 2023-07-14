use serde_json::json;
use zino::prelude::*;
use zino_model::User;

pub async fn generate_token(body: Map) -> Result<(Uuid, String), Error> {
    let account = body
        .get_str("account")
        .ok_or_else(|| Error::new("403 Forbidden: the user `account` shoud be specified"))?;
    let passowrd = body
        .get_str("password")
        .ok_or_else(|| Error::new("403 Forbidden: the user `password` shoud be specified"))?;
    let mut query = Query::new(Map::new());
    query.allow_fields(&["id", "password", "roles"]);
    query.add_filter("status", json!({ "$nin": ["Locked", "Deleted"] }));
    query.add_filter("account", account);

    let user: Map = User::find_one(&query)
        .await?
        .ok_or_else(|| Error::new("404 Not Found: invalid user account or password"))?;
    let encrypted_password = user.get_str("password").unwrap_or_default();
    if User::verify_password(passowrd, encrypted_password)? {
        let user_id = user.get_str("id").unwrap_or_default();
        let mut claims = JwtClaims::new(user_id);
        claims.add_data_entry("roles", user.parse_str_array("roles"));

        let access_token = claims.sign_with(JwtClaims::shared_key())?;
        Ok((user_id.parse()?, access_token))
    } else {
        Err(Error::new("fail to generate access token"))
    }
}
