//! Overview screen — home directory statistics and category breakdown.

use nexus_core::models::{CategoryStats, HomeStats};
use nexus_core::output::format_size;
use ratatui::prelude::*;
use ratatui::widgets::*;
use rusqlite::Connection;

/// State for the overview screen.
#[derive(Default)]
pub struct OverviewState {
    pub stats: Option<HomeStats>,
}

/// Load overview data from the database.
pub fn refresh(conn: &Connection, state: &mut OverviewState) {
    state.stats = nexus_discovery::home_stats(conn).ok();
}

/// Draw the overview screen.
pub fn draw(frame: &mut Frame, area: Rect, state: &OverviewState) {
    let stats = match &state.stats {
        Some(s) => s,
        None => {
            let msg = Paragraph::new("  No scan data. Run `nexus scan` first.")
                .block(Block::bordered().title(" Home Overview "));
            frame.render_widget(msg, area);
            return;
        }
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(8), Constraint::Min(0)])
        .split(area);

    // Summary stats
    let last_scan = stats
        .last_scan
        .map(|ts| {
            chrono::DateTime::from_timestamp(ts, 0)
                .map(|dt| dt.format("%Y-%m-%d %H:%M").to_string())
                .unwrap_or_else(|| "unknown".to_string())
        })
        .unwrap_or_else(|| "never".to_string());

    let summary = vec![
        Line::from(vec![
            Span::raw("  Files:        "),
            Span::styled(
                format!("{}", stats.total_files),
                Style::default().fg(Color::Cyan).bold(),
            ),
        ]),
        Line::from(vec![
            Span::raw("  Directories:  "),
            Span::styled(
                format!("{}", stats.total_dirs),
                Style::default().fg(Color::Cyan).bold(),
            ),
        ]),
        Line::from(vec![
            Span::raw("  Total Size:   "),
            Span::styled(
                format_size(stats.total_size),
                Style::default().fg(Color::Green).bold(),
            ),
        ]),
        Line::from(vec![
            Span::raw("  Last Scan:    "),
            Span::styled(last_scan, Style::default().fg(Color::Yellow)),
        ]),
    ];

    let summary_widget =
        Paragraph::new(summary).block(Block::bordered().title(" Home Overview "));
    frame.render_widget(summary_widget, chunks[0]);

    // Category breakdown table
    draw_categories(frame, chunks[1], &stats.by_category);
}

fn draw_categories(frame: &mut Frame, area: Rect, categories: &[CategoryStats]) {
    let total_size: u64 = categories.iter().map(|c| c.total_size).sum();

    let rows: Vec<Row> = categories
        .iter()
        .map(|cat| {
            let pct = if total_size > 0 {
                (cat.total_size as f64 / total_size as f64) * 100.0
            } else {
                0.0
            };
            Row::new(vec![
                Cell::from(format!("  {}", cat.category.as_str())),
                Cell::from(format!("{}", cat.file_count)).style(Style::default().fg(Color::Cyan)),
                Cell::from(format_size(cat.total_size)).style(Style::default().fg(Color::Green)),
                Cell::from(format!("{pct:.1}%")),
            ])
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Length(16),
            Constraint::Length(10),
            Constraint::Length(12),
            Constraint::Length(8),
        ],
    )
    .header(
        Row::new(vec!["Category", "Files", "Size", "%"])
            .style(Style::default().bold())
            .bottom_margin(1),
    )
    .block(Block::bordered().title(" Categories "));

    frame.render_widget(table, area);
}
