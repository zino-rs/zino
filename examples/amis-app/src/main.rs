use zino_amis::prelude::*;

fn main() {
    Amis::boot()
        .register(phf_map! {
            "/" => "/index.html",
            "/404" => "/404.html",
        })
        .run()
}
