#![allow(non_snake_case)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod domain;
mod extension;
mod logic;
mod model;
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
        .spawn(schedule::job_scheduler())
        .run_with(schedule::async_job_scheduler())
}
