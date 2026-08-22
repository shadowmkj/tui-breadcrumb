//! Dropdown Popover Menus Example for `tui-breadcrumb`
//!
//! Demonstrates:
//! - Ancestor dropdown indicators (`▾`) on deep hierarchical resource paths.
//! - Mouse hit-testing on dropdown triggers (`state.dropdown_at(col, row)`).
//! - Interactive floating popover modal (`Clear` + `List`) allowing switching sibling branches.

use std::error::Error;
use std::time::Duration;

use ratatui::crossterm::event::{
    self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind, MouseButton,
    MouseEventKind,
};
use ratatui::crossterm::execute;
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph};

use tui_breadcrumb::{
    Breadcrumb, BreadcrumbItem, BreadcrumbSeparator, BreadcrumbState, TruncateStrategy,
};

#[derive(Clone)]
struct HierarchyNode {
    name: String,
    siblings: Vec<String>,
}

struct App {
    path_nodes: Vec<HierarchyNode>,
    breadcrumb_state: BreadcrumbState,
    active_dropdown: Option<DropdownState>,
    status_message: String,
}

struct DropdownState {
    node_idx: usize,
    menu_area: Rect,
    list_state: ListState,
    options: Vec<String>,
}

impl App {
    fn new() -> Self {
        let path_nodes = vec![
            HierarchyNode {
                name: String::from("AWS Console"),
                siblings: vec![
                    String::from("AWS Console"),
                    String::from("GCP Cloud"),
                    String::from("Azure Portal"),
                ],
            },
            HierarchyNode {
                name: String::from("us-east-1"),
                siblings: vec![
                    String::from("us-east-1"),
                    String::from("us-west-2"),
                    String::from("eu-central-1"),
                    String::from("ap-northeast-1"),
                ],
            },
            HierarchyNode {
                name: String::from("Production-VPC"),
                siblings: vec![
                    String::from("Production-VPC"),
                    String::from("Staging-VPC"),
                    String::from("Dev-Sandbox-VPC"),
                ],
            },
            HierarchyNode {
                name: String::from("Security-Groups"),
                siblings: vec![
                    String::from("Security-Groups"),
                    String::from("EC2-Instances"),
                    String::from("RDS-Clusters"),
                    String::from("Subnets"),
                ],
            },
            HierarchyNode {
                name: String::from("sg-web-frontend-01"),
                siblings: vec![
                    String::from("sg-web-frontend-01"),
                    String::from("sg-api-backend-02"),
                    String::from("sg-db-postgres-03"),
                ],
            },
        ];

        Self {
            path_nodes,
            breadcrumb_state: BreadcrumbState::default(),
            active_dropdown: None,
            status_message: String::from(
                "Click any ▾ dropdown arrow to open a sibling branch popover.",
            ),
        }
    }

    fn open_dropdown(&mut self, node_idx: usize, frame_area: Rect) {
        if node_idx >= self.path_nodes.len() {
            return;
        }

        let options = self.path_nodes[node_idx].siblings.clone();
        let mut list_state = ListState::default();

        // Select current active option if found
        if let Some(pos) = options
            .iter()
            .position(|s| s == &self.path_nodes[node_idx].name)
        {
            list_state.select(Some(pos));
        } else {
            list_state.select(Some(0));
        }

        // Center popup relative to screen
        let popup_w = 34.min(frame_area.width.saturating_sub(4));
        let popup_h = (options.len() as u16 + 3).min(frame_area.height.saturating_sub(4));
        let popup_x = (frame_area.width.saturating_sub(popup_w)) / 2;
        let popup_y = (frame_area.height.saturating_sub(popup_h)) / 2;

        let menu_area = Rect::new(popup_x, popup_y, popup_w, popup_h);

        let label = self.path_nodes[node_idx].name.clone();
        self.status_message = format!("Opened sibling menu for [{}]: {}", node_idx, label);

        self.active_dropdown = Some(DropdownState {
            node_idx,
            menu_area,
            list_state,
            options,
        });
    }

