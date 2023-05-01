mod controller;
mod middleware;
mod router;
mod schedule;
mod service;

use zino::Application;

fn main() {
    zino::ActixCluster::boot()
        .register(router::routes())
        .spawn(schedule::jobs())
        .run(schedule::async_jobs())
}
