// https://www.w3.org/TR/CSS22/box.html
use crate::css::colour::Colour;

#[derive(Debug, Clone, Copy, Default, PartialEq)]

pub struct Edges {
    pub top: f64,
    pub right: f64,
    pub bottom: f64,
    pub left: f64,
}

impl Edges {
    pub fn all(v: f64) -> Self {
        Edges{top: v, right: v, bottom: v, left: v}
    }
    pub fn horizontal(&self) -> f64 {
        self.left + self.right
    }

    pub fn vertical(&self) -> f64 {
        self.top + self.bottom
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]

pub enum BoxType {
    Block,
    Inline,
    None,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Width {
    Auto,
    Px(f64),
    Percent(f64),
}

impl Default for Width {
    fn default() -> Self {
        Width::Auto
    }
}

#[derive(Debug, Clone)]
pub struct Style {
    pub box_type: BoxType,
    pub width: Width,
    pub height: Option<f64>,
    pub margin: Edges,
    pub border: Edges,
    pub padding: Edges,
    pub colour: Colour,
    pub background_col: Option<Colour>,
}

impl Default for Style {
    fn default() -> Self {
        Style {
            box_type: BoxType::Block,
            width: Width::Auto,
            height: none,
            margin: Edges::default(),
            border: Edges::default(),
            padding: Edges::default(),
            colour: Colour::default(),
            background_col: None,
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Rect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

impl Rect {
    pub fn expandedby(&self, edges: Edges) -> Rect {
        Rect {
            x: self.x - edges.left,
            y: self.y - edges.top,
            width: self.width + edges.horizontal(),
            height: self.height + edges.vertical(),
        }
    }
}

pub struct LayoutBox {
    pub style: Style,
    pub children: Vec<LayoutBox>,
    pub content: Rect,
}

impl LayoutBox {
    pub fn new(style: Style, children: Vec<LayoutBox>) -> Self {
        LayoutBox {style, children, content: Rect::default()}
    }
    pub fn paddingbox(&self) -> Rect {
        self.content.expandedby(self.style.padding)
    }
    pub fn borderbox(&self) -> Rect {
        self.paddingbox().expandedby(self.style.border)
    }
    pub fn marginbox(&self) -> Rect {
        self.borderbox().expandedby(self.style.margin)
    }
}