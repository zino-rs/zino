#![allow(non_snake_case)]

mod controller;
mod router;
mod schedule;

use zino::prelude::*;

fn main() {
    zino::DioxusDesktop::init_dirs(&["assets/uploads"])
        .register(router::Route::default())
        .spawn(schedule::jobs())
        .run(schedule::async_jobs())
}
