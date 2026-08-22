use quickcheck_macros::quickcheck;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::widgets::Widget;

use tui_breadcrumb::{Breadcrumb, BreadcrumbItem, BreadcrumbSeparator, TruncateStrategy};

#[quickcheck]
fn prop_layout_never_overflows_width(
    raw_items: Vec<String>,
    width_raw: u8,
    strategy_kind: u8,
) -> bool {
    let width = (width_raw % 100) + 1; // Width between 1 and 100
    let items: Vec<BreadcrumbItem> = raw_items
        .into_iter()
        .take(15) // limit to reasonable length
        .map(|s| {
            // sanitize string to single line
            let clean: String = s.chars().filter(|c| !c.is_control()).take(20).collect();
            BreadcrumbItem::new(clean)
        })
        .collect();

    let strategy = match strategy_kind % 5 {
        0 => TruncateStrategy::middle(),
        1 => TruncateStrategy::start(),
        2 => TruncateStrategy::shorten_names(),
        3 => TruncateStrategy::end(),
        _ => TruncateStrategy::none(),
    };

    let widget = Breadcrumb::new(items)
        .separator(BreadcrumbSeparator::chevron())
        .strategy(strategy);

    let area = Rect::new(0, 0, width as u16, 1);
    let mut buffer = Buffer::empty(area);

    // Must not panic
    Widget::render(widget, area, &mut buffer);

    true
}
