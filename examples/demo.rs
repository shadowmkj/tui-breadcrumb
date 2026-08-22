// ==============================================================================
// tui-breadcrumb Interactive Demo
// ==============================================================================

//! Interactive demonstration for `tui-breadcrumb`.
//!
//! Features:
//! - Real-time responsive width resizing to visualize truncation strategies.
//! - Keyboard navigation (Left / Right arrow keys, Home / End).
//! - Mouse hit-testing: click segments to focus, click dropdown indicator `▾` to inspect.
//! - Dynamic cycling through all separator presets (Chevron, Slash, Angle, Arrow, Pipe, Backslash, DoubleAngle).

use std::error::Error;
use std::time::Duration;

use ratatui::crossterm::event::{
    self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind, MouseButton,
    MouseEventKind,
};
use ratatui::crossterm::execute;
use ratatui::crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Clear, Paragraph};

use tui_breadcrumb::{
    Breadcrumb, BreadcrumbItem, BreadcrumbSeparator, BreadcrumbState, TruncateStrategy,
};

struct App {
    state: BreadcrumbState,
    items: Vec<BreadcrumbItem<'static>>,
    custom_width: u16,
    separator_idx: usize,
    strategy_idx: usize,
    last_action: String,
    dropdown_message: Option<(String, usize)>,
}

impl App {
    fn new() -> Self {
        let items = vec![
            BreadcrumbItem::new("Home").style(Style::default().fg(Color::Cyan)),
            BreadcrumbItem::with_dropdown("Projects").style(Style::default().fg(Color::Blue)),
            BreadcrumbItem::with_dropdown("ratatui").style(Style::default().fg(Color::Green)),
            BreadcrumbItem::new("tui-breadcrumb").style(Style::default().fg(Color::Magenta)),
            BreadcrumbItem::with_dropdown("src").style(Style::default().fg(Color::Yellow)),
            BreadcrumbItem::new("sparkline.rs").style(Style::default().fg(Color::White)),
        ];

        let mut state = BreadcrumbState::default();
        state.select(Some(2));

        Self {
            state,
            items,
            custom_width: 55,
            separator_idx: 0,
            strategy_idx: 0,
            last_action: String::from("Application started. Use arrow keys or click crumbs."),
            dropdown_message: None,
        }
    }

