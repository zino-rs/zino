use rand::{thread_rng, Rng};
use std::net::Ipv6Addr;

/// Generates a random IPv6 address.
pub(crate) fn gen_ipv6() -> String {
    let mut rng = thread_rng();
    let a = rng.gen::<u16>();
    let b = rng.gen::<u16>();
    let c = rng.gen::<u16>();
    let d = rng.gen::<u16>();
    let e = rng.gen::<u16>();
    let f = rng.gen::<u16>();
    let g = rng.gen::<u16>();
    let h = rng.gen::<u16>();
    Ipv6Addr::new(a, b, c, d, e, f, g, h).to_string()
}
