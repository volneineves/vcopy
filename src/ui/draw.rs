use super::app::{App, Mode, truncate};
use crate::i18n;
use chrono::Local;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    prelude::Alignment,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
};

const SEL_FG: Color = Color::Cyan;
const MATCH_BG: Color = Color::Yellow;
const MATCH_FG: Color = Color::Black;

pub fn draw(f: &mut Frame, app: &mut App) {
    let s = i18n::t();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(3),
        ])
        .split(f.area());

    app.page_size = (chunks[1].height as usize).saturating_sub(2) / 2;

    // Header
    let title = Paragraph::new(Line::from(vec![
        Span::styled("VCopy", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
        Span::raw(" "),
        Span::raw(s.app_subtitle),
    ]))
    .alignment(Alignment::Center)
    .block(Block::default().borders(Borders::ALL));
    f.render_widget(title, chunks[0]);

    // List — selection style applied per-span so match highlight survives.
    let query = app.search_query.clone();
    let selected = app.selected;
    let items = app.filtered_items();
    let max_content = chunks[1].width.saturating_sub(22) as usize;

    let list_items: Vec<ListItem> = items
        .iter()
        .enumerate()
        .map(|(idx, i)| {
            let is_sel = idx == selected;

            let prefix = if is_sel {
                Span::styled("➜ ", Style::default().fg(SEL_FG).add_modifier(Modifier::BOLD))
            } else {
                Span::raw("  ")
            };

            let ts_style = if is_sel {
                Style::default().fg(SEL_FG).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::DarkGray)
            };
            let ts = Span::styled(
                format!("{} ", i.copied_at.with_timezone(&Local).format("%d/%m %H:%M")),
                ts_style,
            );

            let content = truncate(&i.display_content(), max_content);
            let mut spans = vec![prefix, ts];
            spans.extend(highlight(&content, &query, is_sel));
            ListItem::new(Line::from(spans))
        })
        .collect();

    // No highlight_style — we handle it manually above.
    let mut state = ListState::default();
    state.select(Some(selected));

    let list = List::new(list_items).block(
        Block::default()
            .borders(Borders::ALL)
            .title(s.list_title)
            .title_alignment(Alignment::Center),
    );

    f.render_stateful_widget(list, chunks[1], &mut state);

    // Footer
    let footer = match &app.mode {
        Mode::Normal => {
            let content = if let Some(msg) = &app.status_msg {
                Line::from(Span::styled(msg.clone(), Style::default().fg(Color::Green)))
            } else {
                hint_bar(s)
            };
            Paragraph::new(content)
                .alignment(Alignment::Center)
                .block(Block::default().borders(Borders::ALL))
        }
        Mode::Search => {
            let line = Line::from(vec![
                Span::styled("/", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                Span::styled(app.search_query.clone(), Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
                Span::styled("█", Style::default().fg(Color::Cyan)),
            ]);
            Paragraph::new(line)
                .alignment(Alignment::Center)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title(s.search_title)
                        .title_alignment(Alignment::Center),
                )
        }
        Mode::Edit => {
            let prompt = s.edit_prompt.replace("{}", &app.edit_buffer);
            Paragraph::new(prompt)
                .alignment(Alignment::Center)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title(s.edit_title)
                        .title_alignment(Alignment::Center),
                )
        }
    };
    f.render_widget(footer, chunks[2]);
}

/// Splits `text` into spans with case-insensitive match highlighting.
/// Non-match text uses blue foreground when the row is selected, default otherwise.
fn highlight(text: &str, query: &str, is_selected: bool) -> Vec<Span<'static>> {
    let base = if is_selected {
        Style::default().fg(SEL_FG).add_modifier(Modifier::BOLD)
    } else {
        Style::default()
    };
    let matched = Style::default().bg(MATCH_BG).fg(MATCH_FG).add_modifier(Modifier::BOLD);

    if query.is_empty() {
        return vec![Span::styled(text.to_string(), base)];
    }

    let lower_text = text.to_lowercase();
    let lower_query = query.to_lowercase();
    let mut spans: Vec<Span<'static>> = Vec::new();
    let mut last = 0;

    while let Some(pos) = lower_text[last..].find(&lower_query) {
        let start = last + pos;
        let end = start + lower_query.len();

        if start > last {
            spans.push(Span::styled(text[last..start].to_string(), base));
        }
        spans.push(Span::styled(text[start..end].to_string(), matched));
        last = end;
    }

    if last < text.len() {
        spans.push(Span::styled(text[last..].to_string(), base));
    }
    spans
}

fn hint_bar(s: &crate::i18n::Strings) -> Line<'static> {
    fn key(k: &'static str) -> Span<'static> {
        Span::styled(k, Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
    }
    fn label(l: String) -> Span<'static> {
        Span::styled(l, Style::default().fg(Color::Gray))
    }
    fn sep() -> Span<'static> { Span::raw("  ") }

    Line::from(vec![
        key("↑↓/jk"), label(" nav".into()),                  sep(),
        key("Enter"),  label(format!(" {}", s.hint_copy)),    sep(),
        key("e"),      label(format!(" {}", s.hint_edit)),    sep(),
        key("dd"),     label(format!(" {}", s.hint_delete)),  sep(),
        key("/"),      label(format!(" {}", s.hint_search)),  sep(),
        key("gg"),     label(" top".into()),                  sep(),
        key("G"),      label(" bot".into()),                  sep(),
        key("r"),      label(format!(" {}", s.hint_refresh)), sep(),
        key("q"),      label(format!(" {}", s.hint_quit)),
    ])
}
