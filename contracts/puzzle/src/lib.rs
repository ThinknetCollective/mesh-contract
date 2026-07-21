use fluent_bundle::{FluentArgs, FluentResource, FluentBundle};
use unic_langid::LanguageIdentifier;
use std::collections::HashMap;

pub mod test;

/// Embedded locale resources
const LOCALES: &[(&str, &str)] = &[
    ("en", include_str!("../locales/en.ftl")),
    ("fr", include_str!("../locales/fr.ftl")),
    ("es", include_str!("../locales/es.ftl")),
];

pub struct I18nManager {
    active_locale: String,
    active_bundle: FluentBundle<FluentResource>,
    fallback_bundle: FluentBundle<FluentResource>,
    fallback_triggered: bool,
}

impl I18nManager {
    /// Initializes i18n manager with requested locale code (e.g. "fr", "es", "en").
    /// If requested locale is missing or invalid, falls back to "en".
    pub fn new(requested_locale: Option<&str>) -> Self {
        let locale_map: HashMap<&str, &str> = LOCALES.iter().cloned().collect();
        
        let fallback_lang: LanguageIdentifier = "en".parse().expect("Valid fallback langid");
        let mut fallback_bundle = FluentBundle::new(vec![fallback_lang]);
        if let Some(en_src) = locale_map.get("en") {
            let res = FluentResource::try_new(en_src.to_string())
                .expect("Failed to parse en.ftl resource");
            fallback_bundle.add_resource(res).expect("Failed to add en resource");
        }

        let requested = requested_locale.unwrap_or("en").to_lowercase();
        let target_code = if locale_map.contains_key(requested.as_str()) {
            requested.as_str()
        } else {
            "en"
        };

        let fallback_triggered = target_code != requested.as_str();

        let target_lang: LanguageIdentifier = target_code.parse().unwrap_or_else(|_| "en".parse().unwrap());
        let mut active_bundle = FluentBundle::new(vec![target_lang]);

        if let Some(src) = locale_map.get(target_code) {
            if let Ok(res) = FluentResource::try_new(src.to_string()) {
                let _ = active_bundle.add_resource(res);
            }
        }

        Self {
            active_locale: target_code.to_string(),
            active_bundle,
            fallback_bundle,
            fallback_triggered,
        }
    }

    /// Resolves localized string by key with optional arguments.
    /// Falls back to English if key is missing in active locale.
    pub fn get_message(&self, message_id: &str, args: Option<&FluentArgs>) -> String {
        let mut errors = vec![];

        // 1. Try active bundle
        if let Some(msg) = self.active_bundle.get_message(message_id) {
            if let Some(pattern) = msg.value() {
                return self.active_bundle.format_pattern(pattern, args, &mut errors).to_string();
            }
        }

        // 2. Try fallback (English) bundle
        if let Some(msg) = self.fallback_bundle.get_message(message_id) {
            if let Some(pattern) = msg.value() {
                return self.fallback_bundle.format_pattern(pattern, args, &mut errors).to_string();
            }
        }

        // 3. Fallback to key name if missing everywhere
        message_id.to_string()
    }

    /// Returns active locale code (e.g. "en", "fr", "es")
    pub fn active_locale(&self) -> &str {
        &self.active_locale
    }

    /// Returns true if fallback to English was triggered due to missing locale
    pub fn fallback_triggered(&self) -> bool {
        self.fallback_triggered
    }
}

/// Helper to parse `--lang <code>` from command-line arguments.
pub fn parse_lang_flag<I>(args: I) -> Option<String>
where
    I: Iterator<Item = String>,
{
    let mut iter = args.peekable();
    while let Some(arg) = iter.next() {
        if arg == "--lang" {
            return iter.next();
        } else if let Some(code) = arg.strip_prefix("--lang=") {
            return Some(code.to_string());
        }
    }
    None
}
