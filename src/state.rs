// ==============================================================================
// Breadcrumb State & Interactive Hit-Testing
// ==============================================================================

//! Interactive state tracking and spatial hit-testing for breadcrumb navigation.
//!
//! [`BreadcrumbState`] tracks the currently focused/selected crumb segment,
//! mouse hover states, and caches the spatial bounding boxes ([`Rect`]) of all rendered
//! segments and ancestor dropdown indicators during each render pass.

use ratatui::layout::Rect;

/// Interactive state for the [`Breadcrumb`](crate::Breadcrumb) widget.
///
/// Keeps track of the selected crumb item index and stores the bounding boxes of rendered
/// elements to support mouse clicking, hover effects, and keyboard navigation.
///
/// # Examples
///
/// ```rust
/// use tui_breadcrumb::BreadcrumbState;
///
/// let mut state = BreadcrumbState::default();
/// state.select(Some(2));
/// assert_eq!(state.selected(), Some(2));
///
/// // Navigate forward
/// state.select_next(5);
/// assert_eq!(state.selected(), Some(3));
/// ```
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct BreadcrumbState {
    /// Currently focused or selected breadcrumb segment index.
    pub selected: Option<usize>,
    /// Currently hovered breadcrumb segment index (if mouse tracking enabled).
    pub hovered: Option<usize>,

    // Spatial bounding boxes populated during each render pass
    pub(crate) item_rects: Vec<(usize, Rect)>,
    pub(crate) dropdown_rects: Vec<(usize, Rect)>,
    pub(crate) ellipsis_rect: Option<Rect>,
    pub(crate) rendered_area: Rect,
}

impl BreadcrumbState {
    /// Creates a new, empty [`BreadcrumbState`].
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the currently selected breadcrumb segment index.
    pub fn select(&mut self, index: Option<usize>) {
        self.selected = index;
    }

    /// Returns the currently selected breadcrumb segment index, if any.
    #[must_use]
    pub fn selected(&self) -> Option<usize> {
        self.selected
    }

    /// Moves the selection to the next breadcrumb segment.
    ///
    /// If no item is selected, selects the first item (index `0`).
    /// Clamps to `total_items - 1` if already at the end.
    pub fn select_next(&mut self, total_items: usize) {
        if total_items == 0 {
            self.selected = None;
            return;
        }
        self.selected = match self.selected {
            Some(curr) => Some((curr + 1).min(total_items - 1)),
            None => Some(0),
        };
    }

    /// Moves the selection to the previous breadcrumb segment.
    ///
    /// If no item is selected, selects the last item (`total_items - 1`).
    /// Clamps to `0` if already at the beginning.
    pub fn select_previous(&mut self, total_items: usize) {
        if total_items == 0 {
            self.selected = None;
            return;
        }
        self.selected = match self.selected {
            Some(curr) => Some(curr.saturating_sub(1)),
            None => Some(total_items - 1),
        };
    }

    /// Selects the first breadcrumb segment (root, index `0`).
    pub fn select_first(&mut self) {
        self.selected = Some(0);
    }

    /// Selects the last breadcrumb segment (active leaf, index `total_items - 1`).
    pub fn select_last(&mut self, total_items: usize) {
        if total_items == 0 {
            self.selected = None;
        } else {
            self.selected = Some(total_items - 1);
        }
    }

    /// Returns the index of the breadcrumb item located at terminal column `x` and row `y`.
    ///
    /// Returns `None` if the coordinates do not overlap any visible segment label.
    #[must_use]
    pub fn item_at(&self, column: u16, row: u16) -> Option<usize> {
        for &(idx, rect) in &self.item_rects {
            if contains(rect, column, row) {
                return Some(idx);
            }
        }
        None
    }

    /// Returns the index of the ancestor breadcrumb whose dropdown indicator (e.g. `▾`)
    /// is located at terminal column `x` and row `y`.
    ///
    /// Returns `None` if the coordinates do not overlap any visible dropdown indicator.
    #[must_use]
    pub fn dropdown_at(&self, column: u16, row: u16) -> Option<usize> {
        for &(idx, rect) in &self.dropdown_rects {
            if contains(rect, column, row) {
                return Some(idx);
            }
        }
        None
    }

    /// Checks if terminal column `x` and row `y` overlap the collapsed ellipsis indicator (`...`).
    #[must_use]
    pub fn is_ellipsis_at(&self, column: u16, row: u16) -> bool {
        self.ellipsis_rect
            .is_some_and(|rect| contains(rect, column, row))
    }

    /// Resets the spatial cache before a render pass.
    pub(crate) fn reset_cache(&mut self, area: Rect) {
        self.item_rects.clear();
        self.dropdown_rects.clear();
        self.ellipsis_rect = None;
        self.rendered_area = area;
    }
}

/// Helper function to check if a point (x, y) is contained within a Rect.
fn contains(rect: Rect, x: u16, y: u16) -> bool {
    x >= rect.x
        && x < rect.x.saturating_add(rect.width)
        && y >= rect.y
        && y < rect.y.saturating_add(rect.height)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_selection_navigation() {
        let mut state = BreadcrumbState::default();
        assert_eq!(state.selected(), None);

        state.select_next(3);
        assert_eq!(state.selected(), Some(0));

        state.select_next(3);
        assert_eq!(state.selected(), Some(1));

        state.select_next(3);
        assert_eq!(state.selected(), Some(2));

        // Clamped at end
        state.select_next(3);
        assert_eq!(state.selected(), Some(2));

        // Move backward
        state.select_previous(3);
        assert_eq!(state.selected(), Some(1));

        state.select_first();
        assert_eq!(state.selected(), Some(0));

        state.select_last(3);
        assert_eq!(state.selected(), Some(2));
    }

    #[test]
    fn test_hit_testing() {
        let mut state = BreadcrumbState::default();
        state.item_rects.push((0, Rect::new(0, 0, 5, 1)));
        state.dropdown_rects.push((0, Rect::new(5, 0, 2, 1)));
        state.ellipsis_rect = Some(Rect::new(10, 0, 3, 1));

        // Test item hit
        assert_eq!(state.item_at(2, 0), Some(0));
        assert_eq!(state.item_at(6, 0), None);

        // Test dropdown hit
        assert_eq!(state.dropdown_at(5, 0), Some(0));
        assert_eq!(state.dropdown_at(2, 0), None);

        // Test ellipsis hit
        assert!(state.is_ellipsis_at(11, 0));
        assert!(!state.is_ellipsis_at(2, 0));
    }
}
