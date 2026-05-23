mod en;
mod es;
mod pt;
pub mod strings;

use std::sync::OnceLock;
pub use strings::Strings;

// First call wins; subsequent calls are no-ops.
static STRINGS: OnceLock<&'static Strings> = OnceLock::new();

pub fn init(lang: &str) {
    let s: &'static Strings = match lang {
        "pt" => &pt::PT,
        "es" => &es::ES,
        _ => &en::EN,
    };
    STRINGS.set(s).ok();
}

pub fn t() -> &'static Strings {
    STRINGS.get().copied().unwrap_or(&en::EN)
}

/// Resolves language from LC_ALL > LC_MESSAGES > LANG (POSIX precedence).
pub fn detect_system_lang() -> &'static str {
    for var in ["LC_ALL", "LC_MESSAGES", "LANG"] {
        if let Ok(val) = std::env::var(var) {
            let val = val.to_lowercase();
            if val.starts_with("pt") { return "pt"; }
            if val.starts_with("es") { return "es"; }
            if val.starts_with("en") { return "en"; }
        }
    }
    "en"
}

/// Always uses the English table so the label is stable before `init` completes.
pub fn lang_label(lang: &str) -> &'static str {
    match lang {
        "pt" => en::EN.setup_option_pt,
        "es" => en::EN.setup_option_es,
        _ => en::EN.setup_option_en,
    }
}
