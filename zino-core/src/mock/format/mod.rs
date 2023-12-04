use rand::{
    distributions::{Alphanumeric, DistString},
    thread_rng, Rng,
};

mod email;
mod phone_number;
mod uri;

/// Generates a random string with the format.
pub(crate) fn gen_format(format: &str, length: Option<usize>) -> String {
    let mut rng = thread_rng();
    match format {
        "email" => email::gen_email(),
        "phone-number" => phone_number::gen_phone_number(),
        "uri" => uri::gen_uri(),
        _ => {
            let length = length.unwrap_or_else(|| rng.gen_range(1..=32));
            Alphanumeric.sample_string(&mut rng, length)
        }
    }
}
