//! Internationalization and localization.

use crate::{
    LazyLock, SharedString,
    application::{Agent, Application},
    bail,
    error::Error,
    extension::TomlTableExt,
    state::State,
    warn,
};
use fluent::{FluentArgs, FluentResource, bundle::FluentBundle};
use intl_memoizer::concurrent::IntlLangMemoizer;
use std::{fs, io::ErrorKind};
use unic_langid::LanguageIdentifier;

/// A namespace for internationalization formatters.
#[derive(Debug, Clone, Copy)]
pub struct Intl;

impl Intl {
    /// Returns the default locale.
    #[inline]
    pub fn default_locale() -> &'static str {
        &DEFAULT_LOCALE
    }

    /// Selects a language from the supported locales.
    pub fn select_language(accepted_languages: &str) -> Option<&str> {
        let mut languages = accepted_languages
            .split(',')
            .filter_map(|s| {
                let (language, quality) = if let Some((language, quality)) = s.split_once(';') {
                    let quality = quality.trim().strip_prefix("q=")?.parse::<f32>().ok()?;
                    (language.trim(), quality)
                } else {
                    (s.trim(), 1.0)
                };
                SUPPORTED_LOCALES.iter().find_map(|&locale| {
                    (locale.eq_ignore_ascii_case(language) || locale.starts_with(language))
                        .then_some((locale, quality))
                })
            })
            .collect::<Vec<_>>();
        languages.sort_by(|a, b| b.1.total_cmp(&a.1));
        languages.first().map(|&(language, _)| language)
    }

    /// Translates the localization message.
    pub fn translate(
        locale: &LanguageIdentifier,
        message: &str,
        args: Option<FluentArgs<'_>>,
    ) -> Result<SharedString, Error> {
        let bundle = LOCALIZATION
            .iter()
            .find_map(|(lang_id, bundle)| (lang_id == locale).then_some(bundle))
            .or_else(|| {
                let lang = locale.language;
                LOCALIZATION
                    .iter()
                    .find_map(|(lang_id, bundle)| (lang_id.language == lang).then_some(bundle))
            })
            .or(*DEFAULT_BUNDLE)
            .ok_or_else(|| warn!("localization bundle does not exits"))?;
        let pattern = bundle
            .get_message(message)
            .ok_or_else(|| warn!("fail to get the localization message for `{}`", message))?
            .value()
            .ok_or_else(|| {
                warn!(
                    "fail to retrieve an option of the pattern for `{}`",
                    message
                )
            })?;

        let mut errors = vec![];
        if let Some(args) = args {
            let mut value = String::new();
            bundle.write_pattern(&mut value, pattern, Some(&args), &mut errors)?;
            if errors.is_empty() {
                Ok(value.into())
            } else {
                bail!("{:?}", errors);
            }
        } else {
            let value = bundle.format_pattern(pattern, None, &mut errors);
            if errors.is_empty() {
                Ok(value)
            } else {
                bail!("{:?}", errors);
            }
        }
    }
}

/// Translation type.
type Translation = FluentBundle<FluentResource, IntlLangMemoizer>;

/// Localization.
static LOCALIZATION: LazyLock<Vec<(LanguageIdentifier, Translation)>> = LazyLock::new(|| {
    let mut locales = Vec::new();
    let locale_dir = Agent::config_dir().join("locale");
    match fs::read_dir(locale_dir) {
        Ok(entries) => {
            let files = entries.filter_map(|entry| entry.ok());
            for file in files {
                let locale_file = file.path();
                let ftl_string = fs::read_to_string(&locale_file).unwrap_or_else(|err| {
                    let locale_file = locale_file.display();
                    panic!("fail to read `{locale_file}`: {err}");
                });
                let resource =
                    FluentResource::try_new(ftl_string).expect("fail to parse an FTL string");
                if let Some(locale) = file
                    .file_name()
                    .to_str()
                    .map(|s| s.trim_end_matches(".ftl"))
                {
                    let lang = locale
                        .parse::<LanguageIdentifier>()
                        .unwrap_or_else(|_| panic!("fail to language identifier `{locale}`"));

                    let mut bundle = FluentBundle::new_concurrent(vec![lang.clone()]);
                    bundle.set_use_isolating(false);
                    bundle
                        .add_resource(resource)
                        .expect("fail to add FTL resources to the bundle");
                    locales.push((lang, bundle));
                }
            }
        }
        Err(err) => {
            if err.kind() != ErrorKind::NotFound {
                tracing::error!("{err}");
            }
        }
    }
    locales
});

/// Default bundle.
static DEFAULT_BUNDLE: LazyLock<Option<&'static Translation>> = LazyLock::new(|| {
    let default_locale = LazyLock::force(&DEFAULT_LOCALE);
    LOCALIZATION
        .iter()
        .find_map(|(lang_id, bundle)| (lang_id == default_locale).then_some(bundle))
});

/// Default locale.
static DEFAULT_LOCALE: LazyLock<&'static str> = LazyLock::new(|| {
    if let Some(i18n) = State::shared().get_config("i18n") {
        i18n.get_str("default-locale").unwrap_or("en-US")
    } else {
        "en-US"
    }
});

/// Supported locales.
static SUPPORTED_LOCALES: LazyLock<Vec<&'static str>> = LazyLock::new(|| {
    LOCALIZATION
        .iter()
        .map(|(key, _)| {
            let language: &'static str = key.to_string().leak();
            language
        })
        .collect::<Vec<_>>()
});
