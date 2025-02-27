use rand::{
    Rng,
    distr::{Alphanumeric, SampleString},
    seq::IndexedRandom,
};

/// Subdomains for a mocked email address.
const SUBDOMAINS: [&str; 6] = [
    "box.mail",
    "example",
    "email",
    "mail",
    "mail-services",
    "mail.cyberspace",
];

/// Root domains for a mocked email address.
const ROOT_DOMAINS: [&str; 7] = ["app", "com", "dev", "edu", "gov", "net", "org"];

/// Generates a random email address.
pub(crate) fn gen_email() -> String {
    let mut rng = rand::rng();
    let num_chars = rng.random_range(1..=16);
    let username = Alphanumeric
        .sample_string(&mut rng, num_chars)
        .to_lowercase();
    let subdomain = SUBDOMAINS.choose(&mut rng).unwrap_or(&"example");
    let root_domain = ROOT_DOMAINS.choose(&mut rng).unwrap_or(&"com");
    format!("{username}@{subdomain}.{root_domain}")
}
