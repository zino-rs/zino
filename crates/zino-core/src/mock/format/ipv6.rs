use rand::Rng;
use std::net::Ipv6Addr;

/// Generates a random IPv6 address.
pub(crate) fn gen_ipv6() -> String {
    let mut rng = rand::rng();
    let a = rng.random::<u16>();
    let b = rng.random::<u16>();
    let c = rng.random::<u16>();
    let d = rng.random::<u16>();
    let e = rng.random::<u16>();
    let f = rng.random::<u16>();
    let g = rng.random::<u16>();
    let h = rng.random::<u16>();
    Ipv6Addr::new(a, b, c, d, e, f, g, h).to_string()
}
