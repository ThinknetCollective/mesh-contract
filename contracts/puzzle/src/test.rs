#[cfg(test)]
mod tests {
    use crate::{parse_lang_flag, I18nManager};
    use fluent_bundle::FluentArgs;

    #[test]
    fn test_default_english_locale() {
        let i18n = I18nManager::new(None);
        assert_eq!(i18n.active_locale(), "en");
        assert!(!i18n.fallback_triggered());

        let title = i18n.get_message("welcome-title", None);
        assert_eq!(title, "Welcome to Mesh Puzzle Challenge!");
    }

    #[test]
    fn test_french_locale_selection() {
        let i18n = I18nManager::new(Some("fr"));
        assert_eq!(i18n.active_locale(), "fr");
        assert!(!i18n.fallback_triggered());

        let title = i18n.get_message("welcome-title", None);
        assert_eq!(title, "Bienvenue au Défi Mesh Puzzle !");

        let prompt = i18n.get_message("puzzle-prompt", None);
        assert!(prompt.contains("Résolvez le puzzle"));
    }

    #[test]
    fn test_spanish_locale_selection() {
        let i18n = I18nManager::new(Some("es"));
        assert_eq!(i18n.active_locale(), "es");
        assert!(!i18n.fallback_triggered());

        let title = i18n.get_message("welcome-title", None);
        assert_eq!(title, "¡Bienvenido al Desafío Mesh Puzzle!");
    }

    #[test]
    fn test_missing_locale_fallback_to_english() {
        let i18n = I18nManager::new(Some("de")); // German not present
        assert_eq!(i18n.active_locale(), "en");
        assert!(i18n.fallback_triggered());

        let title = i18n.get_message("welcome-title", None);
        assert_eq!(title, "Welcome to Mesh Puzzle Challenge!");
    }

    #[test]
    fn test_missing_key_fallback() {
        let i18n = I18nManager::new(Some("fr"));
        let missing = i18n.get_message("non-existent-key", None);
        assert_eq!(missing, "non-existent-key");
    }

    #[test]
    fn test_parameter_formatting() {
        let i18n = I18nManager::new(Some("en"));
        let mut args = FluentArgs::new();
        args.set("lang", "en");
        let status = i18n.get_message("tui-status-bar", Some(&args));
        assert!(status.contains("Locale: en"));
    }

    #[test]
    fn test_cli_flag_parsing() {
        let args = vec!["puzzle".to_string(), "--lang".to_string(), "fr".to_string()];
        let parsed = parse_lang_flag(args.into_iter());
        assert_eq!(parsed, Some("fr".to_string()));

        let args_eq = vec!["puzzle".to_string(), "--lang=es".to_string()];
        let parsed_eq = parse_lang_flag(args_eq.into_iter());
        assert_eq!(parsed_eq, Some("es".to_string()));

        let args_none = vec!["puzzle".to_string()];
        let parsed_none = parse_lang_flag(args_none.into_iter());
        assert_eq!(parsed_none, None);
    }
}
