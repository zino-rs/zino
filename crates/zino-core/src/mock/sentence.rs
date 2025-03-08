use rand::{
    Rng,
    distr::{Alphanumeric, SampleString},
};

#[cfg(feature = "locale")]
use random_word::Lang;

/// Generates a random sentence for the language.
pub(crate) fn gen_random_sentence(locale: &str, min_length: usize, max_length: usize) -> String {
    let mut rng = rand::rng();
    let mut length = rng.random_range(min_length..=max_length);
    let mut sentence = String::with_capacity(min_length);
    match locale {
        #[cfg(feature = "locale-en")]
        "en" | "en-US" => {
            while length > 0 {
                let word = random_word::get(Lang::En);
                let word_length = word.len();
                if let Some(remainder_length) = length.checked_sub(word_length) {
                    sentence.push_str(word);
                    sentence.push(' ');
                    length = remainder_length.saturating_sub(1);
                } else {
                    if sentence.len() <= min_length {
                        sentence.push_str(word);
                    }
                    break;
                }
            }
        }
        #[cfg(feature = "locale-es")]
        "es" | "es-ES" => {
            while length > 0 {
                let word = random_word::get(Lang::Es);
                let word_length = word.len();
                if let Some(remainder_length) = length.checked_sub(word_length) {
                    sentence.push_str(word);
                    sentence.push(' ');
                    length = remainder_length.saturating_sub(1);
                } else {
                    if sentence.len() <= min_length {
                        sentence.push_str(word);
                    }
                    break;
                }
            }
        }
        #[cfg(feature = "locale-de")]
        "de" | "de-DE" => {
            while length > 0 {
                let word = random_word::get(Lang::De);
                let word_length = word.len();
                if let Some(remainder_length) = length.checked_sub(word_length) {
                    sentence.push_str(word);
                    sentence.push(' ');
                    length = remainder_length.saturating_sub(1);
                } else {
                    if sentence.len() <= min_length {
                        sentence.push_str(word);
                    }
                    break;
                }
            }
        }
        #[cfg(feature = "locale-fr")]
        "fr" | "fr-FR" => {
            while length > 0 {
                let word = random_word::get(Lang::Fr);
                let word_length = word.len();
                if let Some(remainder_length) = length.checked_sub(word_length) {
                    sentence.push_str(word);
                    sentence.push(' ');
                    length = remainder_length.saturating_sub(1);
                } else {
                    if sentence.len() <= min_length {
                        sentence.push_str(word);
                    }
                    break;
                }
            }
        }
        #[cfg(feature = "locale-ru")]
        "ru" | "ru-RU" => {
            while length > 0 {
                let word = random_word::get(Lang::Ru);
                let word_length = word.len();
                if let Some(remainder_length) = length.checked_sub(word_length) {
                    sentence.push_str(word);
                    sentence.push(' ');
                    length = remainder_length.saturating_sub(1);
                } else {
                    if sentence.len() <= min_length {
                        sentence.push_str(word);
                    }
                    break;
                }
            }
        }
        #[cfg(feature = "locale-zh")]
        "zh" | "zh-CN" | "zh-CHS" => {
            while length > 0 {
                let mut word = random_word::get(Lang::Zh).trim();
                if let Some((_, hans)) = word.split_once(' ') {
                    word = hans;
                }

                let word_length = word.len();
                if let Some(remainder_length) = length.checked_sub(word_length) {
                    sentence.push_str(word);
                    length = remainder_length;
                } else {
                    if sentence.len() < min_length {
                        sentence.push_str(word);
                    }
                    break;
                }
            }
        }
        #[cfg(feature = "locale-zh")]
        "zh-HK" | "zh-TW" | "zh-CHT" => {
            while length > 0 {
                let mut word = random_word::get(Lang::Zh).trim();
                if let Some((hant, _)) = word.split_once(' ') {
                    word = hant;
                }

                let word_length = word.len();
                if let Some(remainder_length) = length.checked_sub(word_length) {
                    sentence.push_str(word);
                    length = remainder_length;
                } else {
                    if sentence.len() < min_length {
                        sentence.push_str(word);
                    }
                    break;
                }
            }
        }
        _ => {
            while length > 0 {
                let num_chars = rng.random_range(1..=16);
                let word = Alphanumeric.sample_string(&mut rng, num_chars);
                let word_length = word.len();
                if let Some(remainder_length) = length.checked_sub(word_length) {
                    sentence.push_str(&word);
                    sentence.push(' ');
                    length = remainder_length.saturating_sub(1);
                } else {
                    if sentence.len() <= min_length {
                        sentence.push_str(&word);
                    }
                    break;
                }
            }
        }
    }
    if sentence.ends_with(' ') {
        sentence.pop();
    }
    sentence
}