    fn separator(&self) -> BreadcrumbSeparator<'static> {
        match self.separator_idx % 7 {
            0 => BreadcrumbSeparator::chevron().style(Style::default().fg(Color::DarkGray)),
            1 => BreadcrumbSeparator::slash().style(Style::default().fg(Color::DarkGray)),
            2 => BreadcrumbSeparator::angle().style(Style::default().fg(Color::DarkGray)),
            3 => BreadcrumbSeparator::arrow().style(Style::default().fg(Color::DarkGray)),
            4 => BreadcrumbSeparator::pipe().style(Style::default().fg(Color::DarkGray)),
            5 => BreadcrumbSeparator::backslash().style(Style::default().fg(Color::DarkGray)),
            _ => BreadcrumbSeparator::double_angle().style(Style::default().fg(Color::DarkGray)),
        }
    }

    fn separator_name(&self) -> &'static str {
        match self.separator_idx % 7 {
            0 => "Chevron (❯)",
            1 => "Slash (/)",
            2 => "Angle (›)",
            3 => "Arrow (→)",
            4 => "Pipe (|)",
            5 => "Backslash (\\)",
            _ => "Double Angle (»)",
        }
    }

    fn strategy(&self) -> TruncateStrategy {
        match self.strategy_idx % 5 {
            0 => TruncateStrategy::middle(),
            1 => TruncateStrategy::start(),
            2 => TruncateStrategy::shorten_names(),
            3 => TruncateStrategy::end(),
            _ => TruncateStrategy::none(),
        }
    }

    fn strategy_name(&self) -> &'static str {
        match self.strategy_idx % 5 {
            0 => "Middle (Root ❯ ... ❯ Leaf)",
            1 => "Start (... ❯ Ancestor ❯ Leaf)",
            2 => "ShortenNames (H ❯ P ❯ r ❯ Leaf)",
            3 => "End (Root ❯ Child ❯ ...)",
            _ => "None (Strict Clip)",
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new();

    // Event loop
    loop {
        terminal.draw(|frame| ui(frame, &mut app))?;

        if event::poll(Duration::from_millis(100))? {
            match event::read()? {
                Event::Key(key) if key.kind == KeyEventKind::Press => {
                    let had_popup = app.dropdown_message.is_some();
                    match key.code {
                        KeyCode::Esc => {
                            if had_popup {
                                app.dropdown_message = None;
                                app.last_action = String::from("Closed dropdown popup (Esc)");
                            } else {
                                break;
                            }
                        }
                        KeyCode::Char('q') => break,
                        KeyCode::Enter | KeyCode::Char(' ') if had_popup => {
                            app.dropdown_message = None;
                            app.last_action = String::from("Closed dropdown popup");
                        }
                        KeyCode::Left | KeyCode::Char('h') => {
                            app.dropdown_message = None;
                            let total = app.items.len();
                            app.state.select_previous(total);
                            if let Some(idx) = app.state.selected() {
                                app.last_action = format!("Selected item [{}] via Left Arrow", idx);
                            }
                        }
                        KeyCode::Right | KeyCode::Char('l') => {
                            app.dropdown_message = None;
                            let total = app.items.len();
                            app.state.select_next(total);
                            if let Some(idx) = app.state.selected() {
                                app.last_action =
                                    format!("Selected item [{}] via Right Arrow", idx);
                            }
                        }
                        KeyCode::Home => {
                            app.dropdown_message = None;
                            app.state.select_first();
                            app.last_action = String::from("Jumped to root item (Home key)");
                        }
                        KeyCode::End => {
                            app.dropdown_message = None;
                            let total = app.items.len();
                            app.state.select_last(total);
                            app.last_action = String::from("Jumped to leaf item (End key)");
                        }
                        KeyCode::Tab => {
                            app.dropdown_message = None;
                            app.separator_idx = (app.separator_idx + 1) % 7;
                            app.last_action =
                                format!("Switched separator to {}", app.separator_name());
                        }
                        KeyCode::Char('1') => {
                            app.dropdown_message = None;
                            app.strategy_idx = 0;
                            app.last_action = format!("Set strategy: {}", app.strategy_name());
                        }
                        KeyCode::Char('2') => {
                            app.dropdown_message = None;
                            app.strategy_idx = 1;
                            app.last_action = format!("Set strategy: {}", app.strategy_name());
                        }
                        KeyCode::Char('3') => {
                            app.dropdown_message = None;
                            app.strategy_idx = 2;
                            app.last_action = format!("Set strategy: {}", app.strategy_name());
                        }
                        KeyCode::Char('4') => {
                            app.dropdown_message = None;
                            app.strategy_idx = 3;
                            app.last_action = format!("Set strategy: {}", app.strategy_name());
                        }
                        KeyCode::Char('5') => {
                            app.dropdown_message = None;
                            app.strategy_idx = 4;
                            app.last_action = format!("Set strategy: {}", app.strategy_name());
                        }
                        KeyCode::Char('+') | KeyCode::Char('=') | KeyCode::Char(']') => {
                            app.custom_width = (app.custom_width + 2).min(120);
                            app.last_action =
                                format!("Increased demo width to {}", app.custom_width);
                        }
                        KeyCode::Char('-') | KeyCode::Char('_') | KeyCode::Char('[') => {
                            app.custom_width = (app.custom_width.saturating_sub(2)).max(15);
                            app.last_action =
                                format!("Decreased demo width to {}", app.custom_width);
                        }
                        _ => {
                            if had_popup {
                                app.dropdown_message = None;
                                app.last_action = String::from("Closed dropdown popup");
                            }
                        }
                    }
                }
                Event::Mouse(mouse) if mouse.kind == MouseEventKind::Down(MouseButton::Left) => {
                    let (col, row) = (mouse.column, mouse.row);
                    if let Some(drop_idx) = app.state.dropdown_at(col, row) {
                        // Toggle if already showing this dropdown
                        if app
                            .dropdown_message
                            .as_ref()
                            .is_some_and(|(_, current_idx)| *current_idx == drop_idx)
                        {
                            app.dropdown_message = None;
                            app.last_action = format!("Closed dropdown for [{}]", drop_idx);
                            continue;
                        }
                        let label: String = app.items[drop_idx]
                            .label
                            .spans
                            .iter()
                            .map(|s| s.content.as_ref())
                            .collect();
                        app.dropdown_message = Some((label.clone(), drop_idx));
                        app.last_action = format!(
                            "Clicked dropdown trigger on segment [{}] ({})",
                            drop_idx, label
                        );
                    } else if let Some(item_idx) = app.state.item_at(col, row) {
                        app.state.select(Some(item_idx));
                        app.dropdown_message = None;
                        app.last_action = format!("Clicked directly on segment [{}]", item_idx);
                    } else if app.state.is_ellipsis_at(col, row) {
                        app.dropdown_message = None;
                        app.last_action =
                            String::from("Clicked on collapsed ellipsis indicator (...)");
                    } else if app.dropdown_message.is_some() {
                        app.dropdown_message = None;
                        app.last_action = String::from("Closed dropdown popup (clicked outside)");
                    }
                }
                _ => {}
            }
        }
    }

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}

