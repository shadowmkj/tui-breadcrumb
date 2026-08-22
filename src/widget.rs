// ==============================================================================
// Breadcrumb Ratatui Widget Implementation
// ==============================================================================

//! The primary [`Breadcrumb`] widget implementing Ratatui's [`Widget`] and [`StatefulWidget`].
//!
//! Provides a rich builder API for configuring separators, truncation strategies,
//! ancestor dropdown glyphs, styling, alignment, and optional surrounding [`Block`]s.

use std::borrow::Cow;
use std::path::Path;

use ratatui::buffer::Buffer;
use ratatui::layout::{Alignment, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::widgets::{Block, StatefulWidget, Widget};
use unicode_width::UnicodeWidthStr;

use crate::item::BreadcrumbItem;
use crate::path::from_path;
use crate::separator::BreadcrumbSeparator;
use crate::state::BreadcrumbState;
use crate::strategy::TruncateStrategy;
use crate::truncate::{RenderElement, resolve_layout};

/// A hierarchical breadcrumb navigation trail widget for Ratatui.
///
/// Supports built-in separator glyph presets, responsive overflow truncation strategies,
/// and interactive state tracking with keyboard and mouse hit detection.
///
/// # Examples
///
/// ```rust
/// use tui_breadcrumbs::{Breadcrumb, BreadcrumbSeparator, TruncateStrategy};
/// use ratatui::style::{Color, Style};
///
/// // Stateless widget
/// let widget = Breadcrumb::new(["Home", "Projects", "ratatui", "src"])
///     .separator(BreadcrumbSeparator::chevron())
///     .strategy(TruncateStrategy::middle());
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct Breadcrumb<'a> {
    pub(crate) items: Vec<BreadcrumbItem<'a>>,
    pub(crate) separator: BreadcrumbSeparator<'a>,
    pub(crate) strategy: TruncateStrategy,
    pub(crate) style: Style,
    pub(crate) item_style: Style,
    pub(crate) selected_style: Style,
    pub(crate) active_style: Option<Style>,
    pub(crate) dropdown_style: Style,
    pub(crate) dropdown_symbol: Cow<'a, str>,
    pub(crate) ellipsis_style: Style,
    pub(crate) block: Option<Block<'a>>,
    pub(crate) alignment: Alignment,
}

impl<'a> Default for Breadcrumb<'a> {
    fn default() -> Self {
        Self {
            items: Vec::new(),
            separator: BreadcrumbSeparator::chevron(),
            strategy: TruncateStrategy::middle(),
            style: Style::default(),
            item_style: Style::default(),
            selected_style: Style::default().add_modifier(Modifier::REVERSED),
            active_style: Some(Style::default().add_modifier(Modifier::BOLD)),
            dropdown_style: Style::default().add_modifier(Modifier::DIM),
            dropdown_symbol: Cow::Borrowed("▾"),
            ellipsis_style: Style::default().add_modifier(Modifier::DIM),
            block: None,
            alignment: Alignment::Left,
        }
    }
}

impl<'a> Breadcrumb<'a> {
    /// Creates a new [`Breadcrumb`] widget from an iterator of items.
    ///
    /// Items can be `&str`, `String`, [`ratatui::text::Span`], [`ratatui::text::Line`],
    /// or [`BreadcrumbItem`].
    pub fn new<I, T>(items: I) -> Self
    where
        I: IntoIterator<Item = T>,
        T: Into<BreadcrumbItem<'a>>,
    {
        let items = items.into_iter().map(Into::into).collect();
        Self {
            items,
            ..Default::default()
        }
    }

