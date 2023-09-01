use std::{fmt::Display, str::FromStr};
use zino_core::{
    auth::JwtClaims,
    database::{ModelAccessor, ModelHelper},
    datetime::DateTime,
    error::Error,
    extension::JsonObjectExt,
    model::Query,
    Map, Uuid,
};

/// JWT authentication service.
pub trait JwtAuthService<K = Uuid>
where
    Self: ModelAccessor<K> + ModelHelper<K>,
    K: Default + Display + FromStr + PartialEq + serde::de::DeserializeOwned,
    <K as FromStr>::Err: std::error::Error,
{
    /// Account field name.
    const ACCOUNT_FIELD: &'static str = "account";
    /// Password field name.
    const PASSWORD_FIELD: &'static str = "password";
    /// Role field name.
    const ROLE_FIELD: Option<&'static str> = Some("roles");
    /// Tenant-ID field name.
    const TENANT_ID_FIELD: Option<&'static str> = None;
    /// Login-at field name.
    const LOGIN_AT_FIELD: Option<&'static str> = None;
    /// Login-IP field name.
    const LOGIN_IP_FIELD: Option<&'static str> = None;

    /// Returns the standard claims parsed from the `content` field.
    /// See [the spec](https://openid.net/specs/openid-connect-core-1_0.html#StandardClaims).
    fn standard_claims(&self) -> Map {
        let standard_fields = [
            "name",
            "given_name",
            "family_name",
            "middle_name",
            "nickname",
            "preferred_username",
            "profile",
            "picture",
            "website",
            "email",
            "email_verified",
            "gender",
            "birthdate",
            "zoneinfo",
            "locale",
            "phone_number",
            "phone_number_verified",
            "address",
        ];
        let address_fields = [
            "formatted",
            "street_address",
            "locality",
            "region",
            "postal_code",
            "country",
        ];
        let mut claims = Map::new();
        if let Some(map) = self.content() {
            for (key, value) in map {
                if key == "address" {
                    if let Some(map) = value.as_object() {
                        let mut address = Map::new();
                        for (key, value) in map {
                            if address_fields.contains(&key.as_str()) {
                                address.upsert(key, value.clone());
                            }
                        }
                        claims.upsert(key, address);
                    }
                } else if standard_fields.contains(&key.as_str()) {
                    claims.upsert(key, value.clone());
                }
            }
        }
        claims
    }

    /// Generates the access token and refresh token.
    async fn generate_token(body: Map) -> Result<(K, Map), Error> {
        let account = body
            .get_str("account")
            .ok_or_else(|| Error::new("403 Forbidden: the user `account` shoud be specified"))?;
        let passowrd = body
            .get_str("password")
            .ok_or_else(|| Error::new("403 Forbidden: the user `password` shoud be specified"))?;
        let mut query = Query::default();
        let mut fields = vec![Self::PRIMARY_KEY_NAME, Self::PASSWORD_FIELD];
        if let Some(role_field) = Self::ROLE_FIELD {
            fields.push(role_field);
        }
        if let Some(tenant_id_field) = Self::TENANT_ID_FIELD {
            fields.push(tenant_id_field);
        }
        if let Some(login_at_field) = Self::LOGIN_AT_FIELD {
            fields.push(login_at_field);
        }
        if let Some(login_ip_field) = Self::LOGIN_IP_FIELD {
            fields.push(login_ip_field);
        }
        query.allow_fields(&fields);
        query.add_filter("status", Map::from_entry("$nin", vec!["Locked", "Deleted"]));
        query.add_filter(Self::ACCOUNT_FIELD, account);

        let mut user: Map = Self::find_one(&query)
            .await?
            .ok_or_else(|| Error::new("404 Not Found: invalid user account or password"))?;
        let encrypted_password = user.get_str(Self::PASSWORD_FIELD).unwrap_or_default();
        if Self::verify_password(passowrd, encrypted_password)? {
            let user_id = user.get_str(Self::PRIMARY_KEY_NAME).unwrap_or_default();
            let mut claims = JwtClaims::new(user_id);

            let user_id = user_id.parse()?;
            if let Some(role_field) = Self::ROLE_FIELD && user.contains_key(role_field) {
                claims.add_data_entry("roles", user.parse_str_array(role_field));
            }
            if let Some(tenant_id_field) = Self::TENANT_ID_FIELD &&
                let Some(tenant_id) = user.remove(tenant_id_field)
            {
                claims.add_data_entry("tenant_id", tenant_id);
            }

            let mut data = Map::new();
            data.upsert("expires_in", claims.expires_in().as_secs());
            data.upsert("refresh_token", claims.refresh_token()?);
            data.upsert("access_token", claims.access_token()?);
            if let Some(login_at_field) = Self::LOGIN_AT_FIELD {
                data.upsert(login_at_field, user.remove(login_at_field));
            }
            if let Some(login_ip_field) = Self::LOGIN_IP_FIELD {
                data.upsert(login_ip_field, user.remove(login_ip_field));
            }
            Ok((user_id, data))
        } else {
            Err(Error::new("fail to generate access token"))
        }
    }

    /// Refreshes the access token.
    async fn refresh_token(claims: &JwtClaims) -> Result<Map, Error> {
        if !claims.data().is_empty() {
            return Err(Error::new("the JWT token is not a refresh token"));
        }

        let Some(user_id) = claims.subject() else {
            return Err(Error::new("the JWT token does not have a subject"));
        };

        let mut query = Query::default();
        let mut fields = vec![Self::PRIMARY_KEY_NAME];
        if let Some(role_field) = Self::ROLE_FIELD {
            fields.push(role_field);
        }
        if let Some(tenant_id_field) = Self::TENANT_ID_FIELD {
            fields.push(tenant_id_field);
        }
        query.allow_fields(&fields);
        query.add_filter(Self::PRIMARY_KEY_NAME, user_id);
        query.add_filter(
            "status",
            Map::from_entry("$nin", vec!["SignedOut", "Locked", "Deleted"]),
        );

        let mut user: Map = Self::find_one(&query).await?.ok_or_else(|| {
            let message = format!("403 Forbidden: cannot get the user `{user_id}`");
            Error::new(message)
        })?;
        let mut claims = JwtClaims::new(user_id);
        if let Some(role_field) = Self::ROLE_FIELD && user.contains_key(role_field) {
            claims.add_data_entry("roles", user.parse_str_array(role_field));
        }
        if let Some(tenant_id_field) = Self::TENANT_ID_FIELD &&
            let Some(tenant_id) = user.remove(tenant_id_field)
        {
            claims.add_data_entry("tenant_id", tenant_id);
        }

        let mut data = Map::new();
        data.upsert("expires_in", claims.expires_in().as_secs());
        data.upsert("access_token", claims.access_token()?);
        Ok(data)
    }

    /// Verfifies the JWT claims.
    async fn verify_jwt_claims(claims: &JwtClaims) -> Result<bool, Error> {
        let Some(user_id) = claims.subject() else {
            return Err(Error::new("the JWT token does not have a subject"));
        };

        let mut query = Query::default();
        let mut fields = vec![Self::PRIMARY_KEY_NAME];
        if let Some(role_field) = Self::ROLE_FIELD {
            fields.push(role_field);
        }
        if let Some(tenant_id_field) = Self::TENANT_ID_FIELD {
            fields.push(tenant_id_field);
        }
        if let Some(login_at_field) = Self::LOGIN_AT_FIELD {
            fields.push(login_at_field);
        }
        query.allow_fields(&fields);
        query.add_filter(Self::PRIMARY_KEY_NAME, user_id);
        query.add_filter(
            "status",
            Map::from_entry("$nin", vec!["SignedOut", "Locked", "Deleted"]),
        );

        let user: Map = Self::find_one(&query).await?.ok_or_else(|| {
            let message = format!("403 Forbidden: cannot get the user `{user_id}`");
            Error::new(message)
        })?;
        let data = claims.data();
        if let Some(role_field) = Self::ROLE_FIELD &&
            data.get("roles") != user.get(role_field)
        {
            let message = format!("403 Forbidden: invalid for the `{role_field}` field");
            return Err(Error::new(message));
        }
        if let Some(tenant_id_field) = Self::TENANT_ID_FIELD &&
            data.get("tenant_id") != user.get(tenant_id_field)
        {
            let message = format!("403 Forbidden: invalid for the `{tenant_id_field}` field");
            return Err(Error::new(message));
        }
        if let Some(login_at_field) = Self::LOGIN_AT_FIELD &&
            let Some(login_at_str) = user.get_str(login_at_field) &&
            let Ok(login_at) = login_at_str.parse::<DateTime>() &&
            claims.issued_at().timestamp() < login_at.timestamp()
        {
            let message = format!("403 Forbidden: invalid before the `{login_at_field}` time");
            return Err(Error::new(message));
        }
        Ok(true)
    }

    /// Verifies the user identity.
    async fn verify_identity(user_id: K, body: &Map) -> Result<Map, Error> {
        let mut query = Query::default();
        let mut fields = vec![Self::PRIMARY_KEY_NAME];
        let account = if let Some(account) = body.get_str("account") {
            fields.push(Self::ACCOUNT_FIELD);
            account
        } else {
            ""
        };
        let password = if let Some(passowrd) = body.get_str("password") {
            fields.push(Self::PASSWORD_FIELD);
            passowrd
        } else {
            ""
        };
        query.allow_fields(&fields);
        query.add_filter(Self::PRIMARY_KEY_NAME, user_id.to_string());
        query.add_filter("status", Map::from_entry("$nin", vec!["Locked", "Deleted"]));

        let user: Map = Self::find_one(&query).await?.ok_or_else(|| {
            let message = format!("403 Forbidden: cannot get the user `{user_id}`");
            Error::new(message)
        })?;

        let mut data = Map::new();
        if let Some(user_account) = user.get_str(Self::ACCOUNT_FIELD) {
            let account_verified = user_account == account;
            data.upsert("account_verified", account_verified);
        }
        if let Some(encrypted_password) = user.get_str(Self::PASSWORD_FIELD) {
            let password_verified = Self::verify_password(password, encrypted_password)?;
            data.upsert("password_verified", password_verified);
        }
        Ok(data)
    }
}

impl JwtAuthService for super::User {
    const LOGIN_AT_FIELD: Option<&'static str> = Some("current_login_at");
    const LOGIN_IP_FIELD: Option<&'static str> = Some("current_login_ip");
}
