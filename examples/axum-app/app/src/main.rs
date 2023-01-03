mod controller;
mod router;

/// Reexports.
use zino::{AxumCluster, Request};
use zino_core::{Application, Model, Query, Rejection, RequestContext, Response, Schema, Uuid};
use zino_model::User;

fn main() -> std::io::Result<()> {
    AxumCluster::new().register(router::init()).run()
}