fn ui(frame: &mut Frame, app: &mut App) {
    let area = frame.area();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Length(7), // Interactive Test Box
            Constraint::Length(9), // Strategies Comparison
            Constraint::Min(4),    // Keybindings & Status
        ])
        .split(area);

    // 1. Header
    let title = Paragraph::new(Line::from(vec![
        Span::styled(
            " 🍞 tui-breadcrumb ",
            Style::default().fg(Color::Black).bg(Color::Yellow).bold(),
        ),
        Span::raw(" — Hierarchical Navigation Trail Widget for Ratatui"),
    ]))
    .block(Block::default().borders(Borders::ALL));
    frame.render_widget(title, chunks[0]);

    // 2. Main Interactive Trail Box
    let max_demo_width = app.custom_width.min(chunks[1].width.saturating_sub(4));
    let demo_rect = Rect::new(chunks[1].x + 2, chunks[1].y + 1, max_demo_width, 5);

    let widget = Breadcrumb::new(app.items.clone())
        .separator(app.separator())
        .strategy(app.strategy())
        .selected_style(Style::default().bg(Color::DarkGray).fg(Color::White).bold())
        .block(
            Block::default()
                .title(format!(
                    " Interactive Demo (Width: {} cols) [Use +/- to resize] ",
                    max_demo_width
                ))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Yellow)),
        );

    frame.render_stateful_widget(widget, demo_rect, &mut app.state);

    // 3. Side-by-Side Comparison of Strategies
    let comp_block = Block::default()
        .title(" All Truncation Strategies Side-by-Side ")
        .borders(Borders::ALL);
    let inner_comp = comp_block.inner(chunks[2]);
    frame.render_widget(comp_block, chunks[2]);

    let strat_layouts = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Min(0),
        ])
        .split(inner_comp);

    let test_width = max_demo_width.saturating_sub(2);
    let target_rect = Rect::new(inner_comp.x, 0, test_width, 1);

    let middle_widget = Breadcrumb::new(app.items.clone())
        .separator(app.separator())
        .strategy(TruncateStrategy::middle());
    let mut r0 = target_rect;
    r0.y = strat_layouts[0].y;
    frame.render_widget(middle_widget, r0);

    let start_widget = Breadcrumb::new(app.items.clone())
        .separator(app.separator())
        .strategy(TruncateStrategy::start());
    let mut r1 = target_rect;
    r1.y = strat_layouts[1].y;
    frame.render_widget(start_widget, r1);

    let shorten_widget = Breadcrumb::new(app.items.clone())
        .separator(app.separator())
        .strategy(TruncateStrategy::shorten_names());
    let mut r2 = target_rect;
    r2.y = strat_layouts[2].y;
    frame.render_widget(shorten_widget, r2);

    let end_widget = Breadcrumb::new(app.items.clone())
        .separator(app.separator())
        .strategy(TruncateStrategy::end());
    let mut r3 = target_rect;
    r3.y = strat_layouts[3].y;
    frame.render_widget(end_widget, r3);

    // 4. Instructions & Footer
    let help_text = vec![
        Line::from(vec![
            Span::styled(" [←/→/h/l] ", Style::default().bold().fg(Color::Yellow)),
            Span::raw("Navigate segments   "),
            Span::styled(" [Tab] ", Style::default().bold().fg(Color::Yellow)),
            Span::raw(format!("Cycle Separator ({})   ", app.separator_name())),
            Span::styled(" [1-5] ", Style::default().bold().fg(Color::Yellow)),
            Span::raw(format!("Strategy ({})", app.strategy_name())),
        ]),
        Line::from(vec![
            Span::styled(" [+/-] ", Style::default().bold().fg(Color::Yellow)),
            Span::raw("Resize demo width   "),
            Span::styled(" [Mouse Click] ", Style::default().bold().fg(Color::Yellow)),
            Span::raw("Click crumbs or ▾ dropdown   "),
            Span::styled(" [q/Esc] ", Style::default().bold().fg(Color::Yellow)),
            Span::raw("Quit"),
        ]),
        Line::from(vec![
            Span::styled(" Status: ", Style::default().bold().fg(Color::Green)),
            Span::styled(&app.last_action, Style::default().fg(Color::White)),
        ]),
    ];

    let footer = Paragraph::new(help_text).block(
        Block::default()
            .title(" Controls & Info ")
            .borders(Borders::ALL),
    );
    frame.render_widget(footer, chunks[3]);

    // Optional Dropdown Popup
    if let Some((label, idx)) = &app.dropdown_message {
        let popup_area = Rect::new(
            area.width.saturating_sub(35) / 2,
            area.height.saturating_sub(6) / 2,
            36,
            6,
        );
        frame.render_widget(Clear, popup_area);
        let popup = Paragraph::new(vec![
            Line::from(format!("Ancestor sub-menu for: {}", label)),
            Line::from(format!("Segment Index: {}", idx)),
            Line::from("Press Esc / Enter / Space or click anywhere to close."),
        ])
        .block(
            Block::default()
                .title(" Dropdown Triggered ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Green)),
        );
        frame.render_widget(popup, popup_area);
    }
}
