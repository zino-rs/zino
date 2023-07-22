use super::ClientCredentials;
use crate::error::Error;

/// A server which provides authorization services.
pub trait AuthorizationProvider {
    /// Grants an access token for the client credentials.
    async fn grant_client_credentials(
        client_credentials: &mut ClientCredentials<Self>,
    ) -> Result<(), Error>;
}
