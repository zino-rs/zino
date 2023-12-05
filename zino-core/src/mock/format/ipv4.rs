use rand::{thread_rng, Rng};
use std::net::Ipv4Addr;

/// Generates a random IPv4 address.
pub(crate) fn gen_ipv4() -> String {
    let mut rng = thread_rng();
    let a = rng.gen::<u8>();
    let b = rng.gen::<u8>();
    let c = rng.gen::<u8>();
    let d = rng.gen::<u8>();
    Ipv4Addr::new(a, b, c, d).to_string()
}
