// ==============================================================================
// Truncation & Layout Resolution Engine
// ==============================================================================

//! Internal layout resolution and segment fitting engine.
//!
//! Responsible for taking a sequence of [`BreadcrumbItem`]s, a [`BreadcrumbSeparator`](crate::BreadcrumbSeparator),
//! an available terminal width, and a [`TruncateStrategy`], and producing an optimal
//! visual layout of elements ([`RenderElement`]) that strictly fits within the bounds.

use unicode_width::UnicodeWidthStr;

use crate::item::BreadcrumbItem;
use crate::strategy::TruncateStrategy;

/// An individual visual token in the resolved breadcrumb layout.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RenderElement {
    /// A regular or shortened breadcrumb item segment.
    Item {
        /// Original index in the input items array.
        index: usize,
        /// Optional shortened/abbreviated label string override (e.g. for `ShortenNames`).
        abbreviated_label: Option<String>,
    },
    /// A separator placed between adjacent items or ellipses.
    Separator,
    /// An ellipsis indicating collapsed intermediate or ancestor items.
    Ellipsis {
        /// The ellipsis text to display (e.g. `"..."` or `"…"`).
        text: String,
    },
}

/// Helper function to safely shorten a string by unicode character count.
#[must_use]
pub fn shorten_str(s: &str, max_chars: usize) -> String {
    if s.is_empty() || max_chars == 0 {
        return String::new();
    }
    let mut end_byte = 0;
    for (count, (idx, ch)) in s.char_indices().enumerate() {
        if count >= max_chars {
            break;
        }
        end_byte = idx + ch.len_utf8();
    }
    if end_byte == 0 {
        return s
            .chars()
            .next()
            .map_or_else(String::new, |ch| s[..ch.len_utf8()].to_string());
    }
    s[..end_byte].to_string()
}

/// Computes the visual terminal width of an item, accounting for potential shortened labels.
fn compute_item_width(
    item: &BreadcrumbItem<'_>,
    abbrev_label: Option<&str>,
    default_dropdown_sym: &str,
) -> usize {
    let label_w = match abbrev_label {
        Some(abbrev) => UnicodeWidthStr::width(abbrev),
        None => item.label_width(),
    };
    if item.has_dropdown {
        let sym = item
            .dropdown_symbol
            .as_deref()
            .unwrap_or(default_dropdown_sym);
        label_w + 1 + UnicodeWidthStr::width(sym)
    } else {
        label_w
    }
}

