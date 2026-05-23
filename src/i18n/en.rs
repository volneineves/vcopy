use super::strings::Strings;

pub const EN: Strings = Strings {
    daemon_started: "VCopy daemon started. Monitoring clipboard...",
    history_cleared: "History cleared.",

    app_subtitle: "— clipboard history",
    list_title: " Items ",

    hint_copy: "copy",
    hint_edit: "edit",
    hint_delete: "delete",
    hint_search: "search",
    hint_quit: "quit",
    search_title: " Search ",
    edit_title: " Edit — Enter save  Esc cancel ",
    edit_prompt: "Edit: {}_",

    copied_prefix: "Copied: ",
    copy_failed_prefix: "Copy failed: ",
    item_deleted: "Item deleted.",
    item_edited: "Item updated.",

    setup_title: "first use",
    setup_detected: "Detected language:",
    setup_prompt: "Choose language:\n  1) English\n  2) Português\n  3) Español\n[1/2/3, Enter = keep]: ",
    setup_option_en: "English",
    setup_option_pt: "Português",
    setup_option_es: "Español",
    setup_invalid: "Invalid option. Keeping detected language.",
    setup_saved: "Language saved:",

    lang_current: "Current language:",
    lang_changed: "Language changed to:",
    lang_unknown: "Unknown language. Options: en, pt, es",

    install_prompt: "Start the daemon automatically on login? [Y/n]: ",
    install_yes: "Daemon installed. Run 'vcopy status' to check.",
    install_skipped: "Skipped. Run 'vcopy install' whenever you want.",

    hint_refresh: "refresh",
    refreshed: "Refreshed.",
};
