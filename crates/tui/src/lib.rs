//! TUI dashboard for Nexus — ratatui terminal interface.
//!
//! Four screens: Overview, Configs, Search, Changes.
//! Message-passing architecture: Event → update → draw.

mod screens;

use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
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
    Changes,
}

/// Main TUI application state.
pub struct App {
    screen: Screen,
    should_quit: bool,
    db: Connection,
    show_help: bool,
    // Screen-specific state
    overview: screens::overview::OverviewState,
    configs: screens::configs::ConfigsState,
    search: screens::search::SearchState,
    changes: screens::changes::ChangesState,
}

impl App {
    /// Create a new app instance.
    pub fn new(db: Connection) -> Self {
        Self {
            screen: Screen::Overview,
            should_quit: false,
            show_help: false,
            overview: screens::overview::OverviewState::default(),
            configs: screens::configs::ConfigsState::default(),
            search: screens::search::SearchState::default(),
            changes: screens::changes::ChangesState::default(),
            db,
        }
    }

    /// Load data for the current screen.
    fn refresh(&mut self) {
        match self.screen {
            Screen::Overview => screens::overview::refresh(&self.db, &mut self.overview),
            Screen::Configs => screens::configs::refresh(&self.db, &mut self.configs),
            Screen::Search => {} // search is on-demand
            Screen::Changes => screens::changes::refresh(&self.db, &mut self.changes),
        }
    }
}

/// Launch the TUI.
pub fn run(db: Connection) -> Result<()> {
    let mut terminal = ratatui::init();
    let mut app = App::new(db);
    app.refresh();

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
                    handle_key(&mut app, key.code, key.modifiers);
                }
            }
        }
    }

    ratatui::restore();
    Ok(())
}

fn handle_key(app: &mut App, code: KeyCode, modifiers: KeyModifiers) {
    // Global keys
    if app.show_help {
        app.show_help = false;
        return;
    }

    // If in search input mode, route to search handler first
    if app.screen == Screen::Search && app.search.input_active {
        screens::search::handle_key(&mut app.search, &app.db, code, modifiers);
        return;
    }

    match code {
        KeyCode::Char('q') | KeyCode::Esc => app.should_quit = true,
        KeyCode::Char('?') => app.show_help = true,
        KeyCode::Char('1') => {
            app.screen = Screen::Overview;
            app.refresh();
        }
        KeyCode::Char('2') => {
            app.screen = Screen::Configs;
            app.refresh();
        }
        KeyCode::Char('3') => {
            app.screen = Screen::Search;
        }
        KeyCode::Char('4') => {
            app.screen = Screen::Changes;
            app.refresh();
        }
        KeyCode::Char('r') if modifiers.contains(KeyModifiers::CONTROL) => {
            app.refresh();
        }
        _ => {
            // Delegate to screen-specific handler
            match app.screen {
                Screen::Overview => {}
                Screen::Configs => {
                    screens::configs::handle_key(&mut app.configs, &app.db, code);
                }
                Screen::Search => {
                    screens::search::handle_key(&mut app.search, &app.db, code, modifiers);
                }
                Screen::Changes => {
                    screens::changes::handle_key(&mut app.changes, code);
                }
            }
        }
    }
}

fn draw(frame: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .split(frame.area());

    // Tab bar
    let tab_titles = vec![
        "[1] Overview",
        "[2] Configs",
        "[3] Search",
        "[4] Changes",
    ];
    let tabs = Tabs::new(tab_titles)
        .block(Block::bordered().title(" Nexus "))
        .highlight_style(Style::default().fg(Color::Cyan).bold())
        .select(match app.screen {
            Screen::Overview => 0,
            Screen::Configs => 1,
            Screen::Search => 2,
            Screen::Changes => 3,
        });
    frame.render_widget(tabs, chunks[0]);

    // Content
    match app.screen {
        Screen::Overview => screens::overview::draw(frame, chunks[1], &app.overview),
        Screen::Configs => screens::configs::draw(frame, chunks[1], &app.configs),
        Screen::Search => screens::search::draw(frame, chunks[1], &app.search),
        Screen::Changes => screens::changes::draw(frame, chunks[1], &app.changes),
    }

    // Help bar
    let help_text = match app.screen {
        Screen::Overview => "q: quit | 1-4: tabs | Ctrl+R: refresh | ?: help",
        Screen::Configs => {
            if app.configs.viewing_file {
                "Esc: back | j/k: scroll | q: quit"
            } else {
                "j/k: navigate | Enter: view file | q: quit | 1-4: tabs"
            }
        }
        Screen::Search => {
            if app.search.input_active {
                "Enter: search | Esc: cancel | type query..."
            } else {
                "/: search | j/k: navigate | q: quit | 1-4: tabs"
            }
        }
        Screen::Changes => "j/k: navigate | q: quit | 1-4: tabs",
    };
    let help = Paragraph::new(help_text).style(Style::default().fg(Color::DarkGray));
    frame.render_widget(help, chunks[2]);

    // Help overlay
    if app.show_help {
        draw_help_overlay(frame);
    }
}

fn draw_help_overlay(frame: &mut Frame) {
    let area = frame.area();
    let popup = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(20),
            Constraint::Min(14),
            Constraint::Percentage(20),
        ])
        .split(area);
    let popup = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(20),
            Constraint::Min(40),
            Constraint::Percentage(20),
        ])
        .split(popup[1]);

    let help_text = vec![
        Line::from("Keybindings").style(Style::default().bold()),
        Line::from(""),
        Line::from("  1-4       Switch tabs"),
        Line::from("  j/k       Navigate up/down"),
        Line::from("  Enter     Select / expand"),
        Line::from("  Esc       Back / close"),
        Line::from("  /         Start search (Search tab)"),
        Line::from("  Ctrl+R    Refresh data"),
        Line::from("  ?         Toggle this help"),
        Line::from("  q         Quit"),
        Line::from(""),
        Line::from("Press any key to close"),
    ];

    frame.render_widget(Clear, popup[1]);
    let block = Paragraph::new(help_text).block(Block::bordered().title(" Help "));
    frame.render_widget(block, popup[1]);
}
