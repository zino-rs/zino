use self::AppType::*;
use std::fmt;

/// Application type.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum AppType {
    #[default]
    /// A server application.
    Server,
    /// A desktop application.
    Desktop,
    /// A web application.
    Web,
    /// An agent.
    Agent,
}

impl AppType {
    /// Returns `true` if it is a server application.
    #[inline]
    pub fn is_server(&self) -> bool {
        matches!(self, Server)
    }

    /// Returns `true` if it is a desktop application.
    #[inline]
    pub fn is_desktop(&self) -> bool {
        matches!(self, Desktop)
    }

    /// Returns `true` if it is a web application.
    #[inline]
    pub fn is_web(&self) -> bool {
        matches!(self, Web)
    }

    /// Returns `true` if it is an agent.
    #[inline]
    pub fn is_agent(&self) -> bool {
        matches!(self, Agent)
    }

    /// Returns `self` as `&'static str`.
    #[inline]
    pub fn as_str(&self) -> &'static str {
        match self {
            Server => "server",
            Desktop => "desktop",
            Web => "web",
            Agent => "agent",
        }
    }
}

impl fmt::Display for AppType {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(self.as_str())
    }
}
