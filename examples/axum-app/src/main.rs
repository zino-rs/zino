mod controller;
mod domain;
mod extension;
mod logic;
mod middleware;
mod model;
mod router;
mod schedule;
mod service;

use zino::prelude::*;

fn main() {
    zino::Cluster::boot()
        .register(router::routes())
        .register_debug(router::debug_routes())
        .spawn(schedule::job_scheduler())
        .run_with(schedule::async_job_scheduler())
}
