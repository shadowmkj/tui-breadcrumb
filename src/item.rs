// ==============================================================================
// Breadcrumb Item Representation
// ==============================================================================

//! Segment item definitions for breadcrumb navigation trails.
//!
//! A breadcrumb trail is composed of individual [`BreadcrumbItem`] elements.
//! Each item holds a styled text label, optional individual item styles,
//! and metadata indicating if an ancestor dropdown indicator is available.

use ratatui::style::Style;
use ratatui::text::{Line, Span};
use std::borrow::Cow;
use unicode_width::UnicodeWidthStr;

/// An individual segment in a breadcrumb trail.
///
/// Each item can have custom styling, active/selected styling, and can optionally
/// display a dropdown indicator (e.g. `▾`) to signal ancestor navigation sub-menus.
///
/// # Examples
///
/// ```rust
/// use tui_breadcrumb::BreadcrumbItem;
/// use ratatui::style::{Color, Style};
///
/// // Create from simple string slice
/// let item1 = BreadcrumbItem::new("Projects");
///
/// // Create with dropdown indicator and custom styling
/// let item2 = BreadcrumbItem::new("ratatui")
///     .dropdown(true)
///     .style(Style::default().fg(Color::Cyan));
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BreadcrumbItem<'a> {
    /// The text content and visual spans of this breadcrumb segment.
    pub label: Line<'a>,
    /// Optional specific style override for this item when not selected.
    pub style: Option<Style>,
    /// Optional specific style override for this item when focused / selected.
    pub selected_style: Option<Style>,
    /// Whether this ancestor item supports or displays a dropdown menu indicator.
    pub has_dropdown: bool,
    /// Optional custom dropdown symbol override for this segment (e.g. `▾`, `⌄`, `▼`).
    pub dropdown_symbol: Option<Cow<'a, str>>,
}

impl<'a> BreadcrumbItem<'a> {
    /// Creates a new [`BreadcrumbItem`] with the given label.
    ///
    /// The label can be any type that converts into a Ratatui [`Line`],
    /// such as `&str`, `String`, [`Span`], or [`Line`].
    pub fn new<T>(label: T) -> Self
    where
        T: Into<Line<'a>>,
    {
        Self {
            label: label.into(),
            style: None,
            selected_style: None,
            has_dropdown: false,
            dropdown_symbol: None,
        }
    }

    /// Creates a new [`BreadcrumbItem`] pre-configured with a dropdown indicator.
    pub fn with_dropdown<T>(label: T) -> Self
    where
        T: Into<Line<'a>>,
    {
        Self {
            label: label.into(),
            style: None,
            selected_style: None,
            has_dropdown: true,
            dropdown_symbol: None,
        }
    }

    /// Sets an explicit default style for this item.
    #[must_use]
    pub fn style(mut self, style: impl Into<Style>) -> Self {
        self.style = Some(style.into());
        self
    }

    /// Sets an explicit selected / focused style for this item.
    #[must_use]
    pub fn selected_style(mut self, style: impl Into<Style>) -> Self {
        self.selected_style = Some(style.into());
        self
    }

    /// Sets whether this breadcrumb item displays a dropdown indicator.
    #[must_use]
    pub fn dropdown(mut self, has_dropdown: bool) -> Self {
        self.has_dropdown = has_dropdown;
        self
    }

    /// Sets a custom dropdown symbol for this item (e.g. `"▾"`, `"⌄"`).
    #[must_use]
    pub fn dropdown_symbol(mut self, symbol: impl Into<Cow<'a, str>>) -> Self {
        self.dropdown_symbol = Some(symbol.into());
        self
    }

    /// Calculates the visual terminal width (in columns) of this item's label.
    #[must_use]
    pub fn label_width(&self) -> usize {
        self.label.width()
    }

    /// Calculates the total visual terminal width of this item including its optional dropdown symbol.
    #[must_use]
    pub fn total_width(&self, default_dropdown_symbol: &str) -> usize {
        let base_width = self.label_width();
        if self.has_dropdown {
            let symbol = self
                .dropdown_symbol
                .as_deref()
                .unwrap_or(default_dropdown_symbol);
            // Label + 1 column space + dropdown symbol width
            base_width + 1 + UnicodeWidthStr::width(symbol)
        } else {
            base_width
        }
    }
}

impl<'a> From<Line<'a>> for BreadcrumbItem<'a> {
    fn from(line: Line<'a>) -> Self {
        Self::new(line)
    }
}

impl<'a> From<Span<'a>> for BreadcrumbItem<'a> {
    fn from(span: Span<'a>) -> Self {
        Self::new(Line::from(span))
    }
}

impl<'a> From<&'a str> for BreadcrumbItem<'a> {
    fn from(s: &'a str) -> Self {
        Self::new(Line::from(s))
    }
}

impl From<String> for BreadcrumbItem<'static> {
    fn from(s: String) -> Self {
        Self::new(Line::from(s))
    }
}

impl<'a> From<(&'a str, Style)> for BreadcrumbItem<'a> {
    fn from((s, style): (&'a str, Style)) -> Self {
        Self::new(Line::styled(s, style)).style(style)
    }
}

impl From<(String, Style)> for BreadcrumbItem<'static> {
    fn from((s, style): (String, Style)) -> Self {
        Self::new(Line::styled(s, style)).style(style)
    }
}

impl<'a> From<(Span<'a>, Style)> for BreadcrumbItem<'a> {
    fn from((span, style): (Span<'a>, Style)) -> Self {
        Self::new(Line::from(span)).style(style)
    }
}

impl<'a> From<(Line<'a>, Style)> for BreadcrumbItem<'a> {
    fn from((line, style): (Line<'a>, Style)) -> Self {
        Self::new(line).style(style)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::style::Color;

    #[test]
    fn test_breadcrumb_item_creation() {
        let item = BreadcrumbItem::new("Home");
        assert_eq!(item.label_width(), 4);
        assert_eq!(item.total_width("▾"), 4);
        assert!(!item.has_dropdown);

        let item_dropdown = BreadcrumbItem::with_dropdown("src");
        assert_eq!(item_dropdown.label_width(), 3);
        // "src" (3) + " " (1) + "▾" (1) = 5
        assert_eq!(item_dropdown.total_width("▾"), 5);
        assert!(item_dropdown.has_dropdown);
    }

    #[test]
    fn test_breadcrumb_item_custom_dropdown() {
        let item = BreadcrumbItem::new("config")
            .dropdown(true)
            .dropdown_symbol("▼");
        assert_eq!(item.label_width(), 6);
        // "config" (6) + 1 + 1 = 8
        assert_eq!(item.total_width("▾"), 8);
    }

    #[test]
    fn test_breadcrumb_item_from_conversions() {
        let item_str: BreadcrumbItem = "test".into();
        assert_eq!(item_str.label_width(), 4);

        let item_string: BreadcrumbItem = String::from("workspace").into();
        assert_eq!(item_string.label_width(), 9);

        let item_styled: BreadcrumbItem = ("styled", Style::default().fg(Color::Yellow)).into();
        assert_eq!(item_styled.label_width(), 6);
        assert_eq!(item_styled.style, Some(Style::default().fg(Color::Yellow)));

        let span = Span::raw("span_test");
        let item_span: BreadcrumbItem = span.into();
        assert_eq!(item_span.label_width(), 9);
    }
}
