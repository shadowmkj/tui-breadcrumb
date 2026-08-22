// ==============================================================================
// Breadcrumb Separator Representation & Presets
// ==============================================================================

//! Separator definitions and standard presets for breadcrumb trails.
//!
//! Separators delimit adjacent segments in the navigation path.
//! This module provides built-in glyph presets ([`BreadcrumbSeparator::chevron`],
//! [`BreadcrumbSeparator::slash`], [`BreadcrumbSeparator::angle`], etc.), custom symbols,
//! configurable horizontal spacing, and distinct styles.

use ratatui::style::Style;
use std::borrow::Cow;
use unicode_width::UnicodeWidthStr;

/// A separator rendered between breadcrumb items.
///
/// Holds the separator symbol string, styling attributes, and padding spacing
/// placed before and after the symbol glyph.
///
/// # Presets
///
/// - `BreadcrumbSeparator::chevron()`: `❯` (default spacing: 1 -> `" ❯ "`)
/// - `BreadcrumbSeparator::slash()`: `/` (default spacing: 1 -> `" / "`)
/// - `BreadcrumbSeparator::angle()`: `›` (default spacing: 1 -> `" › "`)
/// - `BreadcrumbSeparator::arrow()`: `→` (default spacing: 1 -> `" → "`)
/// - `BreadcrumbSeparator::pipe()`: `|` (default spacing: 1 -> `" | "`)
/// - `BreadcrumbSeparator::backslash()`: `\\` (default spacing: 1 -> `" \\ "`)
/// - `BreadcrumbSeparator::double_angle()`: `»` (default spacing: 1 -> `" » "`)
///
/// # Examples
///
/// ```rust
/// use tui_breadcrumb::BreadcrumbSeparator;
/// use ratatui::style::{Color, Style};
///
/// // Standard chevron with custom cyan style
/// let sep = BreadcrumbSeparator::chevron()
///     .style(Style::default().fg(Color::DarkGray));
///
/// // Unspaced slash for POSIX style path (/usr/bin/env)
/// let posix_sep = BreadcrumbSeparator::slash().spacing(0);
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BreadcrumbSeparator<'a> {
    /// The separator symbol string.
    pub symbol: Cow<'a, str>,
    /// Style applied to the separator symbol and padding spaces.
    pub style: Style,
    /// Number of blank columns placed on both the left and right sides of the symbol.
    pub spacing: u16,
}

impl Default for BreadcrumbSeparator<'static> {
    fn default() -> Self {
        Self::chevron()
    }
}

impl<'a> BreadcrumbSeparator<'a> {
    /// Creates a custom separator with the given symbol and 1 column of padding on each side.
    pub fn custom(symbol: impl Into<Cow<'a, str>>) -> Self {
        Self {
            symbol: symbol.into(),
            style: Style::default(),
            spacing: 1,
        }
    }

    /// Built-in preset for standard chevron: `❯`
    #[must_use]
    pub fn chevron() -> Self {
        Self::custom("❯")
    }

    /// Built-in preset for forward slash: `/`
    #[must_use]
    pub fn slash() -> Self {
        Self::custom("/")
    }

    /// Built-in preset for single angle quotation mark: `›`
    #[must_use]
    pub fn angle() -> Self {
        Self::custom("›")
    }

    /// Built-in preset for right arrow: `→`
    #[must_use]
    pub fn arrow() -> Self {
        Self::custom("→")
    }

    /// Built-in preset for vertical pipe: `|`
    #[must_use]
    pub fn pipe() -> Self {
        Self::custom("|")
    }

    /// Built-in preset for backward slash: `\\`
    #[must_use]
    pub fn backslash() -> Self {
        Self::custom("\\")
    }

    /// Built-in preset for double angle quotation mark: `»`
    #[must_use]
    pub fn double_angle() -> Self {
        Self::custom("»")
    }

    /// Sets the style for this separator.
    #[must_use]
    pub fn style(mut self, style: impl Into<Style>) -> Self {
        self.style = style.into();
        self
    }

    /// Sets the number of padding spaces on the left and right sides of the separator symbol.
    #[must_use]
    pub fn spacing(mut self, spacing: u16) -> Self {
        self.spacing = spacing;
        self
    }

    /// Calculates the total terminal width (columns) occupied by this separator,
    /// including left and right padding spaces.
    #[must_use]
    pub fn total_width(&self) -> usize {
        UnicodeWidthStr::width(self.symbol.as_ref()) + (self.spacing as usize * 2)
    }
}

impl<'a> From<&'a str> for BreadcrumbSeparator<'a> {
    fn from(s: &'a str) -> Self {
        Self::custom(s)
    }
}

impl From<String> for BreadcrumbSeparator<'static> {
    fn from(s: String) -> Self {
        Self::custom(s)
    }
}

impl From<char> for BreadcrumbSeparator<'static> {
    fn from(c: char) -> Self {
        Self::custom(c.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::style::Color;

    #[test]
    fn test_separator_presets_width() {
        let chevron = BreadcrumbSeparator::chevron();
        assert_eq!(chevron.symbol, "❯");
        assert_eq!(chevron.spacing, 1);
        // " ❯ " -> 1 + 1 + 1 = 3
        assert_eq!(chevron.total_width(), 3);

        let slash = BreadcrumbSeparator::slash().spacing(0);
        assert_eq!(slash.symbol, "/");
        assert_eq!(slash.spacing, 0);
        assert_eq!(slash.total_width(), 1);

        let pipe = BreadcrumbSeparator::pipe();
        assert_eq!(pipe.total_width(), 3);

        let arrow = BreadcrumbSeparator::arrow();
        assert_eq!(arrow.total_width(), 3);

        let angle = BreadcrumbSeparator::angle();
        assert_eq!(angle.total_width(), 3);

        let backslash = BreadcrumbSeparator::backslash();
        assert_eq!(backslash.total_width(), 3);

        let double_angle = BreadcrumbSeparator::double_angle();
        assert_eq!(double_angle.total_width(), 3);
    }

    #[test]
    fn test_custom_separator_styling() {
        let sep = BreadcrumbSeparator::custom("•")
            .spacing(2)
            .style(Style::default().fg(Color::Magenta));

        assert_eq!(sep.symbol, "•");
        assert_eq!(sep.spacing, 2);
        // "  •  " -> 2 + 1 + 2 = 5
        assert_eq!(sep.total_width(), 5);
        assert_eq!(sep.style, Style::default().fg(Color::Magenta));
    }
}
