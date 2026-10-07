use crate::css::{parser::{ComponentValue, Function}, tokenise::{CSSToken, HashToken},};

pub const ACCENT_COLOUR_LIGHT: &str = "#B5A65A";
pub const ACCENT_COLOUR_DARK: &str = "#EFE7B6";
pub const ACCENT_COLOUR_TEXT_LIGHT: &str = "#FFFFFF";
pub const ACCENT_COLOUR_TEXT_DARK: &str = "#0D1B2A";
pub const ACTIVE_TEXT_LIGHT: &str = "#D97706";
pub const ACTIVE_TEXT_DARK: &str = "#FBBF24";
pub const BUTTON_BORDER_LIGHT: &str = "#D6CFA0";
pub const BUTTON_BORDER_DARK: &str = "#2A3A4E";
pub const BUTTON_FACE_LIGHT: &str = "#F7F4E3";
pub const BUTTON_FACE_DARK: &str = "#152232";
pub const BUTTON_TEXT_LIGHT: &str = "#0D1B2A";
pub const BUTTON_TEXT_DARK: &str = "#EFE7B6";
pub const CANVAS_LIGHT: &str = "#FAF8EE";
pub const CANVAS_DARK: &str = "#0A131D";
pub const CANVAS_TEXT_LIGHT: &str = "#0D1B2A";
pub const CANVAS_TEXT_DARK: &str = "#FFFFFF";
pub const FIELD_LIGHT: &str = "#FFFFFF";
pub const FIELD_DARK: &str = "#0D1B2A";
pub const FIELD_TEXT_LIGHT: &str = "#0D1B2A";
pub const FIELD_TEXT_DARK: &str = "#FAF8EE";
pub const GRAY_TEXT_LIGHT: &str = "#78828C";
pub const GRAY_TEXT_DARK: &str = "#8C9B9E";
pub const HIGHLIGHT_TEXT_LIGHT: &str = "#0D1B2A";
pub const HIGHLIGHT_TEXT_DARK: &str = "#EFE7B6";
pub const LINK_TEXT_LIGHT: &str = "#9E9047";
pub const LINK_TEXT_DARK: &str = "#EFE7B6";
pub const MARK_LIGHT: &str = "#EFE7B6";
pub const MARK_DARK: &str = "#4A4318";
pub const MARK_TEXT_LIGHT: &str = "#2E2909";
pub const MARK_TEXT_DARK: &str = "#FAF6DB";
pub const SELECTED_ITEM_LIGHT: &str = "#E7E0B0";
pub const SELECTED_ITEM_DARK: &str = "#23395B";
pub const VISITED_TEXT_LIGHT: &str = "#857437";
pub const VISITED_TEXT_DARK: &str = "#C5BC8E";

// Used gemini to determine hex code colours for the app theme

pub fn issyscolour(name: &str) -> bool {
    matches!(
        name, "AccentColour"
        | "AccentColourText"
        | "ActiveText"
        | "ButtonBorder"
        | "ButtonFace"
        | "ButtonText"
        | "Canvas"
        | "CanvasText"
        | "Field"
        | "FieldText"
        | "GrayText"
        | "Highlight"
        | "HighlightText"
        | "LinkText"
        | "Mark"
        | "MarkText"
        | "SelectedItem"
        | "SelectedItemText"
        | "VisitedText"
    )
}

