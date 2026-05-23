use crate::config::Config;
use crate::i18n;
use crate::clipboard::copy_item_to_clipboard;
use crate::storage::{ClipItem, ClipKind, Storage};
use anyhow::Result;

pub enum Mode {
    Normal,
    Search,
    Edit,
}

pub struct App {
    pub storage: Storage,
    pub items: Vec<ClipItem>,
    pub selected: usize,
    pub mode: Mode,
    pub search_query: String,
    pub edit_buffer: String,
    pub status_msg: Option<String>,
    pub last_key: Option<char>,
    pub page_size: usize,
    history_limit: usize,
}

impl App {
    pub fn new(storage: Storage) -> Result<Self> {
        let history_limit = Config::load()?.history.effective_limit();
        let items = storage.list(history_limit)?;
        Ok(Self {
            storage,
            items,
            selected: 0,
            mode: Mode::Normal,
            search_query: String::new(),
            edit_buffer: String::new(),
            status_msg: None,
            last_key: None,
            page_size: 10,
            history_limit,
        })
    }

    pub fn reload(&mut self) {
        self.items = self.storage.list(self.history_limit).unwrap_or_default();
        if self.selected >= self.items.len() && !self.items.is_empty() {
            self.selected = self.items.len() - 1;
        }
    }

    pub fn filtered_items(&self) -> Vec<&ClipItem> {
        if self.search_query.is_empty() {
            return self.items.iter().collect();
        }
        let q = self.search_query.to_lowercase();
        self.items
            .iter()
            .filter(|i| i.display_content().to_lowercase().contains(&q))
            .collect()
    }

    pub fn selected_item(&self) -> Option<&ClipItem> {
        self.filtered_items().get(self.selected).copied()
    }

    pub fn copy_selected(&mut self) -> bool {
        let Some(item) = self.selected_item() else {
            return false;
        };

        let item = item.clone();
        match copy_item_to_clipboard(&self.storage, &item) {
            Ok(()) => {
                let content = item.display_content();
                self.status_msg = Some(format!(
                    "{}{}",
                    i18n::t().copied_prefix,
                    truncate(&content, 40)
                ));
                true
            }
            Err(err) => {
                self.status_msg = Some(format!("{}{}", i18n::t().copy_failed_prefix, err));
                false
            }
        }
    }

    pub fn delete_selected(&mut self) {
        if let Some(item) = self.selected_item() {
            let id = item.id;
            let _ = self.storage.delete(id);
            self.reload();
            self.status_msg = Some(i18n::t().item_deleted.into());
        }
    }

    pub fn clear_history(&mut self) {
        let _ = self.storage.clear();
        self.reload();
        self.selected = 0;
        self.status_msg = Some(i18n::t().history_cleared.into());
    }

    pub fn start_edit(&mut self) {
        if let Some(item) = self.selected_item() {
            if item.kind == ClipKind::Text {
                self.edit_buffer = item.content.clone();
                self.mode = Mode::Edit;
            }
        }
    }

    pub fn start_search(&mut self) {
        self.mode = Mode::Search;
        self.search_query.clear();
        self.selected = 0;
    }

    pub fn cancel_search(&mut self) {
        self.mode = Mode::Normal;
        self.search_query.clear();
        self.selected = 0;
    }

    pub fn backspace_search(&mut self) {
        if self.search_query.is_empty() {
            self.cancel_search();
        } else {
            self.search_query.pop();
            self.selected = 0;
        }
    }

    pub fn confirm_edit(&mut self) {
        if let Some(item) = self.selected_item() {
            let id = item.id;
            let new = self.edit_buffer.clone();
            let _ = self.storage.update_text(id, &new);
            self.mode = Mode::Normal;
            self.reload();
            self.status_msg = Some(i18n::t().item_edited.into());
        }
    }

    pub fn move_up(&mut self) {
        if self.selected > 0 { self.selected -= 1; }
    }

    pub fn move_down(&mut self) {
        let max = self.filtered_items().len().saturating_sub(1);
        if self.selected < max { self.selected += 1; }
    }

    pub fn move_top(&mut self) { self.selected = 0; }

    pub fn move_bottom(&mut self) {
        self.selected = self.filtered_items().len().saturating_sub(1);
    }

    pub fn page_up(&mut self) {
        self.selected = self.selected.saturating_sub(self.page_size);
    }

    pub fn page_down(&mut self) {
        let max = self.filtered_items().len().saturating_sub(1);
        self.selected = (self.selected + self.page_size).min(max);
    }
}

pub fn truncate(s: &str, max: usize) -> String {
    let s = s.replace('\n', " ");
    if s.len() > max { format!("{}…", &s[..max]) } else { s }
}
