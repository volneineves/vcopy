use super::strings::Strings;

pub const PT: Strings = Strings {
    daemon_started: "VCopy daemon iniciado. Monitorando clipboard...",
    history_cleared: "Histórico limpo.",

    app_subtitle: "— histórico de clipboard",
    list_title: " Itens ",

    hint_copy: "copiar",
    hint_edit: "editar",
    hint_delete: "deletar",
    hint_search: "buscar",
    hint_quit: "sair",
    search_title: " Busca ",
    edit_title: " Editar — Enter salvar  Esc cancelar ",
    edit_prompt: "Editar: {}_",

    copied_prefix: "Copiado: ",
    copy_failed_prefix: "Falha ao copiar: ",
    item_deleted: "Item deletado.",
    item_edited: "Item editado.",

    setup_title: "primeiro uso",
    setup_detected: "Idioma detectado:",
    setup_prompt: "Escolha o idioma:\n  1) English\n  2) Português\n  3) Español\n[1/2/3, Enter = manter]: ",
    setup_option_en: "English",
    setup_option_pt: "Português",
    setup_option_es: "Español",
    setup_invalid: "Opção inválida. Mantendo idioma detectado.",
    setup_saved: "Idioma salvo:",

    lang_current: "Idioma atual:",
    lang_changed: "Idioma alterado para:",
    lang_unknown: "Idioma desconhecido. Opções: en, pt, es",

    install_prompt: "Iniciar o daemon automaticamente no login? [S/n]: ",
    install_yes: "Daemon instalado. Use 'vcopy status' para verificar.",
    install_skipped: "Ignorado. Execute 'vcopy install' quando quiser.",

    hint_refresh: "atualizar",
    refreshed: "Atualizado.",
};
