//! Implementation of the Inertia protocol.

use crate::request::RequestContext;
use zino_core::{Map, bail, error::Error, extension::JsonObjectExt};

/// The Inertia page object for sharing data between the server and client.
#[derive(Debug, Clone, Default)]
pub struct InertiaPage {
    /// The name of the page component.
    component: String,
    /// The page props.
    props: Map,
    /// The page URL.
    url: String,
    /// The current asset version.
    version: String,
    /// The partial reload data.
    partial_data: Vec<String>,
    /// The redirect URL.
    redirect_url: Option<String>,
}

impl InertiaPage {
    /// Creates a new instance of the page object.
    #[inline]
    pub fn new(component: impl ToString) -> Self {
        Self {
            component: component.to_string(),
            props: Map::new(),
            url: String::new(),
            version: String::new(),
            partial_data: Vec::new(),
            redirect_url: None,
        }
    }

    /// Attempts to construct an instance for a partial reload request.
    pub fn partial_reload<Ctx: RequestContext>(ctx: &Ctx) -> Result<Self, Error> {
        if !ctx.get_header("x-inertia").is_some_and(|s| s == "true") {
            bail!("invalid `x-inertia` header");
        }

        let Some(component) = ctx.get_header("x-inertia-partial-component") else {
            bail!("invalid `x-inertia-partial-component` header");
        };
        let mut page = Self {
            component: component.to_owned(),
            props: Map::new(),
            url: ctx.request_path().to_owned(),
            version: String::new(),
            partial_data: Vec::new(),
            redirect_url: None,
        };
        if let Some(version) = ctx.get_header("x-inertia-version") {
            page.version = version.to_owned();
        }
        if let Some(data) = ctx.get_header("x-inertia-partial-data") {
            page.partial_data = data.split(',').map(|s| s.trim().to_owned()).collect();
        }
        Ok(page)
    }

    /// Provides the request context for the response.
    pub fn context<Ctx: RequestContext>(mut self, ctx: &Ctx) -> Self {
        self.url = ctx.request_path().to_owned();
        if let Some(version) = ctx.get_header("x-inertia-version") {
            self.version = version.to_owned();
        }
        if let Some(component) = ctx.get_header("x-inertia-partial-component") {
            self.component = component.to_owned();
        }
        if let Some(data) = ctx.get_header("x-inertia-partial-data") {
            self.partial_data = data.split(',').map(|s| s.trim().to_owned()).collect();
        }
        self
    }

    /// Appends the page props.
    #[inline]
    pub fn append_props(&mut self, props: &mut Map) {
        self.props.append(props);
    }

    /// Sets the page URL.
    #[inline]
    pub fn set_url(&mut self, url: String) {
        self.url = url;
    }

    /// Sets the current asset version.
    #[inline]
    pub fn set_version(&mut self, version: String) {
        self.version = version;
    }

    /// Sets the redirect URL.
    #[inline]
    pub fn set_redirect_url(&mut self, url: String) {
        self.redirect_url = Some(url);
    }

    /// Returns the name of the page component.
    #[inline]
    pub fn component(&self) -> &str {
        &self.component
    }

    /// Returns a reference to the page props.
    #[inline]
    pub fn props(&self) -> &Map {
        &self.props
    }

    /// Returns the page URL.
    #[inline]
    pub fn url(&self) -> &str {
        &self.url
    }

    /// Returns the current asset version.
    #[inline]
    pub fn version(&self) -> &str {
        &self.version
    }

    /// Returns a reference to the partial data.
    #[inline]
    pub fn partial_data(&self) -> &[String] {
        &self.partial_data
    }

    /// Returns a reference to the redirect URL.
    #[inline]
    pub fn redirect_url(&self) -> Option<&str> {
        self.redirect_url.as_deref()
    }

    /// Converts `self` into a JSON response.
    #[inline]
    pub fn into_json_response(self) -> Map {
        let mut map = Map::new();
        map.upsert("component", self.component);
        map.upsert("props", self.props);
        map.upsert("url", self.url);
        map.upsert("version", self.version);
        map
    }
}