    /// Creates a [`Breadcrumb`] widget populated from a standard filesystem [`Path`].
    pub fn from_path<P: AsRef<Path>>(path: P) -> Breadcrumb<'static> {
        let items = from_path(path);
        Breadcrumb {
            items,
            separator: BreadcrumbSeparator::slash(),
            ..Default::default()
        }
    }

    /// Sets the separator to use between breadcrumb segments.
    #[must_use]
    pub fn separator(mut self, separator: BreadcrumbSeparator<'a>) -> Self {
        self.separator = separator;
        self
    }

    /// Sets the overflow / truncation strategy.
    #[must_use]
    pub fn strategy(mut self, strategy: TruncateStrategy) -> Self {
        self.strategy = strategy;
        self
    }

    /// Sets the base style for the entire widget.
    #[must_use]
    pub fn style(mut self, style: impl Into<Style>) -> Self {
        self.style = style.into();
        self
    }

    /// Sets the default style for unselected item segments.
    #[must_use]
    pub fn item_style(mut self, style: impl Into<Style>) -> Self {
        self.item_style = style.into();
        self
    }

    /// Sets the style for focused / selected item segments.
    #[must_use]
    pub fn selected_style(mut self, style: impl Into<Style>) -> Self {
        self.selected_style = style.into();
        self
    }

    /// Sets the style for the active (deepest / last) item segment.
    #[must_use]
    pub fn active_style(mut self, style: impl Into<Style>) -> Self {
        self.active_style = Some(style.into());
        self
    }

    /// Sets the default symbol string used for ancestor dropdown indicators (e.g. `"▾"`, `"▼"`).
    #[must_use]
    pub fn dropdown_symbol(mut self, symbol: impl Into<Cow<'a, str>>) -> Self {
        self.dropdown_symbol = symbol.into();
        self
    }

    /// Sets the style for dropdown indicator glyphs.
    #[must_use]
    pub fn dropdown_style(mut self, style: impl Into<Style>) -> Self {
        self.dropdown_style = style.into();
        self
    }

    /// Sets the style for collapsed ellipsis indicators (`...`).
    #[must_use]
    pub fn ellipsis_style(mut self, style: impl Into<Style>) -> Self {
        self.ellipsis_style = style.into();
        self
    }

    /// Wraps the breadcrumb widget in an optional Ratatui [`Block`].
    #[must_use]
    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);
        self
    }

    /// Sets horizontal text alignment ([`Alignment::Left`], [`Alignment::Center`], [`Alignment::Right`]).
    #[must_use]
    pub fn alignment(mut self, alignment: Alignment) -> Self {
        self.alignment = alignment;
        self
    }

    /// Internal rendering routine shared by [`Widget`] and [`StatefulWidget`].
    fn render_breadcrumb(
        &self,
        area: Rect,
        buf: &mut Buffer,
        mut state: Option<&mut BreadcrumbState>,
    ) {
        if area.width == 0 || area.height == 0 {
            if let Some(s) = state.as_deref_mut() {
                s.reset_cache(area);
            }
            return;
        }

        // Apply surrounding block if present
        let inner_area = match &self.block {
            Some(block) => {
                block.clone().render(area, buf);
                block.inner(area)
            }
            None => area,
        };

        if inner_area.width == 0 || inner_area.height == 0 {
            if let Some(s) = state.as_deref_mut() {
                s.reset_cache(inner_area);
            }
            return;
        }

        if let Some(s) = state.as_deref_mut() {
            s.reset_cache(inner_area);
        }

        let sep_width = self.separator.total_width();
        let elements = resolve_layout(
            &self.items,
            sep_width,
            inner_area.width as usize,
            &self.strategy,
            &self.dropdown_symbol,
        );

        if elements.is_empty() {
            return;
        }

        // Compute total width of resolved elements
        let total_layout_width: usize = elements
            .iter()
            .map(|elem| match elem {
                RenderElement::Separator => sep_width,
                RenderElement::Ellipsis { text } => UnicodeWidthStr::width(text.as_str()),
                RenderElement::Item {
                    index,
                    abbreviated_label,
                } => {
                    let item = &self.items[*index];
                    let label_w = match abbreviated_label {
                        Some(abbrev) => UnicodeWidthStr::width(abbrev.as_str()),
                        None => item.label_width(),
                    };
                    if item.has_dropdown {
                        let sym = item
                            .dropdown_symbol
                            .as_deref()
                            .unwrap_or(&self.dropdown_symbol);
                        label_w + 1 + UnicodeWidthStr::width(sym)
                    } else {
                        label_w
                    }
                }
            })
            .sum();

        // Calculate starting X based on alignment
        let start_x = match self.alignment {
            Alignment::Left => inner_area.x,
            Alignment::Center => {
                let available = inner_area.width as usize;
                let offset = available.saturating_sub(total_layout_width) / 2;
                inner_area.x.saturating_add(offset as u16)
            }
            Alignment::Right => {
                let available = inner_area.width as usize;
                let offset = available.saturating_sub(total_layout_width);
                inner_area.x.saturating_add(offset as u16)
            }
        };

        let row = inner_area.y;
        let mut curr_x = start_x;
        let max_x = inner_area.x.saturating_add(inner_area.width);
        let n_items = self.items.len();

        let selected_idx = state.as_ref().and_then(|s| s.selected);

        for elem in elements {
            if curr_x >= max_x {
                break;
            }

            match elem {
                RenderElement::Separator => {
                    // Left spacing
                    for _ in 0..self.separator.spacing {
                        if curr_x < max_x {
                            if let Some(cell) = buf.cell_mut((curr_x, row)) {
                                cell.set_char(' ').set_style(self.separator.style);
                            }
                            curr_x += 1;
                        }
                    }
                    // Symbol
                    for ch in self.separator.symbol.chars() {
                        if curr_x < max_x {
                            if let Some(cell) = buf.cell_mut((curr_x, row)) {
                                cell.set_char(ch).set_style(self.separator.style);
                            }
                            curr_x += 1;
                        }
                    }
                    // Right spacing
                    for _ in 0..self.separator.spacing {
                        if curr_x < max_x {
                            if let Some(cell) = buf.cell_mut((curr_x, row)) {
                                cell.set_char(' ').set_style(self.separator.style);
                            }
                            curr_x += 1;
                        }
                    }
                }
                RenderElement::Ellipsis { text } => {
                    let elem_x = curr_x;
                    let elem_w = (UnicodeWidthStr::width(text.as_str()) as u16)
                        .min(max_x.saturating_sub(curr_x));

                    for ch in text.chars() {
                        if curr_x < max_x {
                            if let Some(cell) = buf.cell_mut((curr_x, row)) {
                                cell.set_char(ch).set_style(self.ellipsis_style);
                            }
                            curr_x += 1;
                        }
                    }

                    if let Some(s) = state.as_deref_mut() {
                        s.ellipsis_rect = Some(Rect::new(elem_x, row, elem_w, 1));
                    }
                }
                RenderElement::Item {
                    index,
                    abbreviated_label,
                } => {
                    let item = &self.items[index];
                    let is_selected = selected_idx == Some(index);
                    let is_last = index + 1 == n_items;

                    // Resolve base item style
                    let item_style = if is_selected {
                        item.selected_style.unwrap_or(self.selected_style)
                    } else if is_last {
                        self.active_style.or(item.style).unwrap_or(self.item_style)
                    } else {
                        item.style.unwrap_or(self.item_style)
                    };

                    let elem_x = curr_x;

                    // Render label (abbreviated or full)
                    match abbreviated_label {
                        Some(abbrev) => {
                            for ch in abbrev.chars() {
                                if curr_x < max_x {
                                    if let Some(cell) = buf.cell_mut((curr_x, row)) {
                                        cell.set_char(ch).set_style(item_style);
                                    }
                                    curr_x += 1;
                                }
                            }
                        }
                        None => {
                            for span in &item.label.spans {
                                let span_style = if is_selected {
                                    span.style.patch(item_style)
                                } else {
                                    item_style.patch(span.style)
                                };
                                for ch in span.content.chars() {
                                    if curr_x < max_x {
                                        if let Some(cell) = buf.cell_mut((curr_x, row)) {
                                            cell.set_char(ch).set_style(span_style);
                                        }
                                        curr_x += 1;
                                    }
                                }
                            }
                        }
                    }

                    let label_w = curr_x.saturating_sub(elem_x);
                    if let Some(s) = state.as_deref_mut() {
                        s.item_rects
                            .push((index, Rect::new(elem_x, row, label_w, 1)));
                    }

                    // Render optional dropdown indicator
                    if item.has_dropdown && curr_x < max_x {
                        // 1 column space
                        if let Some(cell) = buf.cell_mut((curr_x, row)) {
                            cell.set_char(' ').set_style(self.dropdown_style);
                        }
                        curr_x += 1;

                        let drop_x = curr_x;
                        let sym = item
                            .dropdown_symbol
                            .as_deref()
                            .unwrap_or(&self.dropdown_symbol);

                        for ch in sym.chars() {
                            if curr_x < max_x {
                                if let Some(cell) = buf.cell_mut((curr_x, row)) {
                                    cell.set_char(ch).set_style(self.dropdown_style);
                                }
                                curr_x += 1;
                            }
                        }

                        let drop_w = curr_x.saturating_sub(drop_x);
                        if let Some(s) = state.as_deref_mut() {
                            s.dropdown_rects
                                .push((index, Rect::new(drop_x, row, drop_w, 1)));
                        }
                    }
                }
            }
        }
    }
}

