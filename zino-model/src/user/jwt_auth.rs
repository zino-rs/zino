use std::{fmt::Display, str::FromStr};
use zino_core::{
    auth::JwtClaims,
    database::{ModelAccessor, ModelHelper},
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
    /// Password field name.
    const PASSWORD_FIELD: &'static str = "password";
    /// Role field name.
    const ROLE_FIELD: Option<&'static str> = Some("roles");
    /// Tenant ID field name.
    const TENANT_ID_FIELD: Option<&'static str> = None;

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
        query.allow_fields(&fields);
        query.add_filter("status", Map::from_entry("$in", vec!["Active", "Inactive"]));
        query.add_filter("account", account);

        let mut user: Map = Self::find_one(&query)
            .await?
            .ok_or_else(|| Error::new("404 Not Found: invalid user account or password"))?;
        let encrypted_password = user.get_str("password").unwrap_or_default();
        if Self::verify_password(passowrd, encrypted_password)? {
            let user_id = user.get_str("id").unwrap_or_default();
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
        query.add_filter("id", user_id);
        query.add_filter("status", Map::from_entry("$in", vec!["Active", "Inactive"]));

        let mut user: Map = Self::find_one(&query).await?.ok_or_else(|| {
            let message = format!("404 Not Found: the user `{user_id}` does not exist");
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
}

impl JwtAuthService for super::User {}
