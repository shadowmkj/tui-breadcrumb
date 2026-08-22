//! Responsive Truncation Lab Example for `tui-breadcrumb`
//!
//! Demonstrates:
//! - Interactive exploration and side-by-side comparison of all 5 truncation strategies.
//! - Live width resizing caliper ruler (adjust from 12 to 100 columns).
//! - Unicode and emoji handling across narrow terminal bounds.

use std::error::Error;
use std::time::Duration;

use ratatui::crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph};

use tui_breadcrumb::{Breadcrumb, BreadcrumbItem, BreadcrumbSeparator, TruncateStrategy};

struct App {
    width: u16,
    items: Vec<BreadcrumbItem<'static>>,
}

impl App {
    fn new() -> Self {
        let items = vec![
            BreadcrumbItem::new("📁 Home"),
            BreadcrumbItem::new("💻 Workspace"),
            BreadcrumbItem::new("🦀 ratatui"),
            BreadcrumbItem::new("⚡ components"),
            BreadcrumbItem::new("🎨 styles"),
            BreadcrumbItem::new("📄 sparkline.rs"),
        ];

        Self { width: 50, items }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut terminal = ratatui::init();
    let mut app = App::new();

    loop {
        terminal.draw(|frame| {
            let area = frame.area();

            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3), // Header
                    Constraint::Length(3), // Width Caliper & Slider
                    Constraint::Length(4), // Strategy 1: Middle
                    Constraint::Length(4), // Strategy 2: Start
                    Constraint::Length(4), // Strategy 3: ShortenNames
                    Constraint::Length(4), // Strategy 4: End
                    Constraint::Length(4), // Strategy 5: None
                    Constraint::Min(3),    // Footer
                ])
                .split(area);

            // 1. Header
            let header = Paragraph::new(Line::from(vec![
                Span::styled(
                    " 🔬 Truncation Lab ",
                    Style::default().fg(Color::Black).bg(Color::Magenta).bold(),
                ),
                Span::raw(" — Side-by-Side Responsive Overflow Comparison"),
            ]))
            .block(Block::default().borders(Borders::ALL));
            frame.render_widget(header, chunks[0]);

            // 2. Caliper Ruler
            let max_w = app.width.min(area.width.saturating_sub(4));
            let mut ruler = String::new();
            for i in 1..=max_w {
                if i % 10 == 0 {
                    ruler.push_str(&format!("{}", (i / 10) % 10));
                } else if i % 5 == 0 {
                    ruler.push('+');
                } else {
                    ruler.push('-');
                }
            }

            let caliper = Paragraph::new(vec![
                Line::from(vec![
                    Span::styled(
                        format!(" Current Container Width: {} cols ", max_w),
                        Style::default().fg(Color::Yellow).bold(),
                    ),
                    Span::raw("(Use [←/→] or [+/-] to resize)"),
                ]),
                Line::from(Span::styled(ruler, Style::default().fg(Color::DarkGray))),
            ])
            .block(
                Block::default()
                    .title(" Width Caliper ")
                    .borders(Borders::ALL),
            );
            frame.render_widget(caliper, chunks[1]);

            let render_rect = Rect::new(chunks[2].x + 1, 0, max_w, 3);

            // 3. Strategy 1: Middle
            let middle_w = Breadcrumb::new(app.items.clone())
                .separator(BreadcrumbSeparator::chevron())
                .strategy(TruncateStrategy::middle())
                .item_style(Style::default().fg(Color::Cyan))
                .active_style(Style::default().fg(Color::Yellow).bold())
                .block(
                    Block::default()
                        .title(" 1. Middle (Root ❯ ... ❯ Leaf) ")
                        .borders(Borders::ALL),
                );
            let mut r1 = render_rect;
            r1.y = chunks[2].y;
            frame.render_widget(middle_w, r1);

            // 4. Strategy 2: Start
            let start_w = Breadcrumb::new(app.items.clone())
                .separator(BreadcrumbSeparator::chevron())
                .strategy(TruncateStrategy::start())
                .item_style(Style::default().fg(Color::Cyan))
                .active_style(Style::default().fg(Color::Yellow).bold())
                .block(
                    Block::default()
                        .title(" 2. Start (... ❯ Ancestor ❯ Leaf) ")
                        .borders(Borders::ALL),
                );
            let mut r2 = render_rect;
            r2.y = chunks[3].y;
            frame.render_widget(start_w, r2);

            // 5. Strategy 3: ShortenNames
            let shorten_w = Breadcrumb::new(app.items.clone())
                .separator(BreadcrumbSeparator::chevron())
                .strategy(TruncateStrategy::shorten_names())
                .item_style(Style::default().fg(Color::Cyan))
                .active_style(Style::default().fg(Color::Yellow).bold())
                .block(
                    Block::default()
                        .title(" 3. ShortenNames (H ❯ W ❯ r ❯ Leaf) ")
                        .borders(Borders::ALL),
                );
            let mut r3 = render_rect;
            r3.y = chunks[4].y;
            frame.render_widget(shorten_w, r3);

            // 6. Strategy 4: End
            let end_w = Breadcrumb::new(app.items.clone())
                .separator(BreadcrumbSeparator::chevron())
                .strategy(TruncateStrategy::end())
                .item_style(Style::default().fg(Color::Cyan))
                .active_style(Style::default().fg(Color::Yellow).bold())
                .block(
                    Block::default()
                        .title(" 4. End (Root ❯ Child ❯ ...) ")
                        .borders(Borders::ALL),
                );
            let mut r4 = render_rect;
            r4.y = chunks[5].y;
            frame.render_widget(end_w, r4);

            // 7. Strategy 5: None
            let none_w = Breadcrumb::new(app.items.clone())
                .separator(BreadcrumbSeparator::chevron())
                .strategy(TruncateStrategy::none())
                .item_style(Style::default().fg(Color::Cyan))
                .active_style(Style::default().fg(Color::Yellow).bold())
                .block(
                    Block::default()
                        .title(" 5. None (Strict Boundary Clip) ")
                        .borders(Borders::ALL),
                );
            let mut r5 = render_rect;
            r5.y = chunks[6].y;
            frame.render_widget(none_w, r5);

            // Footer
            let footer = Paragraph::new(vec![Line::from(vec![
                Span::styled(" [←/→/h/l] ", Style::default().bold().fg(Color::Yellow)),
                Span::raw("Step width ±1 col   "),
                Span::styled(" [+/-/]/[] ", Style::default().bold().fg(Color::Yellow)),
                Span::raw("Step width ±5 cols   "),
                Span::styled(" [q/Esc] ", Style::default().bold().fg(Color::Yellow)),
                Span::raw("Quit"),
            ])])
            .block(Block::default().title(" Controls ").borders(Borders::ALL));
            frame.render_widget(footer, chunks[7]);
        })?;

        if event::poll(Duration::from_millis(100))?
            && let Event::Key(key) = event::read()?
            && key.kind == KeyEventKind::Press
        {
            match key.code {
                KeyCode::Char('q') | KeyCode::Esc => break,
                KeyCode::Left | KeyCode::Char('h') => {
                    app.width = app.width.saturating_sub(1).max(12);
                }
                KeyCode::Right | KeyCode::Char('l') => {
                    app.width = (app.width + 1).min(110);
                }
                KeyCode::Char('-') | KeyCode::Char('[') => {
                    app.width = app.width.saturating_sub(5).max(12);
                }
                KeyCode::Char('+') | KeyCode::Char('=') | KeyCode::Char(']') => {
                    app.width = (app.width + 5).min(110);
                }
                _ => {}
            }
        }
    }

    ratatui::restore();
    Ok(())
}
