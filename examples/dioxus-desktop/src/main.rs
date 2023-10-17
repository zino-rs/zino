#![allow(non_snake_case)]
#![feature(let_chains)]

mod controller;
mod router;
mod schedule;
mod service;
mod view;

use router::Route;
use zino::{prelude::*, Desktop};

type App = Desktop<Route>;

fn main() {
    App::boot()
        .register(Route::default())
        .spawn(schedule::jobs())
        .run(schedule::async_jobs())
}
