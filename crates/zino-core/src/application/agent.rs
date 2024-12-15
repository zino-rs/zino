use super::Application;
use crate::schedule::AsyncScheduler;

#[cfg(feature = "http-client")]
use crate::{error::Error, Map};

/// An application agent with no routes.
#[derive(Debug, Clone, Copy)]
pub struct Agent;

impl Application for Agent {
    type Routes = ();

    #[inline]
    fn register(self, _routes: Self::Routes) -> Self {
        self
    }

    #[inline]
    fn run_with<T: AsyncScheduler + Send + 'static>(self, _scheduler: T) {}
}

impl Agent {
    /// Gets the shared HTTP client.
    #[cfg(feature = "http-client")]
    #[inline]
    pub fn get_http_client() -> Option<&'static reqwest::Client> {
        super::http_client::SHARED_HTTP_CLIENT.get()
    }

    /// Constructs a request builder.
    #[cfg(feature = "http-client")]
    #[inline]
    pub fn request_builder(
        url: &str,
        options: Option<&Map>,
    ) -> Result<reqwest_middleware::RequestBuilder, Error> {
        super::http_client::request_builder(url, options)
    }
}
