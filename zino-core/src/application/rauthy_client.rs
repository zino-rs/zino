use super::Application;
use crate::extension::TomlTableExt;
use rauthy_client::{
    oidc_config::{ClaimMapping, JwtClaim, JwtClaimTyp, RauthyConfig},
    provider::OidcProvider,
};
use std::collections::HashSet;

/// Initializes the Rauthy client and setups the OIDC provider.
pub(super) async fn setup<APP: Application + ?Sized>() {
    let Some(config) = APP::config().get_table("rauthy") else {
        tracing::warn!("`rauthy` config should be specified");
        return;
    };
    let Some(client_id) = config.get_str("client-id") else {
        tracing::warn!("`rauthy.client-id` should be specified");
        return;
    };
    let Some(redirect_uri) = config.get_str("redirect-uri") else {
        tracing::warn!("`rauthy.redirect-uri` should be specified");
        return;
    };
    let Some(issuer_uri) = config.get_str("issuer-uri") else {
        tracing::warn!("`rauthy.issuer-uri` should be specified");
        return;
    };
    let audiences = if let Some(audiences) = config.get_str_array("audiences") {
        HashSet::from_iter(audiences.into_iter().map(|s| s.to_owned()))
    } else {
        HashSet::from([client_id.to_owned()])
    };
    let group_claim = if let Some(groups) = config.get_str_array("groups") {
        let claims = groups.into_iter().map(|group| JwtClaim {
            typ: JwtClaimTyp::Groups,
            value: group.to_owned(),
        }).collect();
        ClaimMapping::Or(claims)
    } else {
        ClaimMapping::Any
    };
    let scopes = config
        .get_str_array("scopes")
        .unwrap_or_else(|| vec!["openid"]);
    let rauthy_config = RauthyConfig {
        admin_claim: ClaimMapping::Or(vec![JwtClaim {
            typ: JwtClaimTyp::Roles,
            value: "admin".to_owned(),
        }]),
        user_claim: group_claim,
        allowed_audiences: audiences,
        client_id: client_id.to_owned(),
        email_verified: config.get_bool("email-verified").unwrap_or_default(),
        iss: issuer_uri.to_owned(),
        scope: scopes.into_iter().map(|s| s.to_owned()).collect(),
        secret: config.get_str("secret").map(|s| s.to_owned()),
    };
    if let Err(err) = rauthy_client::init().await {
        tracing::error!("fail to initialize the Rauthy client: {err}");
    }
    if let Err(err) = OidcProvider::setup_from_config(rauthy_config, redirect_uri).await {
        tracing::error!("fail to setup the OIDC provider: {err}");
    }
}
