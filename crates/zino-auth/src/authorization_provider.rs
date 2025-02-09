use super::ClientCredentials;
use zino_core::error::Error;

/// A server which provides authorization services.
///
/// # Examples
///
/// ```rust,ignore
/// use zino_core::{
///     auth::{AuthorizationProvider, ClientCredentials},
///     connector::HttpConnector,
///     error::Error,
///     extension::JsonObjectExt,
///     json,
///     state::State,
///     LazyLock, Map,
/// };
///
/// #[derive(Debug, Clone, Copy)]
/// pub struct DingtalkAuth;
///
/// impl DingtalkAuth {
///     pub async fn get_token() -> Result<String, Error> {
///         DINGTALK_CREDENTIALS.request().await
///     }
/// }
///
/// impl AuthorizationProvider for DingtalkAuth {
///     async fn grant_client_credentials(
///         client_credentials: &ClientCredentials<Self>,
///     ) -> Result<(), Error> {
///         let params = client_credentials.to_request_params();
///         let data: Map = DINGTALK_TOKEN_CONNECTOR.fetch_json(None, Some(&params)).await?;
///         if let Some(access_token) = data.get_str("access_token") {
///             client_credentials.set_access_token(access_token);
///             client_credentials.set_expires(std::time::Duration::from_secs(6600));
///         }
///         Ok(())
///     }
/// }
///
/// static DINGTALK_CREDENTIALS: LazyLock<ClientCredentials<DingtalkAuth>> = LazyLock::new(|| {
///     let config = State::shared()
///         .get_config("dingtalk")
///         .expect("field `dingtalk` should be a table");
///     ClientCredentials::try_from_config(config)
///         .expect("fail to create the Dingtalk credentials")
/// });
///
/// static DINGTALK_TOKEN_CONNECTOR: LazyLock<HttpConnector> = LazyLock::new(|| {
///     let base_url = "https://oapi.dingtalk.com/gettoken";
///     connector = HttpConnector::try_new("GET", base_url)
///         .expect("fail to construct DingTalk token connector")
///         .query_param("appkey", Some("client_key"))
///         .query_param("appsecret", Some("client_secret"))
///         .build_query()
///         .expect("fail to build a query template for the connector")
/// });
/// ```
pub trait AuthorizationProvider {
    /// Grants an access token for the client credentials.
    async fn grant_client_credentials(
        client_credentials: &ClientCredentials<Self>,
    ) -> Result<(), Error>;
}
