#![feature(async_fn_in_trait)]
#![feature(lazy_cell)]

mod controller;
mod extension;
mod middleware;
mod model;
mod router;
mod schedule;
mod service;

use zino::prelude::*;

fn main() {
    zino::Cluster::boot()
        .register(router::routes())
        .spawn(schedule::jobs())
        .run(schedule::async_jobs())
}
