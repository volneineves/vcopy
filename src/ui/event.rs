use super::app::{App, Mode};
use super::draw::draw;
use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use ratatui::{Terminal, backend::Backend};
use std::time::Duration;

pub fn loop_events<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> Result<()> {
    loop {
        terminal.draw(|f| draw(f, app))?;

        if event::poll(Duration::from_millis(200))? {
            if let Event::Key(key) = event::read()? {
                match &app.mode {
                    Mode::Normal => {
                        let prev = app.last_key;
                        app.last_key = None;
                        app.status_msg = None;

                        match key.code {
                            // ── quit ─────────────────────────────────────
                            KeyCode::Char('q') => break,
                            KeyCode::Char('c') if key.modifiers == KeyModifiers::CONTROL => break,

                            // ── navigation ───────────────────────────────
                            KeyCode::Up   | KeyCode::Char('k') => app.move_up(),
                            KeyCode::Down | KeyCode::Char('j') => app.move_down(),

                            KeyCode::Char('g') => {
                                if prev == Some('g') {
                                    app.move_top(); // gg
                                } else {
                                    app.last_key = Some('g');
                                }
                            }
                            KeyCode::Char('G') => app.move_bottom(),

                            KeyCode::Char('u') if key.modifiers == KeyModifiers::CONTROL => app.page_up(),
                            KeyCode::Char('d') if key.modifiers == KeyModifiers::CONTROL => app.page_down(),

                            // n/N: next/prev match (meaningful when search is active)
                            KeyCode::Char('n') => app.move_down(),
                            KeyCode::Char('N') => app.move_up(),

                            // ── actions ───────────────────────────────────
                            KeyCode::Enter | KeyCode::Char('y') => {
                                if app.copy_selected() {
                                    break; // copy-and-close, like a launcher
                                }
                            }
                            KeyCode::Char('e') => app.start_edit(),
                            KeyCode::Char('d') if prev != Some('d') => {
                                app.last_key = Some('d'); // wait for dd
                            }
                            KeyCode::Char('d') if prev == Some('d') => {
                                app.delete_selected(); // dd
                            }
                            KeyCode::Char('c') if prev != Some('c') => {
                                app.last_key = Some('c'); // wait for cc
                            }
                            KeyCode::Char('c') if prev == Some('c') => {
                                app.clear_history(); // cc
                            }

                            // ── refresh ───────────────────────────────────
                            KeyCode::Char('r') => {
                                app.reload();
                                app.status_msg = Some(crate::i18n::t().refreshed.into());
                            }

                            // ── search ────────────────────────────────────
                            KeyCode::Char('/') | KeyCode::Char('?') => app.start_search(),

                            _ => {}
                        }
                    }

                    Mode::Search => match key.code {
                        _ if key.modifiers == KeyModifiers::CONTROL => break,
                        KeyCode::Esc => app.cancel_search(),
                        KeyCode::Enter => {
                            app.mode = Mode::Normal;
                        }
                        KeyCode::Backspace => app.backspace_search(),
                        KeyCode::Char(c) => {
                            app.search_query.push(c);
                            app.selected = 0;
                        }
                        _ => {}
                    },

                    Mode::Edit => match key.code {
                        _ if key.modifiers == KeyModifiers::CONTROL => break,
                        KeyCode::Esc => {
                            app.mode = Mode::Normal;
                            app.edit_buffer.clear();
                        }
                        KeyCode::Enter => app.confirm_edit(),
                        KeyCode::Backspace => { app.edit_buffer.pop(); }
                        KeyCode::Char(c) => { app.edit_buffer.push(c); }
                        _ => {}
                    },
                }
            }
        }
    }
    Ok(())
}
