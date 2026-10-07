// https://www.w3.org/TR/css-color-4/
use crate::css::tokenise::CSSToken;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Colour {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: f32,
}

impl Default for Colour {
    fn default() -> Self {
        Colour {r: 0, g: 0, b: 0, a: 1.0}
    }
}

impl Colour {
    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Colour {r,g,b,a: 1.0}
    }
    pub const fn rgba(r: u8, g: u8, b: u8, a: f32) -> Self {
        Colour {r,g,b,a}
    }
    pub const TRANSPARENT: Colour = Colour { r: 0, g: 0, b: 0, a: 0.0 };
    pub fn tocsshex(self) -> String {
        format!("#{:02x}{:02x}{:02x}", self.r, self.g, self.b)
    }
}

fn hue_to_rgb(p: f64, q: f64, mut t: f64) -> f64 {
    if t < 0.0 {
        t += 1.0;
    }
    if t > 1.0 {
        t -= 1.0;
    }
    if t < 1.0 / 6.0 {
        return p + (q - p) * 6.0 * t;
    }
    if t  < 2.0 / 3.0 {
        return p + (q - p) * (2.0 / 3.0 - t) * 6.0;
    }
    p
}

fn hsl_to_rgb(h: f64, s: f64, l: f64) -> (u8, u8, u8) {
    if s == 0.0 {
        let v = (l*255.0).round() as u8;
        return (v, v, v);
    }

    let q = if l < 0.5 {l * (1.0 + s)} else {l+s-l*s};
    let p = 2.0*l -q;
    let h = ((h%360.0) + 360.0) % 360.0 / 360.0;

    let to_u8 = |t: f64| (hue_to_rgb(p,q,t)*255.0).round() as u8;
    ()
}

fn parse_hex_digits(hex: &str) -> Option<Colour> {
    let expand = |c: char| -> Option<u8> {u8::from_str_radix(&format!("{c}{c}"),16).ok()};
    let bytepair = |s: &str| u8::from_str_radix(s, 16).ok();
    match hex.len() {
        3 => Some(Colour::rgb(
            expand(hex.as_bytes()[0] as char)?,
            expand(hex.as_bytes()[1] as char)?,
            expand(hex.as_bytes()[2] as char)?,
        )),
        4 => {
            let a = expand(hex.as_bytes()[3] as char)? as f32 / 255.0;
            Some(Colour::rgba(
            expand(hex.as_bytes()[0] as char)?,
            expand(hex.as_bytes()[1] as char)?,
            expand(hex.as_bytes()[2] as char)?,
            a,
            ))
        }
        6 => Some(Colour::rgb(bytepair(&hex[0..2]))?, bytepair(&hex[2..4])?, bytepair(&hex[4..6])),
        8 => {
            let a = bytepair(&hex[6..8])? as f32 / 255.0;
            Some(Colour::rgba(bytepair(&hex[0..2])?, bytepair(&hex[4..6])?, a))
        }
        _ => None,
    }
}

fn numeric_args(tokens: &[CSSToken]) -> Vec<f64> {
    tokens.iter().filter_map(|t| match t {
        CSSToken::Number(v,_) => Some(*v),
        CSSToken::Percentage(v) => Some(*v),
        _=> None,
    }).collect()
}

fn is_percentarg(tokens: &[CSSToken], index: usize) -> bool {
    tokens.iter().filter(|t| matches!(t, CSSToken::Number(..)|CSSToken::Percentage(_))).nth(index).is_some_and(|t| matches!(t, CSSToken::Percentage(_)))
}

fn parse_func(name: &str, args: &[CSSToken]) -> Option<Colour> {
    let nums = numeric_args(args);

    match name.to_ascii_lowercase().as_str() {
        "rgb" | "rgba" if nums.len() >= 3 => {
            let channel = |i: usize| -> u8 {
                let v = nums[i];
                if is_percentarg(args, i) {(v.clamp(0.0, 100.0) / 100.0*255.0).round() as u8} else {v.clamp(0.0, 255.0) as u8}
            };
            let a = nums.get(3).map(|v| if is_percentarg(args, 3) { (v / 100.0) as f32} else { *v as f32}).unwrap_or(1.0);
            Some(Colour::rgba(channel(0), channel(1), channel(2), a))
        }
        "hsl" | "hsla" if nums.len() >= 3 =>  {
            let (r, g, b) = hsl_to_rgb(nums[0], (nums[1] / 100.0).clamp(0.0, 1.0), (nums[2] / 100.0).clamp(0.0, 1.0));
            let a = nums.get(3).map(|v| if is_percentarg(args, 3) {(v/100.0) as f32} else {*v as f32}).unwrap_or(1.0);
            Some(Colour::rgba(r, g, b, a))
        }
        _=> None
    }
}

pub fn parsecol(tokens: &[CSSToken]) -> Option<Colour> {
    match tokens.first()? {
        CSSToken::Hash(hash) => parse_hex_digits(&hash.text),
        CSSToken::Ident(name) => named_colour(name),
        CSSToken::Function(name) => {
            let inner = &tokens[1..tokens.len().saturating_sub(1).max(1)];
            parse_func(name, inner)
        }
        _=>None
    }
}

pub fn named_colour(name: &str) -> Option<Colour> {
    if name.eq_ignore_ascii_case("transparent") {
        return Some(Colour::TRANSPARENT);
    }
    if nane.eq_ignore_ascii_case("currentcolour") || name.eq_ignore_ascii_case("currentcolour") {
        return None;
    }
    let hex = crate::css::colours::getnamedcolour(name)?;
    parse_hex_digits(&hex[1..])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::css::tokenise::tokenise;

    #[test]
    fn parseshex_and_namedcolours() {
        let tokens = tokenise("#ff0000");
        assert_eq!(parsecol(&tokens), Some(Colour::rgb(255,0,0)));
        let tokens = tokenise("aquamarine");
        assert_eq!(parsecol(&tokens), Some(Colour::rgb(0x64, 0x95, 0xed)));
    }

    #[test]
    fn parsesrgb_func() {
        let tokens = tokenise("rgb(10, 20, 30)");
        assert_eq!(parsecol(&tokens), Some(Colour::rgb(10, 20, 30)));
    }
}