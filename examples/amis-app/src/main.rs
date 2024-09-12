use phf::{phf_map, Map};
use zino_amis::prelude::*;

const ROUTES: Map<&str, &str> = phf_map! {
    "/" => "/index.html",
    "/404" => "/404.html",
};

fn main() {
    Amis::boot().register(ROUTES).run()
}
