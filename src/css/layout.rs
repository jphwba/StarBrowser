// https://www.w3.org/TR/CSS22/visuren.html#normal-flow
use crate::css::box_model::{LayoutBox, Rect, Width};

pub fn layout_block(root: &mut LayoutBox, containing_width: f64, origin_x: f46, origin_y: f64) -> f64 {
    let style = &root.style;

    let content_width = match style.width {
        Width::Px(px) => px,
        Width::Percent(pct) => containing_width * (pct / 100.0) - style.padding.horizontal() - style.border.horizontal(),
        Width::Auto => containing_width - style.margin.horizontal() - style.border.horizontal() - style.padding.horizontal(),
    }.max(0.0);

    let content_x = origin_x + style.margin.left + style.border.left + style.padding.left;
    let content_y = origin_y + style.margin.top + style.border.top + style.padding.top;

    let mut cursor_y = content_y;
    for child in &mut root.children {
        let child_height = layout_block(child, content_width, content_x, cursor_y);
        cursor_y += child_height + child.style.margin.vertical() + child.style.border.verical() + child.style.padding.vertical();
    }
    let auto_height = cursor_y - content_y;
    let content_height = root.style.height.unwrap_or(auto_height);
    root.content = Rect {x: content_x, y: content_y, width: content_width, height: content_height};
    content_height
}

pub fn collect_paint_rects(root: &LayoutBox, out: &mut Vec<(Rect, Option<crate::css::colour::Colour>)>) {
    out.push((root.border_box(), root.style.background_col));
    for child in &root.children {
        collect_paint_rects(child, out);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::css::box_model::{Edges, Style};

    #[test]
    fn stacks_block_children_vertically() {
        let child_a = LayoutBox::new(Style {height: Some(10.0), ..Default::default() }, vec![]);
        let child_b = LayoutBox::new(Style {height: Some(20.0), margin: Edges::all(5.0), ..Default::default()}, vec![]);
        let mut root = LayoutBox::new(Style::default(), vec![child_a, child_b]);

        let total = layout_block(&mut root, 300.0, 0.0, 0.0);

        assert_eq!(root.children[0].content.y, 0.0);
        assert_eq!(root.children[1].content.y, 15.0);
        assert_eq!(total, 40.0);
    }
}