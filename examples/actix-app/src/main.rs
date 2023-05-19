#![feature(lazy_cell)]

mod controller;
mod extension;
mod middleware;
mod router;
mod schedule;
mod service;

use zino::Application;

fn main() {
    zino::Cluster::boot()
        .register(router::routes())
        .spawn(schedule::jobs())
        .run(schedule::async_jobs())
}
