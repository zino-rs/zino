#![feature(async_fn_in_trait)]
#![feature(lazy_cell)]
#![feature(let_chains)]

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
    zino::Cluster::init_dirs(&["local/uploads"])
        .register(router::routes())
        .spawn(schedule::jobs())
        .run(schedule::async_jobs())
}
