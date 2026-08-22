//! # tui-breadcrumbs
//!
//! A dedicated, highly customizable, and interactive hierarchical breadcrumb navigation
//! widget for [Ratatui](https://crates.io/crates/ratatui).
//!
//! ```text
//! 📁 Home ❯ 📂 Projects ❯ 📦 ratatui ▾ ❯ 📄 sparkline.rs
//! ```
//!
//! ## Key Features
//!
//! - **Built-in Separator Presets**: Slash (`/`), Chevron (`❯`), Angle (`›`), Arrow (`→`), Pipe (`|`), Backslash (`\`), Double Angle (`»`), or custom glyphs.
//! - **Smart Responsive Truncation**:
//!   - [`TruncateStrategy::Middle`]: `Home ❯ ... ❯ src ❯ sparkline.rs`
//!   - [`TruncateStrategy::Start`]: `... ❯ src ❯ sparkline.rs`
//!   - [`TruncateStrategy::ShortenNames`]: `H ❯ P ❯ ratatui ❯ src ❯ sparkline.rs`
//!   - [`TruncateStrategy::End`]: `Home ❯ Projects ❯ ...`
//! - **Interactive State (`BreadcrumbState`)**:
//!   - Keyboard focus navigation with arrow keys.
//!   - Mouse hit testing ([`BreadcrumbState::item_at`], [`BreadcrumbState::dropdown_at`], [`BreadcrumbState::is_ellipsis_at`]).
//! - **Filesystem Path Integration**: Seamless construction from [`std::path::Path`] via [`from_path`].
//!
//! ## Quickstart Example
//!
//! ```rust
//! use ratatui::prelude::*;
//! use tui_breadcrumbs::{Breadcrumb, BreadcrumbSeparator, BreadcrumbState, TruncateStrategy};
//!
//! fn draw_breadcrumbs(frame: &mut Frame, area: Rect, state: &mut BreadcrumbState) {
//!     let widget = Breadcrumb::new(["Home", "Projects", "ratatui", "src", "sparkline.rs"])
//!         .separator(BreadcrumbSeparator::chevron())
//!         .strategy(TruncateStrategy::middle())
//!         .selected_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD));
//!
//!     frame.render_stateful_widget(widget, area, state);
//! }
//! ```

pub mod item;
pub mod path;
pub mod separator;
pub mod state;
pub mod strategy;
pub mod truncate;
pub mod widget;

pub use item::BreadcrumbItem;
pub use path::from_path;
pub use separator::BreadcrumbSeparator;
pub use state::BreadcrumbState;
pub use strategy::TruncateStrategy;
pub use widget::Breadcrumb;
