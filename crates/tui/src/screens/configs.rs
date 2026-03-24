//! Configs screen — browse config tools, view files with syntax highlighting.

use crossterm::event::KeyCode;
use ratatui::prelude::*;
use ratatui::widgets::*;
use rusqlite::Connection;
use syntect::easy::HighlightLines;
use syntect::highlighting::{Style as SynStyle, ThemeSet};
use syntect::parsing::SyntaxSet;

/// State for the configs screen.
pub struct ConfigsState {
    pub tools: Vec<ToolRow>,
    pub selected: usize,
    pub viewing_file: bool,
    pub file_content: Vec<String>,
    pub file_highlighted: Vec<Vec<(SynStyle, String)>>,
    pub file_name: String,
    pub file_scroll: usize,
    syntax_set: SyntaxSet,
    theme_set: ThemeSet,
}

impl Default for ConfigsState {
    fn default() -> Self {
        Self {
            tools: Vec::new(),
            selected: 0,
            viewing_file: false,
            file_content: Vec::new(),
            file_highlighted: Vec::new(),
            file_name: String::new(),
            file_scroll: 0,
            syntax_set: SyntaxSet::load_defaults_newlines(),
            theme_set: ThemeSet::load_defaults(),
        }
    }
}

/// A row in the tools table.
pub struct ToolRow {
    pub name: String,
    pub config_dir: String,
    pub language: String,
    pub file_count: u32,
}

/// Load config tools from the database.
pub fn refresh(conn: &Connection, state: &mut ConfigsState) {
    let mut stmt = conn
        .prepare(
            "SELECT t.name, t.config_dir, t.language,
                    (SELECT COUNT(*) FROM config_files WHERE tool_id = t.id)
             FROM config_tools t ORDER BY t.name",
        )
        .ok();

    state.tools = stmt
        .as_mut()
        .and_then(|s| {
            s.query_map([], |row: &rusqlite::Row<'_>| {
                Ok(ToolRow {
                    name: row.get(0)?,
                    config_dir: row.get(1)?,
                    language: row.get::<_, String>(2).unwrap_or_default(),
                    file_count: row.get::<_, i64>(3).unwrap_or(0) as u32,
                })
            })
            .ok()
        })
        .map(|r| r.filter_map(|r| r.ok()).collect())
        .unwrap_or_default();

    if state.selected >= state.tools.len() && !state.tools.is_empty() {
        state.selected = state.tools.len() - 1;
    }
}

/// Handle key events for the configs screen.
pub fn handle_key(state: &mut ConfigsState, _conn: &Connection, code: KeyCode) {
    if state.viewing_file {
        match code {
            KeyCode::Esc | KeyCode::Backspace => {
                state.viewing_file = false;
                state.file_content.clear();
                state.file_highlighted.clear();
                state.file_scroll = 0;
            }
            KeyCode::Char('j') | KeyCode::Down => {
                if state.file_scroll + 1 < state.file_content.len() {
                    state.file_scroll += 1;
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                state.file_scroll = state.file_scroll.saturating_sub(1);
            }
            KeyCode::Char('d') => {
                state.file_scroll = state.file_scroll.saturating_add(20);
                if state.file_scroll >= state.file_content.len() {
                    state.file_scroll = state.file_content.len().saturating_sub(1);
                }
            }
            KeyCode::Char('u') => {
                state.file_scroll = state.file_scroll.saturating_sub(20);
            }
            _ => {}
        }
        return;
    }

    match code {
        KeyCode::Char('j') | KeyCode::Down => {
            if !state.tools.is_empty() {
                state.selected = (state.selected + 1).min(state.tools.len() - 1);
            }
        }
        KeyCode::Char('k') | KeyCode::Up => {
            state.selected = state.selected.saturating_sub(1);
        }
        KeyCode::Enter => {
            if state.selected < state.tools.len() {
                let config_dir = state.tools[state.selected].config_dir.clone();
                load_tool_main_config(state, &config_dir);
            }
        }
        _ => {}
    }
}

fn load_tool_main_config(state: &mut ConfigsState, config_dir: &str) {
    let dir = std::path::Path::new(config_dir);
    if !dir.exists() {
        return;
    }

    // Find the first readable config file in the tool's directory
    let mut files: Vec<std::path::PathBuf> = Vec::new();
    collect_files(dir, &mut files, 3); // max depth 3
    files.sort();

    if let Some(file_path) = files.first() {
        if let Ok(content) = std::fs::read_to_string(file_path) {
            state.file_name = file_path.to_string_lossy().to_string();
            state.file_content = content.lines().map(String::from).collect();
            state.file_highlighted = highlight_content(&state.syntax_set, &state.theme_set, &content, file_path);
            state.file_scroll = 0;
            state.viewing_file = true;
        }
    }
}

fn collect_files(dir: &std::path::Path, out: &mut Vec<std::path::PathBuf>, depth: u32) {
    if depth == 0 {
        return;
    }
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_file() {
            out.push(path);
        } else if path.is_dir() && depth > 1 {
            collect_files(&path, out, depth - 1);
        }
    }
}

