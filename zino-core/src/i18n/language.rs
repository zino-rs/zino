/// Selects a language from the supported locales.
pub fn select_language<'a>(
    accepted_languages: &'a str,
    supported_locales: &[&'a str],
) -> Option<&'a str> {
    let mut languages = accepted_languages
        .split(',')
        .filter_map(|s| {
            let (language, quality) = if let Some((language, quality)) = s.split_once(';') {
                let quality = quality.trim().strip_prefix("q=")?.parse::<f32>().ok()?;
                (language.trim(), quality)
            } else {
                (s.trim(), 1.0)
            };
            supported_locales.iter().find_map(|&locale| {
                (locale.eq_ignore_ascii_case(language) || locale.starts_with(language))
                    .then_some((locale, quality))
            })
        })
        .collect::<Vec<_>>();
    languages.sort_by(|a, b| b.1.total_cmp(&a.1));
    languages.first().map(|&(language, _)| language)
}

#[cfg(test)]
mod tests {
    use super::select_language;

    #[test]
    fn it_selects_language() {
        let languages = "zh-CN,zh;q=0.9,en;q=0.8,en-US;q=0.7";
        assert_eq!(
            select_language(languages, &["en-US", "zh-CN"]),
            Some("zh-CN"),
        );

        let languages = "zh-HK,zh;q=0.8,en-US; q=0.7";
        assert_eq!(
            select_language(languages, &["en-US", "zh-CN"]),
            Some("zh-CN"),
        );

        let languages = "zh-HK, zh;q=0.8,en-US; q=0.9";
        assert_eq!(
            select_language(languages, &["en-US", "zh-CN"]),
            Some("en-US"),
        );
    }
}
