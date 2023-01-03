use std::{io, time::Instant};

/// Application.
pub trait Application {
    /// Router.
    type Router;

    /// Creates a new application.
    fn new() -> Self;

    /// Registers the router.
    fn register(self, routes: Self::Router) -> Self;

    /// Returns the start time.
    fn start_time(&self) -> Instant;

    /// Runs the application.
    fn run(self) -> io::Result<()>;
}
