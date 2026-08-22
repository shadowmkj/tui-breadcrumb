//! Interactive File Explorer Example for `tui-breadcrumbs`
//!
//! Demonstrates:
//! - Real-time integration of `Breadcrumb::from_path(std::path::Path)`.
//! - Live directory navigation with keyboard (`Enter` to open, `Backspace` to go up).
//! - Direct mouse hit-testing: clicking any ancestor breadcrumb segment jumps directly to that directory.

use std::error::Error;
use std::fs;
use std::path::PathBuf;
use std::time::Duration;

use ratatui::crossterm::event::{
    self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind, MouseButton,
    MouseEventKind,
};
use ratatui::crossterm::execute;
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph};

use tui_breadcrumbs::{Breadcrumb, BreadcrumbSeparator, BreadcrumbState, TruncateStrategy};

struct DirectoryEntry {
    name: String,
    is_dir: bool,
    path: PathBuf,
}

struct App {
    current_dir: PathBuf,
    entries: Vec<DirectoryEntry>,
    list_state: ListState,
    breadcrumb_state: BreadcrumbState,
    status_message: String,
}

impl App {
    fn new() -> Self {
        let current_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        let mut app = Self {
            current_dir,
            entries: Vec::new(),
            list_state: ListState::default(),
            breadcrumb_state: BreadcrumbState::default(),
            status_message: String::from(
                "Navigate folders with [Enter] / [Backspace], or click crumbs!",
            ),
        };
        app.reload_directory();
        app
    }

    fn reload_directory(&mut self) {
        self.entries.clear();

        if let Ok(read_dir) = fs::read_dir(&self.current_dir) {
            let mut items: Vec<DirectoryEntry> = read_dir
                .filter_map(|res| res.ok())
                .map(|entry| {
                    let path = entry.path();
                    let is_dir = path.is_dir();
                    let name = entry.file_name().to_string_lossy().to_string();
                    DirectoryEntry { name, is_dir, path }
                })
                .collect();

            // Sort directories first, then alphabetical
            items.sort_by(|a, b| match (a.is_dir, b.is_dir) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
            });

            self.entries = items;
        }

