mod controller;
mod router;
mod schedule;

use zino::Application;

fn main() {
    zino::AxumCluster::boot()
        .register(router::routes())
        .spawn(schedule::jobs())
        .run(schedule::async_jobs())
}
