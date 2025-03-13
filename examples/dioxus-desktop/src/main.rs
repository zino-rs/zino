#![allow(non_snake_case)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod domain;
mod extension;
mod logic;
mod model;
mod router;
mod service;
mod view;

use router::Route;
use zino::{Desktop, prelude::*};

type App = Desktop<Route>;

fn main() {
    App::boot().register(Route::default()).run()
}