    fn select_active_dropdown_choice(&mut self) {
        if let Some(drop) = &self.active_dropdown
            && let Some(sel) = drop.list_state.selected()
            && let Some(chosen_name) = drop.options.get(sel)
        {
            let node_idx = drop.node_idx;
            self.path_nodes[node_idx].name = chosen_name.clone();
            self.status_message = format!("Switched branch to: {}", chosen_name);
        }
        self.active_dropdown = None;
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
                    Constraint::Length(3), // Header
                    Constraint::Length(4), // Breadcrumb Trail
                    Constraint::Min(6),    // Resource Overview Pane
                    Constraint::Length(3), // Footer
                ])
                .split(area);

            // 1. Header
            let header = Paragraph::new(Line::from(vec![
                Span::styled(
                    " ☁ Cloud Hierarchy ",
                    Style::default().fg(Color::Black).bg(Color::Blue).bold(),
                ),
                Span::raw(" — Deep Navigation with Sibling Popover Menus"),
            ]))
            .block(Block::default().borders(Borders::ALL));
            frame.render_widget(header, chunks[0]);

            // 2. Breadcrumb Trail
            let items: Vec<BreadcrumbItem> = app
                .path_nodes
                .iter()
                .enumerate()
                .map(|(idx, node)| {
                    let mut item = BreadcrumbItem::with_dropdown(node.name.as_str());
                    if idx + 1 == app.path_nodes.len() {
                        item = item.style(Style::default().fg(Color::Yellow).bold());
                    } else {
                        item = item.style(Style::default().fg(Color::Cyan));
                    }
                    item
                })
                .collect();

            let breadcrumbs = Breadcrumb::new(items)
                .separator(
                    BreadcrumbSeparator::angle()
                        .spacing(1)
                        .style(Style::default().fg(Color::DarkGray)),
                )
                .strategy(TruncateStrategy::middle())
                .selected_style(Style::default().bg(Color::DarkGray).fg(Color::White).bold())
                .block(
                    Block::default()
                        .title(" Resource Trail (Click ▾ on any node to switch sibling branch) ")
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(Color::Blue)),
                );

            frame.render_stateful_widget(breadcrumbs, chunks[1], &mut app.breadcrumb_state);

            // 3. Resource Overview Pane
            let mut info_lines = vec![
                Line::from(vec![
                    Span::styled(
                        "Active Hierarchy: ",
                        Style::default().bold().fg(Color::Green),
                    ),
                    Span::raw(
                        app.path_nodes
                            .iter()
                            .map(|n| n.name.as_str())
                            .collect::<Vec<&str>>()
                            .join(" › "),
                    ),
                ]),
                Line::raw(""),
                Line::from(Span::styled(
                    "Available Sibling Branches per Level:",
                    Style::default().bold(),
                )),
            ];

            for (idx, node) in app.path_nodes.iter().enumerate() {
                let siblings_str = node.siblings.join(", ");
                info_lines.push(Line::from(vec![
                    Span::styled(
                        format!("  Level {}: ", idx),
                        Style::default().fg(Color::Yellow),
                    ),
                    Span::styled(&node.name, Style::default().fg(Color::White).bold()),
                    Span::styled(
                        format!(" (Siblings: {})", siblings_str),
                        Style::default().fg(Color::DarkGray),
                    ),
                ]));
            }

            let overview = Paragraph::new(info_lines).block(
                Block::default()
                    .title(" Hierarchy Details ")
                    .borders(Borders::ALL),
            );
            frame.render_widget(overview, chunks[2]);

            // 4. Footer
            let footer = Paragraph::new(vec![
                Line::from(vec![
                    Span::styled(
                        " [Click ▾ / Item] ",
                        Style::default().bold().fg(Color::Yellow),
                    ),
                    Span::raw("Open sibling dropdown   "),
                    Span::styled(" [↑/↓/Enter] ", Style::default().bold().fg(Color::Yellow)),
                    Span::raw("Navigate popup   "),
                    Span::styled(" [Esc] ", Style::default().bold().fg(Color::Yellow)),
                    Span::raw("Close popup / Quit"),
                ]),
                Line::from(vec![
                    Span::styled(" Status: ", Style::default().bold().fg(Color::Green)),
                    Span::styled(&app.status_message, Style::default().fg(Color::White)),
                ]),
            ])
            .block(Block::default().borders(Borders::ALL));
            frame.render_widget(footer, chunks[3]);

            // 5. Floating Dropdown Popover Modal
            if let Some(drop) = &mut app.active_dropdown {
                frame.render_widget(Clear, drop.menu_area);

                let list_items: Vec<ListItem> = drop
                    .options
                    .iter()
                    .map(|opt| {
                        let is_current = opt == &app.path_nodes[drop.node_idx].name;
                        let prefix = if is_current { "● " } else { "○ " };
                        let style = if is_current {
                            Style::default().fg(Color::Green).bold()
                        } else {
                            Style::default().fg(Color::White)
                        };
                        ListItem::new(format!("{}{}", prefix, opt)).style(style)
                    })
                    .collect();

                let title = format!(" Select Sibling for Level {} ", drop.node_idx);
                let list_widget = List::new(list_items)
                    .block(
                        Block::default()
                            .title(title)
                            .borders(Borders::ALL)
                            .border_style(Style::default().fg(Color::Green)),
                    )
                    .highlight_style(Style::default().bg(Color::Green).fg(Color::Black).bold())
                    .highlight_symbol("▶ ");

                frame.render_stateful_widget(list_widget, drop.menu_area, &mut drop.list_state);
            }
        })?;

        if event::poll(Duration::from_millis(100))? {
            let area = terminal.size()?;
            let frame_area = Rect::new(0, 0, area.width, area.height);

            match event::read()? {
                Event::Key(key) if key.kind == KeyEventKind::Press => match key.code {
                    KeyCode::Char('q') => {
                        if app.active_dropdown.is_some() {
                            app.active_dropdown = None;
                            app.status_message = String::from("Closed dropdown menu.");
                        } else {
                            break;
                        }
                    }
                    KeyCode::Esc => {
                        if app.active_dropdown.is_some() {
                            app.active_dropdown = None;
                            app.status_message = String::from("Closed dropdown menu.");
                        } else {
                            break;
                        }
                    }
                    KeyCode::Up | KeyCode::Char('k') => {
                        if let Some(drop) = &mut app.active_dropdown {
                            let i = match drop.list_state.selected() {
                                Some(i) => {
                                    if i == 0 {
                                        drop.options.len().saturating_sub(1)
                                    } else {
                                        i - 1
                                    }
                                }
                                None => 0,
                            };
                            drop.list_state.select(Some(i));
                        }
                    }
                    KeyCode::Down | KeyCode::Char('j') => {
                        if let Some(drop) = &mut app.active_dropdown {
                            let i = match drop.list_state.selected() {
                                Some(i) => {
                                    if i + 1 >= drop.options.len() {
                                        0
                                    } else {
                                        i + 1
                                    }
                                }
                                None => 0,
                            };
                            drop.list_state.select(Some(i));
                        }
                    }
                    KeyCode::Enter if app.active_dropdown.is_some() => {
                        app.select_active_dropdown_choice();
                    }
                    _ => {}
                },
                Event::Mouse(mouse) if mouse.kind == MouseEventKind::Down(MouseButton::Left) => {
                    let (col, row) = (mouse.column, mouse.row);

                    // If popup is open, check if clicked inside popup
                    if let Some(drop) = &app.active_dropdown {
                        let m = drop.menu_area;
                        if col >= m.x
                            && col < m.x + m.width
                            && row > m.y
                            && row <= m.y + drop.options.len() as u16
                        {
                            let clicked_idx = (row - m.y - 1) as usize;
                            if clicked_idx < drop.options.len() {
                                let mut d = app
                                    .active_dropdown
                                    .take()
                                    .expect("active_dropdown was verified Some in if-let");
                                d.list_state.select(Some(clicked_idx));
                                app.active_dropdown = Some(d);
                                app.select_active_dropdown_choice();
                                continue;
                            }
                        } else {
                            // Clicked outside popup -> dismiss
                            app.active_dropdown = None;
                            app.status_message = String::from("Dismissed popup.");
                            continue;
                        }
                    }

                    // Check if clicked dropdown indicator
                    if let Some(drop_idx) = app.breadcrumb_state.dropdown_at(col, row) {
                        app.open_dropdown(drop_idx, frame_area);
                    } else if let Some(item_idx) = app.breadcrumb_state.item_at(col, row) {
                        app.open_dropdown(item_idx, frame_area);
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
