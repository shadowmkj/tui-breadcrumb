use insta::assert_snapshot;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::widgets::Widget;

use tui_breadcrumbs::{Breadcrumb, BreadcrumbSeparator, TruncateStrategy};

fn render_to_string(widget: Breadcrumb, width: u16) -> String {
    let area = Rect::new(0, 0, width, 1);
    let mut buffer = Buffer::empty(area);
    Widget::render(widget, area, &mut buffer);

    (0..width)
        .map(|x| buffer[(x, 0)].symbol())
        .collect::<String>()
        .trim_end()
        .to_string()
}

#[test]
fn test_snapshot_separators() {
    let items = vec!["Home", "Projects", "ratatui"];

    let slash = render_to_string(
        Breadcrumb::new(items.clone()).separator(BreadcrumbSeparator::slash()),
        30,
    );
    let chevron = render_to_string(
        Breadcrumb::new(items.clone()).separator(BreadcrumbSeparator::chevron()),
        30,
    );
    let arrow = render_to_string(
        Breadcrumb::new(items.clone()).separator(BreadcrumbSeparator::arrow()),
        30,
    );
    let pipe = render_to_string(
        Breadcrumb::new(items).separator(BreadcrumbSeparator::pipe()),
        30,
    );

    assert_snapshot!(
        "separators",
        format!(
            "slash: {}\nchevron: {}\narrow: {}\npipe: {}",
            slash, chevron, arrow, pipe
        )
    );
}

#[test]
fn test_snapshot_truncation_strategies() {
    let items = vec!["Home", "Projects", "ratatui", "src", "sparkline.rs"];

    let middle = render_to_string(
        Breadcrumb::new(items.clone()).strategy(TruncateStrategy::middle()),
        30,
    );
    let start = render_to_string(
        Breadcrumb::new(items.clone()).strategy(TruncateStrategy::start()),
        30,
    );
    let shorten = render_to_string(
        Breadcrumb::new(items.clone()).strategy(TruncateStrategy::shorten_names()),
        30,
    );
    let end = render_to_string(Breadcrumb::new(items).strategy(TruncateStrategy::end()), 30);

    assert_snapshot!(
        "truncation_strategies",
        format!(
            "middle: {}\nstart: {}\nshorten: {}\nend: {}",
            middle, start, shorten, end
        )
    );
}
