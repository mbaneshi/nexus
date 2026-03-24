//! Changes screen — recent filesystem changes from the watcher daemon.

use crossterm::event::KeyCode;
use ratatui::prelude::*;
use ratatui::widgets::*;
use rusqlite::Connection;

/// A change row loaded from the database.
pub struct ChangeRow {
    pub path: String,
    pub change_type: String,
    pub detected_at: i64,
    pub new_size: Option<i64>,
}

/// State for the changes screen.
#[derive(Default)]
pub struct ChangesState {
    pub changes: Vec<ChangeRow>,
    pub selected: usize,
}

/// Load recent changes from the database.
pub fn refresh(conn: &Connection, state: &mut ChangesState) {
    let mut stmt = conn
        .prepare(
            "SELECT path, change_type, detected_at, new_size
             FROM file_changes
             ORDER BY detected_at DESC
             LIMIT 200",
        )
        .ok();

    state.changes = stmt
        .as_mut()
        .and_then(|s| {
            s.query_map([], |row: &rusqlite::Row<'_>| {
                Ok(ChangeRow {
                    path: row.get(0)?,
                    change_type: row.get(1)?,
                    detected_at: row.get(2)?,
                    new_size: row.get(3)?,
                })
            })
            .ok()
        })
        .map(|r| r.filter_map(|r| r.ok()).collect())
        .unwrap_or_default();

    if state.selected >= state.changes.len() && !state.changes.is_empty() {
        state.selected = state.changes.len() - 1;
    }
}

/// Handle key events for the changes screen.
pub fn handle_key(state: &mut ChangesState, code: KeyCode) {
    match code {
        KeyCode::Char('j') | KeyCode::Down => {
            if !state.changes.is_empty() {
                state.selected = (state.selected + 1).min(state.changes.len() - 1);
            }
        }
        KeyCode::Char('k') | KeyCode::Up => {
            state.selected = state.selected.saturating_sub(1);
        }
        _ => {}
    }
}

/// Draw the changes screen.
pub fn draw(frame: &mut Frame, area: Rect, state: &ChangesState) {
    if state.changes.is_empty() {
        let msg = Paragraph::new(
            "  No changes recorded yet.\n  Start the daemon with `nexus daemon start` to track filesystem changes.",
        )
        .block(Block::bordered().title(" Recent Changes "));
        frame.render_widget(msg, area);
        return;
    }

    let title = format!(" Recent Changes ({}) ", state.changes.len());

    let rows: Vec<Row> = state
        .changes
        .iter()
        .enumerate()
        .map(|(i, change)| {
            let style = if i == state.selected {
                Style::default().bg(Color::DarkGray).fg(Color::White)
            } else {
                Style::default()
            };

            let type_style = match change.change_type.as_str() {
                "created" => Style::default().fg(Color::Green),
                "modified" => Style::default().fg(Color::Yellow),
                "deleted" => Style::default().fg(Color::Red),
                _ => Style::default(),
            };

            let time = chrono::DateTime::from_timestamp(change.detected_at, 0)
                .map(|dt| dt.format("%H:%M:%S").to_string())
                .unwrap_or_else(|| "?".to_string());

            let size = change
                .new_size
                .map(|s| nexus_core::output::format_size(s as u64))
                .unwrap_or_else(|| "-".to_string());

            // Shorten path: strip home prefix
            let short_path = shorten_path(&change.path);

            Row::new(vec![
                Cell::from(time),
                Cell::from(change.change_type.as_str()).style(type_style),
                Cell::from(size),
                Cell::from(short_path),
            ])
            .style(style)
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Length(10),
            Constraint::Length(10),
            Constraint::Length(10),
            Constraint::Min(30),
        ],
    )
    .header(
        Row::new(vec!["Time", "Type", "Size", "Path"])
            .style(Style::default().bold())
            .bottom_margin(1),
    )
    .block(Block::bordered().title(title));

    frame.render_widget(table, area);
}

fn shorten_path(path: &str) -> String {
    if let Some(home) = dirs::home_dir() {
        let home_str = home.to_string_lossy();
        if let Some(rest) = path.strip_prefix(home_str.as_ref()) {
            return format!("~{rest}");
        }
    }
    path.to_string()
}
