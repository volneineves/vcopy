/// All user-visible strings. Add a language by filling this struct in a new module.
pub struct Strings {
    pub daemon_started: &'static str,
    pub history_cleared: &'static str,

    pub app_subtitle: &'static str,
    pub list_title: &'static str,

    /// Labels for the footer hint bar — one per action, paired with a key in draw.rs.
    pub hint_copy: &'static str,
    pub hint_edit: &'static str,
    pub hint_delete: &'static str,
    pub hint_search: &'static str,
    pub hint_quit: &'static str,

    pub search_title: &'static str,
    pub edit_title: &'static str,
    /// Replace `{}` with the edit buffer: `s.edit_prompt.replace("{}", &buf)`
    pub edit_prompt: &'static str,

    pub copied_prefix: &'static str,
    pub copy_failed_prefix: &'static str,
    pub item_deleted: &'static str,
    pub item_edited: &'static str,

    pub setup_title: &'static str,
    pub setup_detected: &'static str,
    pub setup_prompt: &'static str,
    pub setup_option_en: &'static str,
    pub setup_option_pt: &'static str,
    pub setup_option_es: &'static str,
    pub setup_invalid: &'static str,
    pub setup_saved: &'static str,

    pub lang_current: &'static str,
    pub lang_changed: &'static str,
    pub lang_unknown: &'static str,

    pub install_prompt: &'static str,
    pub install_yes: &'static str,
    pub install_skipped: &'static str,

    pub hint_refresh: &'static str,
    pub refreshed: &'static str,
}
