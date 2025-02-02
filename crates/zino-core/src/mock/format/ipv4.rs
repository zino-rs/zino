use rand::Rng;
use std::net::Ipv4Addr;

/// Generates a random IPv4 address.
pub(crate) fn gen_ipv4() -> String {
    let mut rng = rand::rng();
    let a = rng.random::<u8>();
    let b = rng.random::<u8>();
    let c = rng.random::<u8>();
    let d = rng.random::<u8>();
    Ipv4Addr::new(a, b, c, d).to_string()
}
