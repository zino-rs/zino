//! Internationalization and localization.

use crate::{
    LazyLock, SharedString,
    application::{Agent, Application},
    extension::TomlTableExt,
    state::State,
};
use fluent::{FluentArgs, FluentError, FluentResource, bundle::FluentBundle};
use intl_memoizer::concurrent::IntlLangMemoizer;
use std::{fmt, fs, io::ErrorKind};
use unic_langid::{LanguageIdentifier, LanguageIdentifierError};

/// An error which can be returned when fomrating localization messages.
#[derive(Debug)]
pub enum IntlError {
    /// An error for no localization bundle.
    NoBundle,
    /// An error for no localization message.
    NoMessage(String),
    /// An error for no localization message attribute.
    NoMessageAttribute(String, String),
    /// An error which can occur while formatting a message.
    Format(fmt::Error),
    /// Errors for Fluent runtime system.
    Fluent(Vec<FluentError>),
}

impl fmt::Display for IntlError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            IntlError::NoBundle => write!(f, "localization bundle does not exits"),
            IntlError::NoMessage(message) => write!(f, "no localization message `{message}`"),
            IntlError::NoMessageAttribute(message, attr) => {
                write!(f, "no localization message attribute `{message}.{attr}`")
            }
            IntlError::Format(err) => write!(f, "format error: {err}"),
            IntlError::Fluent(_errors) => write!(f, "errors occurred for Fluent runtime system"),
        }
    }
}

impl std::error::Error for IntlError {}

/// A namespace for internationalization formatters.
#[derive(Debug, Clone, Copy)]
pub struct Intl;

impl Intl {
    /// Returns the default locale, which can also be specified by
    /// the environment variable `ZINO_APP_LOCALE`.
    #[inline]
    pub fn default_locale() -> &'static LanguageIdentifier {
        &DEFAULT_LOCALE
    }

    /// Parses the locale as a `LanguageIdentifier` with no script and variants.
    #[inline]
    pub fn parse_locale(locale: &str) -> Result<LanguageIdentifier, LanguageIdentifierError> {
        let mut langid = locale.parse::<LanguageIdentifier>()?;
        langid.script = None;
        langid.clear_variants();
        Ok(langid)
    }

    /// Returns `true` if the locale is supported.
    #[inline]
    pub fn supports(locale: &LanguageIdentifier) -> bool {
        SUPPORTED_LOCALES.iter().any(|&lang| locale == lang)
    }

    /// Selects a language from the supported locales.
    pub fn select_language(accepted_languages: &str) -> Option<LanguageIdentifier> {
        let mut languages = accepted_languages
            .split(',')
            .filter_map(|s| {
                let (locale, quality) = if let Some((locale, quality)) = s.split_once(';') {
                    let quality = quality.trim().strip_prefix("q=")?.parse::<f32>().ok()?;
                    (locale.trim(), quality)
                } else {
                    (s.trim(), 1.0)
                };
                SUPPORTED_LOCALES.iter().find_map(|&lang| {
                    Self::parse_locale(locale)
                        .ok()
                        .filter(|langid| langid == lang)
                        .map(|langid| (langid, quality))
                })
            })
            .collect::<Vec<_>>();
        languages.sort_by(|a, b| b.1.total_cmp(&a.1));
        if languages.is_empty() {
            None
        } else {
            Some(languages.swap_remove(0).0)
        }
    }

    /// Translates the localization message with the default locale.
    #[inline]
    pub fn translate(
        message: &str,
        args: Option<FluentArgs<'_>>,
    ) -> Result<SharedString, IntlError> {
        Self::translate_with(message, args, Self::default_locale())
    }

    /// Translates the localization message with a specific locale.
    pub fn translate_with(
        message: &str,
        args: Option<FluentArgs<'_>>,
        locale: &LanguageIdentifier,
    ) -> Result<SharedString, IntlError> {
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
            .ok_or_else(|| IntlError::NoBundle)?;
        let pattern = if let Some((message, attr)) = message.split_once('.') {
            bundle
                .get_message(message)
                .and_then(|m| m.get_attribute(attr))
                .ok_or_else(|| IntlError::NoMessageAttribute(message.to_owned(), attr.to_owned()))?
                .value()
        } else {
            bundle
                .get_message(message)
                .and_then(|m| m.value())
                .ok_or_else(|| IntlError::NoMessage(message.to_owned()))?
        };

        let mut errors = vec![];
        if let Some(args) = args {
            let mut value = String::new();
            bundle
                .write_pattern(&mut value, pattern, Some(&args), &mut errors)
                .map_err(IntlError::Format)?;
            if errors.is_empty() {
                Ok(value.into())
            } else {
                Err(IntlError::Fluent(errors))
            }
        } else {
            let value = bundle.format_pattern(pattern, None, &mut errors);
            if errors.is_empty() {
                Ok(value)
            } else {
                Err(IntlError::Fluent(errors))
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
static DEFAULT_LOCALE: LazyLock<LanguageIdentifier> = LazyLock::new(|| {
    if let Ok(locale) = std::env::var("ZINO_APP_LOCALE") {
        return locale
            .parse()
            .expect("invalid environment variable for the default application locale");
    }
    let locale = if let Some(config) = State::shared().get_config("i18n") {
        config.get_str("default-locale").unwrap_or("en-US")
    } else {
        "en-US"
    };
    let mut langid = locale
        .parse::<LanguageIdentifier>()
        .expect("invalid value for the default locale");
    langid.script = None;
    langid.clear_variants();
    langid
});

/// Supported locales.
static SUPPORTED_LOCALES: LazyLock<Vec<&'static LanguageIdentifier>> = LazyLock::new(|| {
    LOCALIZATION
        .iter()
        .map(|(langid, _)| langid)
        .collect::<Vec<_>>()
});
