//! Search screen — interactive FTS5 search with live results.

use crossterm::event::{KeyCode, KeyModifiers};
use nexus_core::models::{SearchQuery, SearchResult};
use nexus_core::output::format_size;
use ratatui::prelude::*;
use ratatui::widgets::*;
use rusqlite::Connection;

/// State for the search screen.
#[derive(Default)]
pub struct SearchState {
    pub input: String,
    pub input_active: bool,
    pub results: Vec<SearchResult>,
    pub selected: usize,
    pub last_query: String,
    pub error: Option<String>,
}

/// Handle key events for the search screen.
pub fn handle_key(
    state: &mut SearchState,
    conn: &Connection,
    code: KeyCode,
    _modifiers: KeyModifiers,
) {
    if state.input_active {
        match code {
            KeyCode::Enter => {
                execute_search(state, conn);
                state.input_active = false;
            }
            KeyCode::Esc => {
                state.input_active = false;
            }
            KeyCode::Backspace => {
                state.input.pop();
            }
            KeyCode::Char(c) => {
                state.input.push(c);
            }
            _ => {}
        }
        return;
    }

    match code {
        KeyCode::Char('/') | KeyCode::Char('i') => {
            state.input_active = true;
        }
        KeyCode::Char('j') | KeyCode::Down => {
            if !state.results.is_empty() {
                state.selected = (state.selected + 1).min(state.results.len() - 1);
            }
        }
        KeyCode::Char('k') | KeyCode::Up => {
            state.selected = state.selected.saturating_sub(1);
        }
        _ => {}
    }
}

fn execute_search(state: &mut SearchState, conn: &Connection) {
    if state.input.trim().is_empty() {
        state.results.clear();
        return;
    }

    let query = SearchQuery {
        text: state.input.clone(),
        limit: 50,
        ..Default::default()
    };

    match nexus_discovery::search(conn, &query) {
        Ok(results) => {
            state.results = results;
            state.selected = 0;
            state.last_query = state.input.clone();
            state.error = None;
        }
        Err(e) => {
            state.error = Some(e.to_string());
            state.results.clear();
        }
    }
}

/// Draw the search screen.
pub fn draw(frame: &mut Frame, area: Rect, state: &SearchState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)])
        .split(area);

    // Search input
    let input_style = if state.input_active {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let cursor_char = if state.input_active { "▏" } else { "" };
    let input_text = if state.input.is_empty() && !state.input_active {
        "  Press / to search...".to_string()
    } else {
        format!("  {}{cursor_char}", state.input)
    };

    let input_widget =
        Paragraph::new(input_text)
            .style(input_style)
            .block(Block::bordered().title(" Search "));
    frame.render_widget(input_widget, chunks[0]);

    // Results
    if let Some(ref err) = state.error {
        let msg = Paragraph::new(format!("  Error: {err}"))
            .style(Style::default().fg(Color::Red))
            .block(Block::bordered().title(" Results "));
        frame.render_widget(msg, chunks[1]);
        return;
    }

    if state.results.is_empty() {
        let msg = if state.last_query.is_empty() {
            "  Type a query and press Enter to search indexed files."
        } else {
            "  No results found."
        };
        let paragraph = Paragraph::new(msg).block(Block::bordered().title(" Results "));
        frame.render_widget(paragraph, chunks[1]);
        return;
    }

    let title = format!(" Results ({}) ", state.results.len());

    let rows: Vec<Row> = state
        .results
        .iter()
        .enumerate()
        .map(|(i, result)| {
            let style = if i == state.selected {
                Style::default().bg(Color::DarkGray).fg(Color::White)
            } else {
                Style::default()
            };
            Row::new(vec![
                Cell::from(result.name.as_str()),
                Cell::from(result.category.as_str()),
                Cell::from(format_size(result.size)),
                Cell::from(
                    result
                        .path
                        .to_string_lossy()
                        .to_string(),
                ),
            ])
            .style(style)
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Length(25),
            Constraint::Length(10),
            Constraint::Length(10),
            Constraint::Min(30),
        ],
    )
    .header(
        Row::new(vec!["Name", "Category", "Size", "Path"])
            .style(Style::default().bold())
            .bottom_margin(1),
    )
    .block(Block::bordered().title(title));

    frame.render_widget(table, chunks[1]);
}
