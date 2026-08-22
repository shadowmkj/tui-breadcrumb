// ==============================================================================
// Truncation Strategies
// ==============================================================================

//! Smart overflow and truncation strategies for breadcrumb trails.
//!
//! Terminal displays frequently have constrained horizontal space. When a breadcrumb path
//! is longer than the available render width, a [`TruncateStrategy`] dictates how segments
//! are condensed, abbreviated, or replaced with ellipsis indicators.

/// Strategy determining how breadcrumb segments are collapsed when overflowing horizontal bounds.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TruncateStrategy {
    /// Collapses intermediate ancestor segments into an ellipsis while preserving
    /// the root ancestor(s) and deepest active segment(s).
    ///
    /// Example: `Home ❯ ... ❯ src ❯ sparkline.rs`
    Middle {
        /// Minimum number of root/head segments to preserve on the left.
        min_head_items: usize,
        /// Minimum number of leaf/tail segments to preserve on the right.
        min_tail_items: usize,
        /// The ellipsis indicator string (default `"..."`).
        ellipsis: String,
    },

    /// Collapses leftmost ancestor segments into an ellipsis while preserving
    /// the deepest active leaf segments.
    ///
    /// Example: `... ❯ src ❯ sparkline.rs`
    Start {
        /// Minimum number of leaf/tail segments to preserve on the right.
        min_tail_items: usize,
        /// The ellipsis indicator string (default `"..."`).
        ellipsis: String,
    },

    /// Collapses deepest leaf segments into an ellipsis while preserving
    /// root ancestor segments (left-to-right priority).
    ///
    /// Example: `Home ❯ Projects ❯ ...`
    End {
        /// Minimum number of root/head segments to preserve on the left.
        min_head_items: usize,
        /// The ellipsis indicator string (default `"..."`).
        ellipsis: String,
    },

    /// Progressively abbreviates ancestor item labels to their leading characters
    /// before collapsing them into an ellipsis if space is still constrained.
    ///
    /// Example: `H ❯ P ❯ ratatui ❯ src ❯ sparkline.rs`
    ShortenNames {
        /// Target abbreviation character length for shortened ancestor labels (default 1).
        max_abbrev_len: usize,
        /// Number of deepest tail segments to leave unshortened.
        preserve_tail_items: usize,
        /// Fallback ellipsis indicator if the trail still overflows after shortening.
        ellipsis: String,
    },

    /// Performs no smart condensation or ellipsis substitution; segments are clipped
    /// at the boundary.
    None,
}

impl Default for TruncateStrategy {
    /// Defaults to [`TruncateStrategy::Middle`] with 1 head item, 2 tail items, and `...` ellipsis.
    fn default() -> Self {
        Self::middle()
    }
}

impl TruncateStrategy {
    /// Creates a default `Middle` strategy preserving 1 root item, 2 tail items, and `"..."` ellipsis.
    #[must_use]
    pub fn middle() -> Self {
        Self::Middle {
            min_head_items: 1,
            min_tail_items: 2,
            ellipsis: String::from("..."),
        }
    }

    /// Creates a customized `Middle` strategy.
    #[must_use]
    pub fn middle_with(
        min_head_items: usize,
        min_tail_items: usize,
        ellipsis: impl Into<String>,
    ) -> Self {
        Self::Middle {
            min_head_items,
            min_tail_items,
            ellipsis: ellipsis.into(),
        }
    }

    /// Creates a default `Start` strategy preserving 2 tail items and `"..."` ellipsis.
    #[must_use]
    pub fn start() -> Self {
        Self::Start {
            min_tail_items: 2,
            ellipsis: String::from("..."),
        }
    }

    /// Creates a customized `Start` strategy.
    #[must_use]
    pub fn start_with(min_tail_items: usize, ellipsis: impl Into<String>) -> Self {
        Self::Start {
            min_tail_items,
            ellipsis: ellipsis.into(),
        }
    }

    /// Creates a default `End` strategy preserving 1 head item and `"..."` ellipsis.
    #[must_use]
    pub fn end() -> Self {
        Self::End {
            min_head_items: 1,
            ellipsis: String::from("..."),
        }
    }

    /// Creates a customized `End` strategy.
    #[must_use]
    pub fn end_with(min_head_items: usize, ellipsis: impl Into<String>) -> Self {
        Self::End {
            min_head_items,
            ellipsis: ellipsis.into(),
        }
    }

    /// Creates a default `ShortenNames` strategy abbreviating ancestors to 1 character,
    /// preserving 2 tail items, and falling back to `"..."`.
    #[must_use]
    pub fn shorten_names() -> Self {
        Self::ShortenNames {
            max_abbrev_len: 1,
            preserve_tail_items: 2,
            ellipsis: String::from("..."),
        }
    }

    /// Creates a customized `ShortenNames` strategy.
    #[must_use]
    pub fn shorten_names_with(
        max_abbrev_len: usize,
        preserve_tail_items: usize,
        ellipsis: impl Into<String>,
    ) -> Self {
        Self::ShortenNames {
            max_abbrev_len,
            preserve_tail_items,
            ellipsis: ellipsis.into(),
        }
    }

    /// Creates a `None` strategy that performs no truncation or ellipsis insertion.
    #[must_use]
    pub fn none() -> Self {
        Self::None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strategy_defaults() {
        let strategy = TruncateStrategy::default();
        assert_eq!(
            strategy,
            TruncateStrategy::Middle {
                min_head_items: 1,
                min_tail_items: 2,
                ellipsis: String::from("..."),
            }
        );
        assert_eq!(TruncateStrategy::none(), TruncateStrategy::None);
    }
}