/// Resolves the layout elements that fit within `available_width`.
#[must_use]
pub fn resolve_layout(
    items: &[BreadcrumbItem<'_>],
    sep_width: usize,
    available_width: usize,
    strategy: &TruncateStrategy,
    default_dropdown_sym: &str,
) -> Vec<RenderElement> {
    if items.is_empty() || available_width == 0 {
        return Vec::new();
    }

    let n = items.len();
    let original_widths: Vec<usize> = items
        .iter()
        .map(|item| compute_item_width(item, None, default_dropdown_sym))
        .collect();

    // Check if everything fits completely without truncation
    let total_untruncated_width: usize =
        original_widths.iter().sum::<usize>() + (n.saturating_sub(1) * sep_width);

    if total_untruncated_width <= available_width {
        return build_full_layout(n);
    }

    match strategy {
        TruncateStrategy::None => {
            build_clipped_layout(&original_widths, sep_width, available_width)
        }
        TruncateStrategy::Start {
            min_tail_items,
            ellipsis,
        } => resolve_start_layout(
            items,
            &original_widths,
            sep_width,
            available_width,
            *min_tail_items,
            ellipsis,
            default_dropdown_sym,
        ),
        TruncateStrategy::Middle {
            min_head_items,
            min_tail_items,
            ellipsis,
        } => resolve_middle_layout(
            items,
            &original_widths,
            sep_width,
            available_width,
            *min_head_items,
            *min_tail_items,
            ellipsis,
            default_dropdown_sym,
        ),
        TruncateStrategy::End {
            min_head_items,
            ellipsis,
        } => resolve_end_layout(
            &original_widths,
            sep_width,
            available_width,
            *min_head_items,
            ellipsis,
        ),
        TruncateStrategy::ShortenNames {
            max_abbrev_len,
            preserve_tail_items,
            ellipsis,
        } => resolve_shorten_names_layout(
            items,
            sep_width,
            available_width,
            *max_abbrev_len,
            *preserve_tail_items,
            ellipsis,
            default_dropdown_sym,
        ),
    }
}

/// Builds an untruncated layout with all items and separators.
fn build_full_layout(n: usize) -> Vec<RenderElement> {
    let mut elements = Vec::with_capacity(n * 2);
    for i in 0..n {
        if i > 0 {
            elements.push(RenderElement::Separator);
        }
        elements.push(RenderElement::Item {
            index: i,
            abbreviated_label: None,
        });
    }
    elements
}

/// Builds a clipped layout (left to right) without ellipsis.
fn build_clipped_layout(
    widths: &[usize],
    sep_width: usize,
    available_width: usize,
) -> Vec<RenderElement> {
    let mut elements = Vec::new();
    let mut current_width = 0;

    for (i, &w) in widths.iter().enumerate() {
        let needed = if i > 0 { sep_width + w } else { w };
        if current_width + needed > available_width {
            break;
        }
        if i > 0 {
            elements.push(RenderElement::Separator);
        }
        elements.push(RenderElement::Item {
            index: i,
            abbreviated_label: None,
        });
        current_width += needed;
    }
    elements
}

/// Resolves `TruncateStrategy::Start`: `... ❯ item_{N-k} ❯ ... ❯ item_{N-1}`
fn resolve_start_layout(
    _items: &[BreadcrumbItem<'_>],
    widths: &[usize],
    sep_width: usize,
    available_width: usize,
    _min_tail_items: usize,
    ellipsis: &str,
    _default_dropdown_sym: &str,
) -> Vec<RenderElement> {
    let n = widths.len();
    let ellipsis_w = UnicodeWidthStr::width(ellipsis);

    // If available width is extremely small, try just the last item or just the ellipsis
    if available_width < ellipsis_w {
        return vec![RenderElement::Ellipsis {
            text: ellipsis.to_string(),
        }];
    }

    // Always try to include at least the last item (tail)
    let last_idx = n.saturating_sub(1);
    let last_w = widths[last_idx];

    // Base cost: ellipsis + sep + last_item
    let base_cost = ellipsis_w + sep_width + last_w;
    if base_cost > available_width {
        // If even ellipsis + last item doesn't fit, check if last item alone fits
        if last_w <= available_width {
            return vec![RenderElement::Item {
                index: last_idx,
                abbreviated_label: None,
            }];
        }
        return vec![RenderElement::Ellipsis {
            text: ellipsis.to_string(),
        }];
    }

    // Accumulate tail items from right to left
    let mut included_tail_count = 1;
    let mut current_cost = base_cost;

    for idx in (0..last_idx).rev() {
        let item_cost = sep_width + widths[idx];
        if current_cost + item_cost <= available_width {
            current_cost += item_cost;
            included_tail_count += 1;
        } else {
            break;
        }
    }

    // If all items fit, we don't need ellipsis
    if included_tail_count == n {
        return build_full_layout(n);
    }

    let start_tail_idx = n - included_tail_count;
    let mut elements = Vec::with_capacity(included_tail_count * 2 + 1);
    elements.push(RenderElement::Ellipsis {
        text: ellipsis.to_string(),
    });
    for idx in start_tail_idx..n {
        elements.push(RenderElement::Separator);
        elements.push(RenderElement::Item {
            index: idx,
            abbreviated_label: None,
        });
    }

    elements
}

/// Resolves `TruncateStrategy::Middle`: `item_0 ❯ ... ❯ item_{N-k} ❯ item_{N-1}`
#[allow(clippy::too_many_arguments)]
fn resolve_middle_layout(
    items: &[BreadcrumbItem<'_>],
    widths: &[usize],
    sep_width: usize,
    available_width: usize,
    min_head_items: usize,
    min_tail_items: usize,
    ellipsis: &str,
    default_dropdown_sym: &str,
) -> Vec<RenderElement> {
    let n = widths.len();
    if n <= 2 {
        // Fallback to start layout for very short lists
        return resolve_start_layout(
            items,
            widths,
            sep_width,
            available_width,
            min_tail_items,
            ellipsis,
            default_dropdown_sym,
        );
    }

    let ellipsis_w = UnicodeWidthStr::width(ellipsis);
    let head_count_req = min_head_items.max(1);
    let tail_count_req = min_tail_items.max(1);

    // Initial check: Head 0 and Tail N-1 with ellipsis between them:
    // item_0 + sep + ellipsis + sep + item_{N-1}
    let minimal_middle_cost = widths[0] + sep_width + ellipsis_w + sep_width + widths[n - 1];
    if minimal_middle_cost > available_width {
        // Fallback to Start layout
        return resolve_start_layout(
            items,
            widths,
            sep_width,
            available_width,
            min_tail_items,
            ellipsis,
            default_dropdown_sym,
        );
    }

    let mut head_end = 1; // exclusive index: items [0..head_end]
    let mut tail_start = n - 1; // inclusive index: items [tail_start..n]
    let mut current_cost = minimal_middle_cost;

    // Try to satisfy required head and tail counts first if space permits
    while head_end < head_count_req && head_end < tail_start {
        let add_cost = sep_width + widths[head_end];
        if current_cost + add_cost <= available_width {
            current_cost += add_cost;
            head_end += 1;
        } else {
            break;
        }
    }

    while (n - tail_start) < tail_count_req && tail_start > head_end {
        let add_cost = sep_width + widths[tail_start - 1];
        if current_cost + add_cost <= available_width {
            current_cost += add_cost;
            tail_start -= 1;
        } else {
            break;
        }
    }

    // Greedily expand tail, then head, until space exhausted or they meet
    loop {
        let mut expanded = false;

        // Try adding another tail item first (deeper context)
        if tail_start > head_end {
            let add_cost = sep_width + widths[tail_start - 1];
            if current_cost + add_cost <= available_width {
                current_cost += add_cost;
                tail_start -= 1;
                expanded = true;
            }
        }

        // Try adding another head item
        if head_end < tail_start {
            let add_cost = sep_width + widths[head_end];
            if current_cost + add_cost <= available_width {
                current_cost += add_cost;
                head_end += 1;
                expanded = true;
            }
        }

        if !expanded {
            break;
        }
    }

    // If head meets tail, everything fits without ellipsis
    if head_end >= tail_start {
        return build_full_layout(n);
    }

    // Assemble layout: [0..head_end] + [Ellipsis] + [tail_start..n]
    let mut elements = Vec::new();
    for i in 0..head_end {
        if i > 0 {
            elements.push(RenderElement::Separator);
        }
        elements.push(RenderElement::Item {
            index: i,
            abbreviated_label: None,
        });
    }

    elements.push(RenderElement::Separator);
    elements.push(RenderElement::Ellipsis {
        text: ellipsis.to_string(),
    });
    elements.push(RenderElement::Separator);

    for i in tail_start..n {
        elements.push(RenderElement::Item {
            index: i,
            abbreviated_label: None,
        });
        if i + 1 < n {
            elements.push(RenderElement::Separator);
        }
    }

    elements
}

/// Resolves `TruncateStrategy::End`: `item_0 ❯ item_1 ❯ ...`
fn resolve_end_layout(
    widths: &[usize],
    sep_width: usize,
    available_width: usize,
    min_head_items: usize,
    ellipsis: &str,
) -> Vec<RenderElement> {
    let n = widths.len();
    let ellipsis_w = UnicodeWidthStr::width(ellipsis);

    if available_width < ellipsis_w {
        return vec![RenderElement::Ellipsis {
            text: ellipsis.to_string(),
        }];
    }

    let mut elements = Vec::new();
    let mut current_cost = 0;
    let mut included_count = 0;

    for (i, &w) in widths.iter().enumerate() {
        let is_first = i == 0;
        let item_cost = if is_first { w } else { sep_width + w };
        let cost_with_ellipsis = current_cost + item_cost + sep_width + ellipsis_w;

        if (cost_with_ellipsis <= available_width)
            || (is_first && item_cost <= available_width && min_head_items >= 1)
        {
            if !is_first {
                elements.push(RenderElement::Separator);
            }
            elements.push(RenderElement::Item {
                index: i,
                abbreviated_label: None,
            });
            current_cost += item_cost;
            included_count += 1;
        } else {
            break;
        }
    }

    if included_count == n {
        return build_full_layout(n);
    }

    if !elements.is_empty() {
        elements.push(RenderElement::Separator);
    }
    elements.push(RenderElement::Ellipsis {
        text: ellipsis.to_string(),
    });

    elements
}

/// Resolves `TruncateStrategy::ShortenNames`: `H ❯ P ❯ ratatui ❯ src ❯ sparkline.rs`
fn resolve_shorten_names_layout(
    items: &[BreadcrumbItem<'_>],
    sep_width: usize,
    available_width: usize,
    max_abbrev_len: usize,
    preserve_tail_items: usize,
    ellipsis: &str,
    default_dropdown_sym: &str,
) -> Vec<RenderElement> {
    let n = items.len();
    let tail_preserve_start = n.saturating_sub(preserve_tail_items.max(1));

    // Generate shortened labels for candidates (indices 0..tail_preserve_start)
    let mut shortened_labels: Vec<Option<String>> = Vec::with_capacity(n);
    let mut new_widths: Vec<usize> = Vec::with_capacity(n);

    for (i, item) in items.iter().enumerate() {
        if i < tail_preserve_start {
            // Extract raw string representation from line to abbreviate
            let raw_text = item
                .label
                .spans
                .iter()
                .map(|s| s.content.as_ref())
                .collect::<String>();
            let abbrev = shorten_str(&raw_text, max_abbrev_len);
            let w = compute_item_width(item, Some(&abbrev), default_dropdown_sym);
            shortened_labels.push(Some(abbrev));
            new_widths.push(w);
        } else {
            let w = compute_item_width(item, None, default_dropdown_sym);
            shortened_labels.push(None);
            new_widths.push(w);
        }
    }

    let total_shortened_width: usize =
        new_widths.iter().sum::<usize>() + (n.saturating_sub(1) * sep_width);

    // If shortened fits, return layout with shortened tokens
    if total_shortened_width <= available_width {
        let mut elements = Vec::with_capacity(n * 2);
        for (i, label) in shortened_labels.into_iter().enumerate() {
            if i > 0 {
                elements.push(RenderElement::Separator);
            }
            elements.push(RenderElement::Item {
                index: i,
                abbreviated_label: label,
            });
        }
        return elements;
    }

    // Fallback: apply Middle or Start truncation over the shortened layout
    resolve_middle_layout(
        items,
        &new_widths,
        sep_width,
        available_width,
        1,
        preserve_tail_items,
        ellipsis,
        default_dropdown_sym,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::item::BreadcrumbItem;

    #[test]
    fn test_shorten_str() {
        assert_eq!(shorten_str("Home", 1), "H");
        assert_eq!(shorten_str("Projects", 2), "Pr");
        assert_eq!(shorten_str("🦀 Rust", 1), "🦀");
        assert_eq!(shorten_str("", 1), "");
    }

    #[test]
    fn test_resolve_layout_full_fit() {
        let items = vec![
            BreadcrumbItem::new("Home"),
            BreadcrumbItem::new("Projects"),
            BreadcrumbItem::new("ratatui"),
        ];
        // "Home" (4) + sep(3) + "Projects" (8) + sep(3) + "ratatui" (7) = 25
        let layout = resolve_layout(&items, 3, 30, &TruncateStrategy::middle(), "▾");
        assert_eq!(
            layout,
            vec![
                RenderElement::Item {
                    index: 0,
                    abbreviated_label: None
                },
                RenderElement::Separator,
                RenderElement::Item {
                    index: 1,
                    abbreviated_label: None
                },
                RenderElement::Separator,
                RenderElement::Item {
                    index: 2,
                    abbreviated_label: None
                },
            ]
        );
    }

    #[test]
    fn test_resolve_layout_start() {
        let items = vec![
            BreadcrumbItem::new("Home"),
            BreadcrumbItem::new("Projects"),
            BreadcrumbItem::new("ratatui"),
            BreadcrumbItem::new("src"),
            BreadcrumbItem::new("lib.rs"),
        ];
        let layout = resolve_layout(&items, 3, 20, &TruncateStrategy::start(), "▾");
        assert_eq!(
            layout,
            vec![
                RenderElement::Ellipsis { text: "...".into() },
                RenderElement::Separator,
                RenderElement::Item {
                    index: 3,
                    abbreviated_label: None
                },
                RenderElement::Separator,
                RenderElement::Item {
                    index: 4,
                    abbreviated_label: None
                },
            ]
        );
    }

    #[test]
    fn test_resolve_layout_middle() {
        let items = vec![
            BreadcrumbItem::new("Home"),
            BreadcrumbItem::new("Projects"),
            BreadcrumbItem::new("ratatui"),
            BreadcrumbItem::new("src"),
            BreadcrumbItem::new("lib.rs"),
        ];
        let layout = resolve_layout(&items, 3, 26, &TruncateStrategy::middle(), "▾");
        assert_eq!(
            layout,
            vec![
                RenderElement::Item {
                    index: 0,
                    abbreviated_label: None
                },
                RenderElement::Separator,
                RenderElement::Ellipsis { text: "...".into() },
                RenderElement::Separator,
                RenderElement::Item {
                    index: 3,
                    abbreviated_label: None
                },
                RenderElement::Separator,
                RenderElement::Item {
                    index: 4,
                    abbreviated_label: None
                },
            ]
        );
    }

    #[test]
    fn test_resolve_layout_shorten_names() {
        let items = vec![
            BreadcrumbItem::new("Home"),
            BreadcrumbItem::new("Projects"),
            BreadcrumbItem::new("ratatui"),
            BreadcrumbItem::new("src"),
            BreadcrumbItem::new("lib.rs"),
        ];
        let strategy = TruncateStrategy::shorten_names_with(1, 2, "...");
        let layout = resolve_layout(&items, 3, 24, &strategy, "▾");
        assert_eq!(
            layout,
            vec![
                RenderElement::Item {
                    index: 0,
                    abbreviated_label: Some("H".into())
                },
                RenderElement::Separator,
                RenderElement::Item {
                    index: 1,
                    abbreviated_label: Some("P".into())
                },
                RenderElement::Separator,
                RenderElement::Item {
                    index: 2,
                    abbreviated_label: Some("r".into())
                },
                RenderElement::Separator,
                RenderElement::Item {
                    index: 3,
                    abbreviated_label: None
                },
                RenderElement::Separator,
                RenderElement::Item {
                    index: 4,
                    abbreviated_label: None
                },
            ]
        );
    }
}
