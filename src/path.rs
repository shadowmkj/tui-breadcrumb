// ==============================================================================
// Path Conversion Utilities
// ==============================================================================

//! Utilities to convert standard filesystem paths ([`Path`]) into breadcrumb items.

use crate::item::BreadcrumbItem;
use std::path::{Component, Path};

/// Parses a filesystem [`Path`] into a vector of [`BreadcrumbItem`] segments.
///
/// Handles root directories (`/` on Unix, drive letters on Windows), normal folder/file components,
/// and special dot segments (`.`, `..`).
///
/// # Examples
///
/// ```rust
/// use std::path::Path;
/// use tui_breadcrumb::from_path;
///
/// let path = Path::new("/var/log/nginx/access.log");
/// let items = from_path(path);
/// assert_eq!(items.len(), 5);
/// ```
#[must_use]
pub fn from_path<P: AsRef<Path>>(path: P) -> Vec<BreadcrumbItem<'static>> {
    let path = path.as_ref();
    let mut items = Vec::new();

    for component in path.components() {
        match component {
            Component::RootDir => {
                items.push(BreadcrumbItem::new("/".to_string()));
            }
            Component::Prefix(prefix) => {
                let s = prefix.as_os_str().to_string_lossy().to_string();
                items.push(BreadcrumbItem::new(s));
            }
            Component::CurDir => {
                items.push(BreadcrumbItem::new(".".to_string()));
            }
            Component::ParentDir => {
                items.push(BreadcrumbItem::new("..".to_string()));
            }
            Component::Normal(os_str) => {
                let s = os_str.to_string_lossy().to_string();
                items.push(BreadcrumbItem::new(s));
            }
        }
    }

    items
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_path() {
        let path = Path::new("/projects/ratatui/src/main.rs");
        let items = from_path(path);
        let labels: Vec<String> = items
            .iter()
            .map(|item| {
                item.label
                    .spans
                    .iter()
                    .map(|s| s.content.as_ref())
                    .collect()
            })
            .collect();

        assert_eq!(labels, vec!["/", "projects", "ratatui", "src", "main.rs"]);
    }
}
