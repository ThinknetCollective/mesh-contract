use fluent_bundle::FluentArgs;
use mesh_puzzle::{parse_lang_flag, I18nManager};

fn main() {
    let lang_arg = parse_lang_flag(std::env::args());
    let requested_lang = lang_arg.as_deref();

    let i18n = I18nManager::new(requested_lang);

    if i18n.fallback_triggered() {
        if let Some(requested) = requested_lang {
            let mut args = FluentArgs::new();
            args.set("code", requested);
            println!("{}", i18n.get_message("error-locale-missing", Some(&args)));
        }
    }

    // Display localized title
    println!("=== {} ===", i18n.get_message("welcome-title", None));

    // Display localized status bar
    let mut status_args = FluentArgs::new();
    status_args.set("lang", i18n.active_locale());
    println!("{}", i18n.get_message("tui-status-bar", Some(&status_args)));

    // Display UI menu options
    println!("\n{}", i18n.get_message("tui-menu-play", None));
    println!("{}", i18n.get_message("tui-menu-quit", None));
    println!("{}", i18n.get_message("tui-menu-select-lang", None));

    // Display puzzle prompt and hints
    println!("\nPrompt: {}", i18n.get_message("puzzle-prompt", None));
    println!("Hint:   {}", i18n.get_message("hint-label", None));
}
