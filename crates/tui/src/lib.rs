//! TUI dashboard for Nexus — ratatui terminal interface.
//!
//! Four screens: Overview, Configs, Search, AI Chat.
//! Message-passing architecture: Event → update → draw.

use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use nexus_core::Result;
use ratatui::prelude::*;
use ratatui::widgets::*;
use rusqlite::Connection;

/// Active screen in the TUI.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    Overview,
    Configs,
    Search,
}

/// Main TUI application state.
pub struct App {
    screen: Screen,
    should_quit: bool,
    db: Connection,
}

impl App {
    /// Create a new app instance.
    pub fn new(db: Connection) -> Self {
        Self {
            screen: Screen::Overview,
            should_quit: false,
            db,
        }
    }
}

/// Launch the TUI.
pub fn run(db: Connection) -> Result<()> {
    let mut terminal = ratatui::init();
    let mut app = App::new(db);

    while !app.should_quit {
        terminal
            .draw(|frame| draw(frame, &app))
            .map_err(|e| nexus_core::NexusError::Internal(e.to_string()))?;

        if event::poll(std::time::Duration::from_millis(250))
            .map_err(|e| nexus_core::NexusError::Internal(e.to_string()))?
        {
            if let Event::Key(key) =
                event::read().map_err(|e| nexus_core::NexusError::Internal(e.to_string()))?
            {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => app.should_quit = true,
                        KeyCode::Char('1') => app.screen = Screen::Overview,
                        KeyCode::Char('2') => app.screen = Screen::Configs,
                        KeyCode::Char('3') => app.screen = Screen::Search,
                        _ => {}
                    }
                }
            }
        }
    }

    ratatui::restore();
    Ok(())
}

fn draw(frame: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)])
        .split(frame.area());

    // Tab bar
    let tabs = Tabs::new(vec!["[1] Overview", "[2] Configs", "[3] Search"])
        .block(Block::bordered().title(" Nexus "))
        .highlight_style(Style::default().fg(Color::Cyan).bold())
        .select(match app.screen {
            Screen::Overview => 0,
            Screen::Configs => 1,
            Screen::Search => 2,
        });
    frame.render_widget(tabs, chunks[0]);

    // Content
    match app.screen {
        Screen::Overview => draw_overview(frame, chunks[1], app),
        Screen::Configs => draw_configs(frame, chunks[1], app),
        Screen::Search => draw_search(frame, chunks[1]),
    }
}

fn draw_overview(frame: &mut Frame, area: Rect, app: &App) {
    let stats = nexus_discovery::home_stats(&app.db).unwrap_or(nexus_core::models::HomeStats {
        total_files: 0,
        total_dirs: 0,
        total_size: 0,
        by_category: vec![],
        last_scan: None,
    });

    let text = vec![
        Line::from(format!("  Files: {}", stats.total_files)),
        Line::from(format!("  Directories: {}", stats.total_dirs)),
        Line::from(format!(
            "  Total Size: {}",
            nexus_core::output::format_size(stats.total_size)
        )),
        Line::from(""),
        Line::from("  Press 'q' to quit, 1-3 to switch tabs"),
    ];

    let paragraph = Paragraph::new(text).block(Block::bordered().title(" Home Overview "));
    frame.render_widget(paragraph, area);
}

fn draw_configs(frame: &mut Frame, area: Rect, app: &App) {
    let mut stmt = app
        .db
        .prepare("SELECT name, config_dir, language FROM config_tools ORDER BY name")
        .ok();

    let tool_data: Vec<(String, String, String)> = stmt
        .as_mut()
        .and_then(|s| {
            s.query_map([], |row: &rusqlite::Row<'_>| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2).unwrap_or_default(),
                ))
            })
            .ok()
        })
        .map(|r| {
            r.filter_map(|r: std::result::Result<(String, String, String), _>| r.ok())
                .collect()
        })
        .unwrap_or_default();

    let rows: Vec<Row> = tool_data
        .iter()
        .map(|(name, dir, lang)| {
            Row::new(vec![
                Cell::from(name.as_str()),
                Cell::from(dir.as_str()),
                Cell::from(lang.as_str()),
            ])
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Length(15),
            Constraint::Min(30),
            Constraint::Length(10),
        ],
    )
    .header(Row::new(vec!["Tool", "Path", "Language"]).style(Style::default().bold()))
    .block(Block::bordered().title(" Config Tools "));

    frame.render_widget(table, area);
}

fn draw_search(frame: &mut Frame, area: Rect) {
    let paragraph = Paragraph::new("  Search coming soon — use CLI: nexus search <query>")
        .block(Block::bordered().title(" Search "));
    frame.render_widget(paragraph, area);
}
