#![allow(non_snake_case)]

mod controller;
mod router;
mod schedule;
mod service;
mod view;

use zino::prelude::*;

fn main() {
    zino::Desktop::boot()
        .register(router::Route::default())
        .spawn(schedule::jobs())
        .run(schedule::async_jobs())
}
