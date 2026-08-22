//! Minimal Quickstart Example for `tui-breadcrumb`

use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph};
use std::error::Error;
use tui_breadcrumb::{Breadcrumb, BreadcrumbSeparator};

fn main() -> Result<(), Box<dyn Error>> {
    let mut terminal = ratatui::init();

    terminal.draw(|frame| {
        let area = frame.area();
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(0)])
            .split(area);

        let breadcrumbs = Breadcrumb::new(["Home", "Projects", "ratatui", "src", "main.rs"])
            .separator(BreadcrumbSeparator::chevron())
            .active_style(Style::default().fg(Color::Yellow).bold())
            .block(
                Block::default()
                    .title(" Breadcrumb Trail ")
                    .borders(Borders::ALL),
            );

        frame.render_widget(breadcrumbs, chunks[0]);

        let help =
            Paragraph::new("Press any key to exit.").style(Style::default().fg(Color::DarkGray));
        frame.render_widget(help, chunks[1]);
    })?;

    use ratatui::crossterm::event::{self, Event, KeyEventKind};

    // Wait for any key press before exiting
    loop {
        if let Event::Key(key) = event::read()?
            && key.kind == KeyEventKind::Press
        {
            break;
        }
    }

    ratatui::restore();
    Ok(())
}
