mod controller;
mod router;
mod schedule;

use zino_core::Application;

fn main() {
    zino::AxumCluster::new()
        .register(router::init_routes())
        .spawn(schedule::init_jobs())
        .run(schedule::init_async_jobs())
}
