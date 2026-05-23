use super::strings::Strings;

pub const ES: Strings = Strings {
    daemon_started: "Daemon VCopy iniciado. Monitoreando portapapeles...",
    history_cleared: "Historial borrado.",

    app_subtitle: "— historial del portapapeles",
    list_title: " Elementos ",

    hint_copy: "copiar",
    hint_edit: "editar",
    hint_delete: "borrar",
    hint_search: "buscar",
    hint_quit: "salir",
    search_title: " Búsqueda ",
    edit_title: " Editar — Enter guardar  Esc cancelar ",
    edit_prompt: "Editar: {}_",

    copied_prefix: "Copiado: ",
    copy_failed_prefix: "Error al copiar: ",
    item_deleted: "Elemento borrado.",
    item_edited: "Elemento editado.",

    setup_title: "primer uso",
    setup_detected: "Idioma detectado:",
    setup_prompt: "Elija el idioma:\n  1) English\n  2) Português\n  3) Español\n[1/2/3, Enter = mantener]: ",
    setup_option_en: "English",
    setup_option_pt: "Português",
    setup_option_es: "Español",
    setup_invalid: "Opción inválida. Manteniendo idioma detectado.",
    setup_saved: "Idioma guardado:",

    lang_current: "Idioma actual:",
    lang_changed: "Idioma cambiado a:",
    lang_unknown: "Idioma desconocido. Opciones: en, pt, es",

    install_prompt: "¿Iniciar el daemon automáticamente al iniciar sesión? [S/n]: ",
    install_yes: "Daemon instalado. Use 'vcopy status' para verificar.",
    install_skipped: "Omitido. Ejecute 'vcopy install' cuando quiera.",

    hint_refresh: "actualizar",
    refreshed: "Actualizado.",
};
