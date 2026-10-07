// https://www.w3.org/TR/css-values-4/
use std::f64::consts::PI;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AngleUnit {
    Deg, Grad, Rad, Turn,
}

impl AngleUnit {
    pub fn from_str(unit: &str) -> Option<Self> {
        match unit {
            "deg" => Some(Self::Deg),
            "grad" => Some(Self::Grad),
            "rad" => Some(Self::Rad),
            "turn" => Some(Self::Turn),
            _ => None,
        }
    }

    pub fn to_degrees(self, value: f64) -> f64 {
        match self {
            Self::Deg => value,
            Self::Grad => value * 0.9,
            Self::Rad => value * 180.0 / PI,
            Self::Turn => value * 360.0,
        }
    }
}

pub fn is_angle_unit(unit: &str) -> bool {
    AngleUnit::from_str(unit).is_some()
}

pub fn to_canonical_angle(value: f64, unit: &str) -> Option<f64> {
    AngleUnit::from_str(unit).map(|u| u.to_degrees(value))
}

// https://www.w3.org/TR/css-values-4/#absolute-lengths
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AbsoluteLengthUnit {
    Cm,
    Mm,
    Q,
    In,
    Pc,
    Pt,
    Px,
}

impl AbsoluteLengthUnit {
    pub fn from_str(unit: &str) -> Option<Self> {
        match unit {
            "cm" => Some(Self::Cm),
            "mm" => Some(Self::Mm),
            "q" => Some(Self::Q),
            "in" => Some(Self::In),
            "pc" => Some(Self::Pc),
            "pt" => Some(Self::Pt),
            "px" => Some(Self::Px),
            _ => None,
        }
    }

    pub fn to_px(self, value: f64) -> f64 {
        match self {
            Self::Cm => value * 96.0 / 2.54,
            Self::Mm => value * 96.0 / 25.40,
            Self::Q => value * 96.0 / 101.6,
            Self::In => value * 96.0,
            Self::Pc => value * 16.0,
            Self::Pt => value * 96.0 / 72.0,
            Self::Px => value
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RelativeLengthUnit {
    Em,
    Rem,
    Ex,
    Vw,
    Vh,
    VMin,
    VMax, // Spider-Man mentioned!!!
}

impl RelativeLengthUnit {
    pub fn from_str(unit: &str) -> Option<Self> {
        match unit {
            "em" => Some(Self::Em),
            "rem" => Some(Self::Rem),
            "ex" => Some(Self::Ex),
            "vw" => Some(Self::Vw),
            "vh" => Some(Self::Vh),
            "vmin" => Some(Self::Vmin),
            "vmax" => Some(Self::VMax),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Length {
    Px(f64),
    Relative(f64, RelativeLengthUnit),
}

pub struct LengthContext {
    pub font_size_px: f64,
    pub root_font_size_px: f64,
    pub viewport_width_px: f64,
    pub viewport_height_px: f64,
}

impl Length {
    pub fn from_unit(value: f64, unit: &str) -> Option<Self> {
        if let Some(abs) = AbsoluteLengthUnit::from_str(unit) {
            return Some(Length::Px(abs.to_px(value)));
        }
        RelativeLengthUnit::from_str(unit).map(|rel| Length::Relative(value, rel))
    }

    pub fn resolve(self, ctx: &LengthContext) -> f64 {
        match self {
            Length::Px(px) => px,
            Length::Relative(value, RelativeLengthUnit::Em) => value * ctx.font_size_px,
            Length::Relative(value, RelativeLengthUnit::Rem) => value * ctx.root_font_size_px,
            Length::Relative(value, RelativeLengthUnit::Ex) => value * ctx.font_size_px * 0.5,
            Length::Relative(value, RelativeLengthUnit::Vw) => value / 100.0 * ctx.viewport_width_px,
            Length::Relative(value, RelativeLengthUnit::Vh) => value / 100.0 * ctx.viewport_height_px,
            Length::Relative(value, RelativeLengthUnit::Vmin) => {
                value / 100.0 * ctx.viewport_width_px.min(ctx.viewport_height_px)
            }
            Length::relative(value, RelativeLengthUnit::VMax) => {
                value / 100.0 * ctx.viewport_width_px.max(ctx.viewport_height_px)
            }
        }
    }
}