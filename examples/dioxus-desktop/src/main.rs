#![allow(non_snake_case)]

mod controller;
mod router;
mod schedule;
mod service;

use zino::prelude::*;

fn main() {
    zino::Desktop::init_dirs(&["local/uploads"])
        .register(router::Route::default())
        .spawn(schedule::jobs())
        .run(schedule::async_jobs())
}