impl<'a> Widget for Breadcrumb<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        self.render_breadcrumb(area, buf, None);
    }
}

impl<'a> StatefulWidget for Breadcrumb<'a> {
    type State = BreadcrumbState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        self.render_breadcrumb(area, buf, Some(state));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::buffer::Buffer;

    #[test]
    fn test_widget_render_stateless() {
        let widget = Breadcrumb::new(["Home", "Projects", "ratatui"])
            .separator(BreadcrumbSeparator::slash().spacing(1));

        let mut buffer = Buffer::empty(Rect::new(0, 0, 30, 1));
        Widget::render(widget, Rect::new(0, 0, 30, 1), &mut buffer);

        let content = (0..30).map(|x| buffer[(x, 0)].symbol()).collect::<String>();
        assert!(content.starts_with("Home / Projects / ratatui"));
    }

    #[test]
    fn test_widget_render_stateful() {
        let widget = Breadcrumb::new(["Home", "Projects", "ratatui"])
            .separator(BreadcrumbSeparator::chevron());

        let mut state = BreadcrumbState::default();
        state.select(Some(1));

        let mut buffer = Buffer::empty(Rect::new(0, 0, 30, 1));
        StatefulWidget::render(widget, Rect::new(0, 0, 30, 1), &mut buffer, &mut state);

        assert_eq!(state.item_rects.len(), 3);
        assert_eq!(state.item_at(0, 0), Some(0));
        assert_eq!(state.item_at(7, 0), Some(1));
    }
}
