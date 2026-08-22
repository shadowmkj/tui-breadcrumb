//! Custom Styling & Themes Gallery for `tui-breadcrumb`
//!
//! Demonstrates advanced styling capabilities, custom Unicode / Powerline glyphs,
//! per-item span formatting, and unique separator presets.

use std::error::Error;
use std::time::Duration;

use ratatui::crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph};

use tui_breadcrumb::{Breadcrumb, BreadcrumbItem, BreadcrumbSeparator, TruncateStrategy};

fn main() -> Result<(), Box<dyn Error>> {
    let mut terminal = ratatui::init();

    loop {
        terminal.draw(|frame| {
            let area = frame.area();

            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3), // Header
                    Constraint::Length(4), // Theme 1: Powerline Pill Theme
                    Constraint::Length(4), // Theme 2: CI/CD Pipeline Theme
                    Constraint::Length(4), // Theme 3: Retro Terminal Theme
                    Constraint::Length(4), // Theme 4: Minimal Dots Theme
                    Constraint::Min(2),    // Footer
                ])
                .split(area);

            // Header
            let header = Paragraph::new(Line::from(vec![
                Span::styled(
                    " 🎨 tui-breadcrumb ",
                    Style::default().fg(Color::Black).bg(Color::Cyan).bold(),
                ),
                Span::raw(" — Custom Themes & Advanced Styling Gallery"),
            ]))
            .block(Block::default().borders(Borders::ALL));
            frame.render_widget(header, chunks[0]);

            // ==============================================================
            // Theme 1: Powerline Pill Theme
            // ==============================================================
            let powerline_items = vec![
                BreadcrumbItem::new(Line::from(vec![Span::styled(
                    " 🏠 Root ",
                    Style::default().bg(Color::Blue).fg(Color::White).bold(),
                )])),
                BreadcrumbItem::new(Line::from(vec![Span::styled(
                    " 📦 packages ",
                    Style::default().bg(Color::DarkGray).fg(Color::White),
                )])),
                BreadcrumbItem::new(Line::from(vec![Span::styled(
                    " 🦀 ratatui ",
                    Style::default().bg(Color::DarkGray).fg(Color::Cyan),
                )])),
                BreadcrumbItem::new(Line::from(vec![Span::styled(
                    " 🚀 v0.30.2 ",
                    Style::default().bg(Color::Green).fg(Color::Black).bold(),
                )])),
            ];

            let powerline_widget = Breadcrumb::new(powerline_items)
                .separator(
                    BreadcrumbSeparator::custom("")
                        .spacing(0)
                        .style(Style::default().fg(Color::Blue)),
                )
                .block(
                    Block::default()
                        .title(" 1. Powerline Pill Theme ")
                        .borders(Borders::ALL),
                );
            frame.render_widget(powerline_widget, chunks[1]);

            // ==============================================================
            // Theme 2: CI / DevOps Pipeline Theme
            // ==============================================================
            let pipeline_items = vec![
                BreadcrumbItem::new(Line::from(vec![
                    Span::styled("✔ ", Style::default().fg(Color::Green).bold()),
                    Span::styled("Build", Style::default().fg(Color::White)),
                ])),
                BreadcrumbItem::new(Line::from(vec![
                    Span::styled("✔ ", Style::default().fg(Color::Green).bold()),
                    Span::styled("Unit Tests", Style::default().fg(Color::White)),
                ])),
                BreadcrumbItem::new(Line::from(vec![
                    Span::styled("⚡ ", Style::default().fg(Color::Yellow).bold()),
                    Span::styled(
                        "Integration Tests",
                        Style::default().fg(Color::Yellow).bold(),
                    ),
                ])),
                BreadcrumbItem::new(Line::from(vec![
                    Span::styled("⏳ ", Style::default().fg(Color::DarkGray)),
                    Span::styled("Deploy to Prod", Style::default().fg(Color::DarkGray)),
                ])),
            ];

            let pipeline_widget = Breadcrumb::new(pipeline_items)
                .separator(
                    BreadcrumbSeparator::chevron()
                        .spacing(1)
                        .style(Style::default().fg(Color::DarkGray)),
                )
                .block(
                    Block::default()
                        .title(" 2. CI/CD Pipeline Workflow ")
                        .borders(Borders::ALL),
                );
            frame.render_widget(pipeline_widget, chunks[2]);

            // ==============================================================
            // Theme 3: Retro Phosphor Terminal Theme
            // ==============================================================
            let retro_items = vec![
                BreadcrumbItem::new("SYSTEM"),
                BreadcrumbItem::new("DRIVE_C"),
                BreadcrumbItem::new("USERS"),
                BreadcrumbItem::new("ADMIN"),
                BreadcrumbItem::new("CONFIG.SYS"),
            ];

            let retro_widget = Breadcrumb::new(retro_items)
                .separator(
                    BreadcrumbSeparator::custom("//")
                        .spacing(1)
                        .style(Style::default().fg(Color::Green)),
                )
                .item_style(Style::default().fg(Color::Green))
                .active_style(Style::default().fg(Color::Black).bg(Color::Green).bold())
                .block(
                    Block::default()
                        .title(" 3. Retro Phosphor Amber/Green ")
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(Color::Green)),
                );
            frame.render_widget(retro_widget, chunks[3]);

            // ==============================================================
            // Theme 4: Minimal Dots Theme
            // ==============================================================
            let minimal_items = vec![
                BreadcrumbItem::new("App"),
                BreadcrumbItem::new("Settings"),
                BreadcrumbItem::new("Account"),
                BreadcrumbItem::new("Security"),
                BreadcrumbItem::new("Two-Factor Auth"),
            ];

            let minimal_widget = Breadcrumb::new(minimal_items)
                .separator(
                    BreadcrumbSeparator::custom("•")
                        .spacing(1)
                        .style(Style::default().fg(Color::Magenta)),
                )
                .strategy(TruncateStrategy::middle())
                .item_style(Style::default().fg(Color::DarkGray))
                .active_style(Style::default().fg(Color::Magenta).bold().underlined())
                .block(
                    Block::default()
                        .title(" 4. Minimal Bullet Theme ")
                        .borders(Borders::ALL),
                );
            frame.render_widget(minimal_widget, chunks[4]);

            // Footer
            let footer = Paragraph::new("Press [q] or [Esc] to exit.")
                .style(Style::default().fg(Color::DarkGray));
            frame.render_widget(footer, chunks[5]);
        })?;

        if event::poll(Duration::from_millis(100))?
            && let Event::Key(key) = event::read()?
            && key.kind == KeyEventKind::Press
            && matches!(key.code, KeyCode::Char('q') | KeyCode::Esc)
        {
            break;
        }
    }

    ratatui::restore();
    Ok(())
}
