use std::{fmt::Display, str::FromStr};
use zino_auth::JwtClaims;
use zino_core::{
    Map, Uuid, bail,
    datetime::DateTime,
    error::Error,
    extension::{JsonObjectExt, JsonValueExt},
    model::Query,
    warn,
};
use zino_orm::{ModelAccessor, ModelHelper};

/// JWT authentication service.
pub trait JwtAuthService<K = Uuid>
where
    Self: ModelAccessor<K> + ModelHelper<K>,
    K: Default + Display + FromStr + PartialEq + serde::de::DeserializeOwned,
    <K as FromStr>::Err: std::error::Error + Send + 'static,
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

    /// Consumes the user into standard claims without a `sub` field,
    /// which can be used to create a [`JwtClaims`] and generate an ID token.
    /// See [the spec](https://openid.net/specs/openid-connect-core-1_0.html#StandardClaims).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use zino_auth::JwtClaims;
    /// use zino_core::model::Model;
    /// use zino_model::user::{JwtAuthService, User};
    /// use zino_orm::ModelAccessor;
    ///
    /// let user = User::new();
    /// let subject = user.id().to_string();
    /// let custom_data = user.into_standard_claims();
    /// let claims = JwtClaims::with_data(subject, custom_data);
    /// let id_token = claims.sign_with(JwtClaims::shared_key());
    /// ```
    fn into_standard_claims(self) -> Map {
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
        claims.upsert("updated_at", self.updated_at().timestamp());
        if let Some(map) = self.extra() {
            for (key, value) in map {
                if key == "address" {
                    if let Some(map) = value.as_object() {
                        let mut address = Map::new();
                        for (key, value) in map {
                            if address_fields.contains(&key.as_str()) {
                                address.upsert(key, value.to_owned());
                            }
                        }
                        claims.upsert(key, address);
                    }
                } else if standard_fields.contains(&key.as_str()) {
                    claims.upsert(key, value.to_owned());
                }
            }
        }
        for (key, value) in self.into_map() {
            if key == "address" {
                if let Some(map) = value.into_map_opt() {
                    let mut address = Map::new();
                    for (key, value) in map {
                        if address_fields.contains(&key.as_str()) {
                            address.upsert(key, value);
                        }
                    }
                    claims.upsert(key, address);
                }
            } else if standard_fields.contains(&key.as_str()) {
                claims.upsert(key, value);
            }
        }
        claims
    }

    /// Generates the access token and refresh token.
    async fn generate_token(body: Map) -> Result<(K, Map), Error> {
        let account = body
            .get_str("account")
            .ok_or_else(|| warn!("401 Unauthorized: user `account` should be specified"))?;
        let passowrd = body
            .get_str("password")
            .ok_or_else(|| warn!("401 Unauthorized: user `password` should be specified"))?;
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
            .ok_or_else(|| warn!("404 Not Found: invalid user account or password"))?;
        let encrypted_password = user
            .get_str(Self::PASSWORD_FIELD)
            .ok_or_else(|| warn!("404 Not Found: user password is absent"))?;
        if Self::verify_password(passowrd, encrypted_password)
            .map_err(|_| warn!("401 Unauthorized: invalid user account or password"))?
        {
            // Cann't use `get_str` because the primary key may be an integer
            let user_id = user
                .parse_string(Self::PRIMARY_KEY_NAME)
                .ok_or_else(|| warn!("404 Not Found: user id is absent"))?;
            let mut claims = JwtClaims::new(user_id.as_ref());

            let user_id = user_id.parse()?;
            if let Some(role_field) = Self::ROLE_FIELD.filter(|&field| user.contains_key(field)) {
                claims.add_data_entry("roles", user.parse_str_array(role_field));
            }
            if let Some(tenant_id_field) = Self::TENANT_ID_FIELD {
                if let Some(tenant_id) = user.remove(tenant_id_field) {
                    claims.add_data_entry("tenant_id", tenant_id);
                }
            }

            let refresh_token = claims.refresh_token()?;
            let mut data = claims.bearer_auth()?;
            data.upsert("refresh_token", refresh_token);
            if let Some(login_at_field) = Self::LOGIN_AT_FIELD {
                data.upsert(login_at_field, user.remove(login_at_field));
            }
            if let Some(login_ip_field) = Self::LOGIN_IP_FIELD {
                data.upsert(login_ip_field, user.remove(login_ip_field));
            }
            Ok((user_id, data))
        } else {
            Err(warn!("fail to generate access token"))
        }
    }

    /// Refreshes the access token.
    async fn refresh_token(claims: &JwtClaims) -> Result<Map, Error> {
        if !claims.data().is_empty() {
            bail!("401 Unauthorized: JWT token is not a refresh token");
        }

        let Some(user_id) = claims.subject() else {
            bail!("401 Unauthorized: JWT token does not have a subject");
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

        let mut user: Map = Self::find_one(&query)
            .await?
            .ok_or_else(|| warn!("404 Not Found: cannot get the user `{}`", user_id))?;
        let mut claims = JwtClaims::new(user_id);
        if let Some(role_field) = Self::ROLE_FIELD.filter(|&field| user.contains_key(field)) {
            claims.add_data_entry("roles", user.parse_str_array(role_field));
        }
        if let Some(tenant_id_field) = Self::TENANT_ID_FIELD {
            if let Some(tenant_id) = user.remove(tenant_id_field) {
                claims.add_data_entry("tenant_id", tenant_id);
            }
        }
        claims.bearer_auth()
    }

    /// Verfifies the JWT claims.
    async fn verify_jwt_claims(claims: &JwtClaims) -> Result<bool, Error> {
        let Some(user_id) = claims.subject() else {
            bail!("401 Unauthorized: JWT token does not have a subject");
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

        let user: Map = Self::find_one(&query)
            .await?
            .ok_or_else(|| warn!("404 Not Found: cannot get the user `{}`", user_id))?;
        let data = claims.data();
        if let Some(role_field) = Self::ROLE_FIELD {
            if let Some(roles) = data.get("roles") {
                if user.get(role_field) != Some(roles) {
                    bail!("401 Unauthorized: invalid for the `{}` field", role_field);
                }
            }
        }
        if let Some(tenant_id_field) = Self::TENANT_ID_FIELD {
            if let Some(tenant_id) = data.get("tenant_id") {
                if user.get(tenant_id_field) != Some(tenant_id) {
                    bail!(
                        "401 Unauthorized: invalid for the `{}` field",
                        tenant_id_field
                    );
                }
            }
        }
        if let Some(login_at_field) = Self::LOGIN_AT_FIELD {
            if let Some(login_at_str) = user.get_str(login_at_field) {
                if let Ok(login_at) = login_at_str.parse::<DateTime>() {
                    if claims.issued_at().timestamp() < login_at.timestamp() {
                        bail!(
                            "401 Unauthorized: invalid before the `{}` time",
                            login_at_field
                        );
                    }
                }
            }
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

        let user: Map = Self::find_one(&query)
            .await?
            .ok_or_else(|| warn!("404 Not Found: cannot get the user `{}`", user_id))?;

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

impl JwtAuthService<Uuid> for super::User {
    const LOGIN_AT_FIELD: Option<&'static str> = Some("current_login_at");
    const LOGIN_IP_FIELD: Option<&'static str> = Some("current_login_ip");
}
