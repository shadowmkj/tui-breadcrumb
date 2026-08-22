<div align="center">

# 🍞 `tui-breadcrumbs`

**A customizable, responsive, and interactive hierarchical navigation trail widget for [Ratatui](https://crates.io/crates/ratatui).**

[![Crates.io](https://img.shields.io/crates/v/tui-breadcrumbs.svg?style=flat-square)](https://crates.io/crates/tui-breadcrumbs)
[![Documentation](https://docs.rs/tui-breadcrumbs/badge.svg?style=flat-square)](https://docs.rs/tui-breadcrumbs)
[![Downloads](https://img.shields.io/crates/d/tui-breadcrumbs.svg?style=flat-square)](https://crates.io/crates/tui-breadcrumbs)
[![CI](https://img.shields.io/github/actions/workflow/status/shadowmkj/tui-breadcrumbs/ci.yml?branch=main&style=flat-square&label=CI)](https://github.com/shadowmkj/tui-breadcrumbs/actions/workflows/ci.yml)
[![Codecov](https://img.shields.io/codecov/c/gh/shadowmkj/tui-breadcrumbs?style=flat-square&logo=codecov)](https://codecov.io/gh/shadowmkj/tui-breadcrumbs)
[![Release](https://img.shields.io/github/v/release/shadowmkj/tui-breadcrumbs?style=flat-square&include_prereleases)](https://github.com/shadowmkj/tui-breadcrumbs/releases)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg?style=flat-square)](LICENSE)
[![Ratatui](https://img.shields.io/badge/ratatui-v0.30+-purple.svg?style=flat-square)](https://crates.io/crates/ratatui)

```text
┌─────────────────────────────────────────────────────────────────────────────┐
│ 📁 Home ❯ 📂 Projects ❯ 📦 ratatui ▾ ❯ 📄 sparkline.rs                     │
│    ▲          ▲             ▲                ▲                              │
│  Root      Parent      Ancestor with       Active                           │
│  Segment   Segment     Dropdown            Item                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

</div>

---

## 📖 Overview

Terminal user interfaces—such as file managers, cloud resource consoles, database browsers, and nested configuration screens—frequently require breadcrumb navigation paths. Without a dedicated widget, developers often manually assemble `Line` and `Span` collections and write custom string truncations that break on Unicode characters or fail to support mouse interaction.

`tui-breadcrumbs` solves this by providing:
- **Built-in Separator Presets**: Slash (`/`), Chevron (`❯`), Angle (`›`), Arrow (`→`), Pipe (`|`), Backslash (`\\`), Double Angle (`»`), or custom glyphs.
- **Smart Responsive Truncation**: Intelligently fits breadcrumbs to any terminal column width using strategies like `Middle`, `Start`, `ShortenNames`, `End`, or `None`.
- **Full Interactivity (`BreadcrumbState`)**: Keyboard focus navigation (`Left`/`Right` arrows) and pixel-accurate mouse hit testing for segment clicks and ancestor dropdown triggers (`▾`).
- **Filesystem Path Integration**: Seamless conversions from standard `std::path::Path`.
- **Zero Panics & Unicode Safe**: Built with `unicode-width` to prevent column drift across emojis, CJK characters, and combining glyphs.

---

## 📦 Installation

Add `tui-breadcrumbs` to your `Cargo.toml`:

```bash
cargo add tui-breadcrumbs
```

Or manually:

```toml
[dependencies]
tui-breadcrumbs = "0.1.0"
ratatui = "0.30"
```

---

## 🚀 Quick Start

### 1. Simple Stateless Breadcrumb

```rust
use ratatui::prelude::*;
use tui_breadcrumbs::{Breadcrumb, BreadcrumbSeparator, TruncateStrategy};

fn render_trail(frame: &mut Frame, area: Rect) {
    let widget = Breadcrumb::new(["Home", "Projects", "ratatui", "src", "sparkline.rs"])
        .separator(BreadcrumbSeparator::chevron())
        .strategy(TruncateStrategy::middle())
        .item_style(Style::default().fg(Color::Gray))
        .active_style(Style::default().fg(Color::Yellow).bold());

    frame.render_widget(widget, area);
}
```

### 2. Interactive Stateful Breadcrumb (Keyboard & Mouse Support)

```rust
use ratatui::prelude::*;
use ratatui::crossterm::event::{Event, KeyCode, MouseEventKind, MouseButton};
use tui_breadcrumbs::{Breadcrumb, BreadcrumbItem, BreadcrumbSeparator, BreadcrumbState, TruncateStrategy};

struct App {
    state: BreadcrumbState,
    items: Vec<BreadcrumbItem<'static>>,
}

impl App {
    fn new() -> Self {
        let mut state = BreadcrumbState::default();
        state.select(Some(2)); // Focus 3rd segment

        let items = vec![
            BreadcrumbItem::new("Home"),
            BreadcrumbItem::with_dropdown("Projects"),
            BreadcrumbItem::with_dropdown("ratatui"),
            BreadcrumbItem::new("src"),
            BreadcrumbItem::new("main.rs"),
        ];

        Self { state, items }
    }

    fn handle_event(&mut self, event: Event) {
        match event {
            Event::Key(key) => match key.code {
                KeyCode::Left => self.state.select_previous(self.items.len()),
                KeyCode::Right => self.state.select_next(self.items.len()),
                KeyCode::Home => self.state.select_first(),
                KeyCode::End => self.state.select_last(self.items.len()),
                _ => {}
            },
            Event::Mouse(mouse) if mouse.kind == MouseEventKind::Down(MouseButton::Left) => {
                let (col, row) = (mouse.column, mouse.row);
                // Check if user clicked an ancestor dropdown arrow (▾)
                if let Some(drop_idx) = self.state.dropdown_at(col, row) {
                    println!("Clicked dropdown for item index {}", drop_idx);
                }
                // Check if user clicked a crumb label
                else if let Some(item_idx) = self.state.item_at(col, row) {
                    self.state.select(Some(item_idx));
                }
            }
            _ => {}
        }
    }

    fn render(&mut self, frame: &mut Frame, area: Rect) {
        let widget = Breadcrumb::new(self.items.clone())
            .separator(BreadcrumbSeparator::chevron())
            .strategy(TruncateStrategy::middle())
            .selected_style(Style::default().bg(Color::DarkGray).fg(Color::White).bold());

        frame.render_stateful_widget(widget, area, &mut self.state);
    }
}
```

---

## 📐 Truncation Strategies

When a breadcrumb trail exceeds available terminal width, [`TruncateStrategy`] determines how segments are condensed:

| Strategy | Output Pattern | Description |
|---|---|---|
| **`Middle`** *(Default)* | `Home ❯ ... ❯ src ❯ sparkline.rs` | Preserves root context and active leaf; collapses intermediate segments into `...`. |
| **`Start`** | `... ❯ tui-breadcrumbs ❯ src ❯ sparkline.rs` | Preserves deepest active leaf and immediate parents; collapses leftmost ancestors. |
| **`ShortenNames`** | `H ❯ P ❯ ratatui ❯ src ❯ sparkline.rs` | Progressively abbreviates ancestor segment labels to single characters before collapsing. |
| **`End`** | `Home ❯ Projects ❯ ratatui ❯ ...` | Left-to-right priority; preserves root ancestors and collapses leaf segments. |
| **`None`** | `Home ❯ Projects ❯ ratatui ❯ src` | Strict clipping at the boundary without ellipsis substitution. |

```rust
// Examples of configuring strategies:
let middle = TruncateStrategy::middle_with(1, 2, "...");
let start = TruncateStrategy::start_with(2, "…");
let shorten = TruncateStrategy::shorten_names_with(1, 2, "...");
```

---

## 🎨 Separator Presets

`BreadcrumbSeparator` includes built-in glyph presets with configurable spacing and styling:

| Preset Constructor | Symbol | Visual Preview (Spacing = 1) |
|---|---|---|
| `BreadcrumbSeparator::chevron()` | `❯` | `Home ❯ Projects ❯ ratatui` |
| `BreadcrumbSeparator::slash()` | `/` | `Home / Projects / ratatui` |
| `BreadcrumbSeparator::angle()` | `›` | `Home › Projects › ratatui` |
| `BreadcrumbSeparator::arrow()` | `→` | `Home → Projects → ratatui` |
| `BreadcrumbSeparator::pipe()` | `\|` | `Home \| Projects \| ratatui` |
| `BreadcrumbSeparator::backslash()` | `\\` | `Home \\ Projects \\ ratatui` |
| `BreadcrumbSeparator::double_angle()` | `»` | `Home » Projects » ratatui` |
| `BreadcrumbSeparator::custom(sym)` | `*` | `Home * Projects * ratatui` |

```rust
// Customize separator styling and spacing:
let sep = BreadcrumbSeparator::chevron()
    .spacing(1)
    .style(Style::default().fg(Color::DarkGray));
```

---

## 📂 Filesystem Path Integration

Easily initialize a breadcrumb navigation trail directly from a [`std::path::Path`]:

```rust
use std::path::Path;
use tui_breadcrumbs::Breadcrumb;

let path = Path::new("/var/log/nginx/access.log");
let widget = Breadcrumb::from_path(path);
```

---

## 🎮 Interactive Demo

An interactive demonstration is included in `examples/demo.rs`. It showcases live responsive resizing, keyboard navigation, separator switching, strategy comparisons, and mouse dropdown triggers:

```bash
cargo run --example demo
```

### Controls in Demo:
- **`←` / `→` / `h` / `l`**: Move selection through path segments.
- **`Home` / `End`**: Jump to root / leaf segment.
- **`Tab`**: Cycle through separator presets.
- **`1` – `5`**: Switch truncation strategies (`Middle`, `Start`, `ShortenNames`, `End`, `None`).
- **`+` / `-`**: Resize container width to observe responsive truncation in real time.
- **Mouse Click**: Click any crumb segment to focus, or click `▾` dropdown triggers to open sub-menus.
- **`q` / `Esc`**: Exit demo.

---

## 🧪 Testing & Verification

`tui-breadcrumbs` includes a comprehensive verification suite:

```bash
# Run unit tests
cargo test --lib

# Run documentation tests
cargo test --doc

# Run property-based invariant tests (QuickCheck)
cargo test --test property_test

# Run visual regression snapshot tests (Insta)
cargo test --test snapshots

# Run linter and formatting checks
cargo clippy --all-targets --all-features -- -D warnings
cargo fmt --all -- --check
```

---

## 🤝 Contributing

Contributions are welcome! Please check out [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines on development setup, running tests, code standards, and submitting pull requests.

---

## 📄 License

This project is dual-licensed under:

- **MIT License** ([LICENSE-MIT](LICENSE-MIT) or <https://opensource.org/licenses/MIT>)
- **Apache License, Version 2.0** ([LICENSE-APACHE](LICENSE-APACHE) or <https://www.apache.org/licenses/LICENSE-2.0>)

You may choose either license at your option.
