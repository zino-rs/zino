use rand::{
    distributions::{Alphanumeric, DistString},
    random,
    seq::SliceRandom,
    thread_rng, Rng,
};

/// Schemes for a mocked URI.
const SCHEMES: [&str; 2] = ["http", "https"];

/// Subdomains for a mocked URI.
const SUBDOMAINS: [&str; 4] = ["example", "test", "www.example", "www.test"];

/// Root domains for a mocked URI.
const ROOT_DOMAINS: [&str; 3] = ["com", "net", "org"];

/// Generates a random URI.
pub(crate) fn gen_uri() -> String {
    let mut rng = thread_rng();
    let num_chars = rng.gen_range(1..=16);
    let mut path = Alphanumeric.sample_string(&mut rng, num_chars);
    if random::<bool>() {
        let num_chars = rng.gen_range(1..=16);
        let segment = Alphanumeric.sample_string(&mut rng, num_chars);
        path.push('/');
        path.push_str(&segment);
    }

    let scheme = SCHEMES.choose(&mut rng).unwrap_or(&"https");
    let subdomain = SUBDOMAINS.choose(&mut rng).unwrap_or(&"example");
    let root_domain = ROOT_DOMAINS.choose(&mut rng).unwrap_or(&"com");
    format!("{scheme}://{subdomain}.{root_domain}/{path}")
}