fn highlight_content(
    ss: &SyntaxSet,
    ts: &ThemeSet,
    content: &str,
    path: &std::path::Path,
) -> Vec<Vec<(SynStyle, String)>> {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");

    let syntax = ss
        .find_syntax_by_extension(ext)
        .or_else(|| ss.find_syntax_by_first_line(content.lines().next().unwrap_or("")))
        .unwrap_or_else(|| ss.find_syntax_plain_text());

    let theme = &ts.themes["base16-ocean.dark"];
    let mut highlighter = HighlightLines::new(syntax, theme);

    content
        .lines()
        .map(|line| {
            highlighter
                .highlight_line(line, ss)
                .unwrap_or_default()
                .into_iter()
                .map(|(style, text)| (style, text.to_string()))
                .collect()
        })
        .collect()
}

/// Draw the configs screen.
pub fn draw(frame: &mut Frame, area: Rect, state: &ConfigsState) {
    if state.viewing_file {
        draw_file_viewer(frame, area, state);
        return;
    }

    let rows: Vec<Row> = state
        .tools
        .iter()
        .enumerate()
        .map(|(i, tool)| {
            let style = if i == state.selected {
                Style::default().bg(Color::DarkGray).fg(Color::White)
            } else {
                Style::default()
            };
            Row::new(vec![
                Cell::from(tool.name.as_str()),
                Cell::from(tool.config_dir.as_str()),
                Cell::from(tool.language.as_str()),
                Cell::from(format!("{}", tool.file_count)),
            ])
            .style(style)
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Length(15),
            Constraint::Min(30),
            Constraint::Length(10),
            Constraint::Length(8),
        ],
    )
    .header(
        Row::new(vec!["Tool", "Path", "Language", "Files"])
            .style(Style::default().bold())
            .bottom_margin(1),
    )
    .block(Block::bordered().title(" Config Tools "));

    frame.render_widget(table, area);
}

fn draw_file_viewer(frame: &mut Frame, area: Rect, state: &ConfigsState) {
    let title = format!(" {} ", state.file_name);
    let block = Block::bordered().title(title);
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let visible_height = inner.height as usize;
    let start = state.file_scroll;
    let end = (start + visible_height).min(state.file_highlighted.len());

    let lines: Vec<Line> = state.file_highlighted[start..end]
        .iter()
        .enumerate()
        .map(|(i, spans)| {
            let line_num = start + i + 1;
            let mut line_spans = vec![Span::styled(
                format!("{line_num:4} "),
                Style::default().fg(Color::DarkGray),
            )];
            for (syn_style, text) in spans {
                let fg = Color::Rgb(
                    syn_style.foreground.r,
                    syn_style.foreground.g,
                    syn_style.foreground.b,
                );
                line_spans.push(Span::styled(text.clone(), Style::default().fg(fg)));
            }
            Line::from(line_spans)
        })
        .collect();

    let scroll_info = format!(
        " {}/{} ",
        state.file_scroll + 1,
        state.file_content.len()
    );
    let scroll_span = Paragraph::new(scroll_info)
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Right);

    let content_area = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(1)])
        .split(inner);

    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, content_area[0]);
    frame.render_widget(scroll_span, content_area[1]);
}