        if !self.entries.is_empty() {
            self.list_state.select(Some(0));
        } else {
            self.list_state.select(None);
        }
    }

    fn navigate_into_selected(&mut self) {
        if let Some(selected_idx) = self.list_state.selected()
            && let Some(entry) = self.entries.get(selected_idx)
        {
            if entry.is_dir {
                self.current_dir = entry.path.clone();
                self.status_message = format!("Entered directory: {}", self.current_dir.display());
                self.reload_directory();
            } else {
                self.status_message = format!("Selected file: {}", entry.name);
            }
        }
    }

    fn navigate_up(&mut self) {
        if let Some(parent) = self.current_dir.parent() {
            self.current_dir = parent.to_path_buf();
            self.status_message = format!("Moved up to: {}", self.current_dir.display());
            self.reload_directory();
        } else {
            self.status_message = String::from("Already at filesystem root.");
        }
    }

    fn jump_to_ancestor_index(&mut self, item_index: usize) {
        let items = tui_breadcrumbs::from_path(&self.current_dir);
        if item_index < items.len() {
            // Reconstruct path up to item_index
            let mut target = PathBuf::new();
            for (idx, comp) in self.current_dir.components().enumerate() {
                target.push(comp.as_os_str());
                if idx == item_index {
                    break;
                }
            }
            if target.exists() {
                self.current_dir = target;
                self.status_message = format!("Jumped to ancestor: {}", self.current_dir.display());
                self.reload_directory();
            }
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut terminal = ratatui::init();
    execute!(std::io::stdout(), EnableMouseCapture)?;

    let mut app = App::new();

    loop {
        terminal.draw(|frame| {
            let area = frame.area();

            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3), // Top Breadcrumb Bar
                    Constraint::Min(5),    // Directory Listing Pane
                    Constraint::Length(3), // Status & Keybindings Footer
                ])
                .split(area);

            // 1. Top Bar: Path Breadcrumbs
            let breadcrumbs = Breadcrumb::from_path(&app.current_dir)
                .separator(BreadcrumbSeparator::slash())
                .strategy(TruncateStrategy::middle())
                .item_style(Style::default().fg(Color::Cyan))
                .active_style(Style::default().fg(Color::Yellow).bold())
                .block(
                    Block::default()
                        .title(" 📁 Active Path (Click any segment to jump) ")
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(Color::Cyan)),
                );

            frame.render_stateful_widget(breadcrumbs, chunks[0], &mut app.breadcrumb_state);

            // 2. Directory Contents List
            let list_items: Vec<ListItem> = app
                .entries
                .iter()
                .map(|e| {
                    let icon = if e.is_dir { "📂 " } else { "📄 " };
                    let style = if e.is_dir {
                        Style::default().fg(Color::Blue).bold()
                    } else {
                        Style::default().fg(Color::White)
                    };
                    ListItem::new(Line::from(vec![
                        Span::styled(icon, style),
                        Span::styled(&e.name, style),
                    ]))
                })
                .collect();

            let count_label = format!(" Directory Contents ({} items) ", app.entries.len());
            let list_widget = List::new(list_items)
                .block(Block::default().title(count_label).borders(Borders::ALL))
                .highlight_style(
                    Style::default()
                        .bg(Color::DarkGray)
                        .fg(Color::Yellow)
                        .bold(),
                )
                .highlight_symbol("▶ ");

            frame.render_stateful_widget(list_widget, chunks[1], &mut app.list_state);

            // 3. Footer
            let footer = Paragraph::new(vec![
                Line::from(vec![
                    Span::styled(" [Enter] ", Style::default().bold().fg(Color::Yellow)),
                    Span::raw("Open dir   "),
                    Span::styled(" [Backspace/h] ", Style::default().bold().fg(Color::Yellow)),
                    Span::raw("Parent dir   "),
                    Span::styled(" [Mouse Click] ", Style::default().bold().fg(Color::Yellow)),
                    Span::raw("Jump to crumb   "),
                    Span::styled(" [q/Esc] ", Style::default().bold().fg(Color::Yellow)),
                    Span::raw("Quit"),
                ]),
                Line::from(vec![
                    Span::styled(" Status: ", Style::default().bold().fg(Color::Green)),
                    Span::styled(&app.status_message, Style::default().fg(Color::White)),
                ]),
            ])
            .block(Block::default().borders(Borders::ALL));

            frame.render_widget(footer, chunks[2]);
        })?;

        if event::poll(Duration::from_millis(100))? {
            match event::read()? {
                Event::Key(key) if key.kind == KeyEventKind::Press => match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => break,
                    KeyCode::Up | KeyCode::Char('k') => {
                        let i = match app.list_state.selected() {
                            Some(i) => {
                                if i == 0 {
                                    app.entries.len().saturating_sub(1)
                                } else {
                                    i - 1
                                }
                            }
                            None => 0,
                        };
                        app.list_state.select(Some(i));
                    }
                    KeyCode::Down | KeyCode::Char('j') => {
                        let i = match app.list_state.selected() {
                            Some(i) => {
                                if i + 1 >= app.entries.len() {
                                    0
                                } else {
                                    i + 1
                                }
                            }
                            None => 0,
                        };
                        app.list_state.select(Some(i));
                    }
                    KeyCode::Enter | KeyCode::Char('l') => {
                        app.navigate_into_selected();
                    }
                    KeyCode::Backspace | KeyCode::Char('h') => {
                        app.navigate_up();
                    }
                    _ => {}
                },
                Event::Mouse(mouse) if mouse.kind == MouseEventKind::Down(MouseButton::Left) => {
                    let (col, row) = (mouse.column, mouse.row);
                    if let Some(item_idx) = app.breadcrumb_state.item_at(col, row) {
                        app.jump_to_ancestor_index(item_idx);
                    }
                }
                _ => {}
            }
        }
    }

    execute!(std::io::stdout(), DisableMouseCapture)?;
    ratatui::restore();
    Ok(())
}