pub fn getnamedcolour(name: &str) -> Option<&'static str> {
    match name.to_lowercase().as_str() {
        "aliceblue" => Some("#f0f8ff"),
        "antiquewhite" => Some("#faebd7"),
        "aqua" => Some("#00ffff"),
        "aquamarine" => Some("#7fffd4"),
        "azure" => Some("#f0ffff"),
        "beige" => Some("#f5f5dc"),
        "bisque" => Some("#ffe4c4"),
        "black" => Some("#000000"),
        "blanchedalmond" => Some("#ffebcd"),
        "blue" => Some("#0000ff"),
        "blueviolet" => Some("#8a2be2"),
        "brown" => Some("#a52a2a"),
        "burlywood" => Some("#deb887"),
        "cadetblue" => Some("#5f9ea0"),
        "chartreuse" => Some("#7fff00"),
        "chocolate" => Some("#d2691e"),
        "coral" => Some("#ff7f50"),
        "cornflowerblue" => Some("#6495ed"),
        "cornsilk" => Some("#fff8dc"),
        "crimson" => Some("#dc143c"),
        "cyan" => Some("#00ffff"),
        "darkblue" => Some("#00008b"),
        "darkcyan" => Some("#008b8b"),
        "darkgoldenrod" => Some("#b8860b"),
        "darkgray" => Some("#a9a9a9"),
        "darkgreen" => Some("#006400"),
        "darkgrey" => Some("#a9a9a9"),
        "darkkhaki" => Some("#bdb76b"),
        "darkmagenta" => Some("#8b008b"),
        "darkolivegreen" => Some("#556b2f"),
        "darkorange" => Some("#ff8c00"),
        "darkorchid" => Some("#9932cc"),
        "darkred" => Some("#8b0000"),
        "darksalmon" => Some("#e9967a"),
        "darkseagreen" => Some("#8fbc8f"),
        "darkslateblue" => Some("#483d8b"),
        "darkslategray" => Some("#2f4f4f"),
        "darkslategrey" => Some("#2f4f4f"),
        "darkturquoise" => Some("#00ced1"),
        "darkviolet" => Some("#9400d3"),
        "deeppink" => Some("#ff1493"),
        "deepskyblue" => Some("#00bfff"),
        "dimgray" => Some("#696969"),
        "dimgrey" => Some("#696969"),
        "dodgerblue" => Some("#1e90ff"),
        "firebrick" => Some("#b22222"),
        "floralwhite" => Some("#fffaf0"),
        "forestgreen" => Some("#228b22"),
        "fuchsia" => Some("#ff00ff"),
        "gainsboro" => Some("#dcdcdc"),
        "ghostwhite" => Some("#f8f8ff"),
        "gold" => Some("#ffd700"),
        "goldenrod" => Some("#daa520"),
        "gray" => Some("#808080"),
        "green" => Some("#008000"),
        "greenyellow" => Some("#adff2f"),
        "grey" => Some("#808080"),
        "honeydew" => Some("#f0fff0"),
        "hotpink" => Some("#ff69b4"),
        "indianred" => Some("#cd5c5c"),
        "indigo" => Some("#4b0082"),
        "ivory" => Some("#fffff0"),
        "khaki" => Some("#f0e68c"),
        "lavender" => Some("#e6e6fa"),
        "lavenderblush" => Some("#fff0f5"),
        "lawngreen" => Some("#7cfc00"),
        "lemonchiffon" => Some("#fffacd"),
        "lightblue" => Some("#add8e6"),
        "lightcoral" => Some("#f08080"),
        "lightcyan" => Some("#e0ffff"),
        "lightgoldenrodyellow" => Some("#fafad2"),
        "lightgray" => Some("#d3d3d3"),
        "lightgreen" => Some("#90ee90"),
        "lightgrey" => Some("#d3d3d3"),
        "lightpink" => Some("#ffb6c1"),
        "lightsalmon" => Some("#ffa07a"),
        "lightseagreen" => Some("#20b2aa"),
        "lightskyblue" => Some("#87cefa"),
        "lightslategray" => Some("#778899"),
        "lightslategrey" => Some("#778899"),
        "lightsteelblue" => Some("#b0c4de"),
        "lightyellow" => Some("#ffffe0"),
        "lime" => Some("#00ff00"),
        "limegreen" => Some("#32cd32"),
        "linen" => Some("#faf0e6"),
        "magenta" => Some("#ff00ff"),
        "maroon" => Some("#800000"),
        "mediumaquamarine" => Some("#66cdaa"),
        "mediumblue" => Some("#0000cd"),
        "mediumorchid" => Some("#ba55d3"),
        "mediumpurple" => Some("#9370db"),
        "mediumseagreen" => Some("#3cb371"),
        "mediumslateblue" => Some("#7b68ee"),
        "mediumspringgreen" => Some("#00fa9a"),
        "mediumturquoise" => Some("#48d1cc"),
        "mediumvioletred" => Some("#c71585"),
        "midnightblue" => Some("#191970"),
        "mintcream" => Some("#f5fffa"),
        "mistyrose" => Some("#ffe4e1"),
        "moccasin" => Some("#ffe4b5"),
        "navajowhite" => Some("#ffdead"),
        "navy" => Some("#000080"),
        "oldlace" => Some("#fdf5e6"),
        "olive" => Some("#808000"),
        "olivedrab" => Some("#6b8e23"),
        "orange" => Some("#ffa500"),
        "orangered" => Some("#ff4500"),
        "orchid" => Some("#da70d6"),
        "palegoldenrod" => Some("#eee8aa"),
        "palegreen" => Some("#98fb98"),
        "paleturquoise" => Some("#afeeee"),
        "palevioletred" => Some("#db7093"),
        "papayawhip" => Some("#ffefd5"),
        "peachpuff" => Some("#ffdab9"),
        "peru" => Some("#cd853f"),
        "pink" => Some("#ffc0cb"),
        "plum" => Some("#dda0dd"),
        "powderblue" => Some("#b0e0e6"),
        "purple" => Some("#800080"),
        "rebeccapurple" => Some("#663399"),
        "red" => Some("#ff0000"),
        "rosybrown" => Some("#bc8f8f"),
        "royalblue" => Some("#4169e1"),
        "saddlebrown" => Some("#8b4513"),
        "salmon" => Some("#fa8072"),
        "sandybrown" => Some("#f4a460"),
        "seagreen" => Some("#2e8b57"),
        "seashell" => Some("#fff5ee"),
        "sienna" => Some("#a0522d"),
        "silver" => Some("#c0c0c0"),
        "skyblue" => Some("#87ceeb"),
        "slateblue" => Some("#6a5acd"),
        "slategray" => Some("#708090"),
        "slategrey" => Some("#708090"),
        "snow" => Some("#fffafa"),
        "springgreen" => Some("#00ff7f"),
        "steelblue" => Some("#4682b4"),
        "tan" => Some("#d2b48c"),
        "teal" => Some("#008080"),
        "thistle" => Some("#d8bfd8"),
        "tomato" => Some("#ff6347"),
        "turquoise" => Some("#40e0d0"),
        "violet" => Some("#ee82ee"),
        "wheat" => Some("#f5deb3"),
        "white" => Some("#ffffff"),
        "whitesmoke" => Some("#f5f5f5"),
        "yellow" => Some("#ffff00"),
        "yellowgreen" => Some("#9acd32"),

        _ => None,
    }
}

pub fn iscolourfunction(name: &str) -> bool {
    matches!(name.to_lowercase().as_str(),
                "rgb" | "rgba" | "hsl" | "hsla" | "hwb" | "lab" | "lch" | "oklab" | "oklch" "colour")
}

pub fn iscolour(token: &ComponentValue) -> bool {
    match token {
        ComponentValue::Token(CSSToken::Ident(name))
            if name == "currentcolour" || name == "transparent" || issyscolour(name) => {true}
        ComponentValue::Token(CSSToken::Ident(name)) if getnamedcolour(name).is_some() => true,
        ComponentValue::Token(CSSToken::Hash(HashToken{value: val, ..})) => match val.len() {
            3 | 6 | 4 | 8 => val.chars().all(|c| c.is_ascii_hexdigit()),
            _=>false,
        },
        ComponentValue::Function(Function(name, ..)) if iscolourfunction(name) => {true} _=>false,
    }
}

pub fn hexToRGB(hex: &str) -> UsedColour {
    let hex = hex.trim.start_matches('#');
    let (r, g, b, a) = match hex.len() {
        3 => (u8::from_str_radix(&hex[0..1].repeat(2), 16).unwrap_or(0),
            u8::from_str_radix(&hex[1..2].repeat(2), 16).unwrap_or(0),
            u8::from_str_radix(&hex[2..3].repeat(2), 16).unwrap_or(0),
            100.0,),
        4 => (u8::from_str_radix(&hex[0..1].repeat(2), 16).unwrap_or(0),
            u8::from_str_radix(&hex[1..2].repeat(2), 16).unwrap_or(0),
            u8::from_str_radix(&hex[2..3].repeat(2), 16).unwrap_or(0),
            u8::from_str_radix(&hex[3..4].repeat(2), 16).unwrap_or(0) as f32 * 100.0 / 255.0,),
        6 => (u8::from_str_radix(&hex[0..2], 16).unwrap_or(0),
            u8::from_str_radix(&hex[2..4], 16).unwrap_or(0),
            u8::from_str_radix(&hex[4..6], 16).unwrap_or(0),
            100.0,),
        8 => (u8::from_str_radix(&hex[0..2], 16).unwrap_or(0),
            u8::from_str_radix(&hex[2..4], 16).unwrap_or(0),
            u8::from_str_radix(&hex[4..6], 16).unwrap_or(0),
            u8::from_str_radix(&hex[6..8], 16).unwrap_or(0) as f32 * 100.0 / 255.0,),
        _ => (0, 0, 0, 100.0),
    };
    [r as f32/255.0, g as f32/255.0, b as f32/255.0, a/100.0,]
}

#[derive(Debug, Clone, PartialEq)]
pub enum Colour {
    Named(String),
    Hex(String),
    Function(Function),
}

impl Default for Colour{fn default() -> Self {
    Colour::Named(String::from("black"))
}}

impl Colour {pub fn transparent() -> Self{
    Colour::Hex(String::from("#00000000"))
}
    pub fn parsecv(csv: &Vec<ComponentValue>) -> Option<Colour> {
        if cvs.len() !=1 {return None;}
    
    match &cvs[0] {ComponentValue::Token(CSSToken::Ident(name)) if getnamedcolour(name).is_some() =>{Some(Colour::Named(name.clone()))}
    ComponentValue::Token(CSSToken::Hash(HashToken{value:val,..}))=>{Some(Colour::Hex(val.clone()))}
    ComponentValue::Function(func) if iscolourfunction(&func.0)=>{Some(Colour::Function(func.clone()))}
    _=>None,
    }
    }

    pub fn used(&self) -> [f32; 4] {
        match self {
            Colour::Named(name) => {
                if let Some(hex) = getnamedcolour(name) {hexToRGB(hex)} else{[0.0, 0.0, 0.0, 0.0]}
            }
            Colour::Hex(hex) => hexToRGB(hex),
            Colour::Function(func)=>{todo!("Need to implement colour funcs", func)}
        }
    }
}
pub type UsedColour = [f32; 4];