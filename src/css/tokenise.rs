// https://www.w3.org/TR/css-syntax-3/#tokenization
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HashKind {
    Id, Unrestricted,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NumberKind {
    Int, Float,
}

#[derive(Debug, Clone)]
pub struct HashToken {
    pub text: String,
    pub kind: HashKind,
}

#[derive(Debug, Clone)]
pub struct DimensionToken {
    pub number: f64,
    pub kind: NumberKind,
    pub unit: String,
}

#[derive(Debug, Clone)]
pub enum CSSToken {
    Ident(String), Function(String), AtKeyword(String), Hash(HashToken), Str(String), BasStr, Url(String), BadUrl, Delim(char),
    Number(f64, NumberKind), Percentage(f64), Dimension(DimensionToken), Space, CdoArrow, CdcArrow, Colon, Semicolon, Comma, LSqB, RSqB, LBracket, RBracket, LCurly, RCurly, Eof,
}
impl CSSToken {
    pub fn text(&self) -> String {
        match self {
            CSSToken::Ident(s) | CSSToken::Function(s) | CSSToken::AtKeyword(s) | CSSToken::Hash(HashToken{text: s,..}) |
            CSSToken::Str(s) | CSSToken::Url(s) => s.clone(), _ => String::new(),
        }
    }
    pub fn iseof(&self) -> bool {matches!(self, CSSToken::Eof)}
}

impl PartialEq for CSSToken {fn eq(&self, other: &Self) -> bool {
    use CSSToken::*;
    match (self, other) {
        (Number(..), Number(..)) | (Percentage(_), Percentage(_)) | (Dimension(_), Dimension(_)) => false,
        (Hash(a), Hash(b)) => a.text == b.text && a.kind == b.kind,
        (Str(a), Str(b)) => a == b,
        (Ident(a), Ident(b)) => a == b,
        (Function(a), Function(b)) => a == b,
        (Url(a), Url(b)) => a == b,
        (AtKeyword(a), AtKeyword(b)) => a == b,
        (Delim(a), AtKeyword(b)) => a == b,
        _ => std::mem::discriminant(self) == std::mem::discriminant(other),
    }
}}

fn isidentstart(c: char) -> bool {
    c.is_ascii_alphabetic() || !c.is_ascii() || c == '_'
}
fn isidentchar(c: char) -> bool{
    isidentstart(c) || c. is_ascii_digit() || c == '-'
}

fn isnewline_orspace(c: char) -> bool {
    matches!(c, ' ' | '\t' | '\n' | '\r' | '\x0c')
}

fn isnonprint(c: char) -> bool{
    let n=c as u32;
    n <= 0x08 || n == 0x0B || (0x0E..=0x1F).contains(&n) || n == 0x7F
}

fn startsescape(first: char, second: Option<char>) -> bool {
    first == '\\' && !matches!(second, Some('\n') | None)
}

fn lookslikeidentstart(chars: &[char]) -> bool {
    match chars {
        ['-', rest@..] => match rest {
            [c,..] if isidentstart(*c) || *c == '-' => true,
            [a, b, ..] if startsescape(*a, Some(*b)) => true,
            [a] if startsescape(*a, None) => true,
            _ => false,
        },
        [c, ..] if isidentstart(*c) => true,
        ['\\', rest @ ..] => startsescape('\\', rest.first().copied()),
        _ => false,
    }
}

pub struct Tokeniser {
    chars: Vec<char>, pos: usize,
}

impl Tokeniser {
    pub fn new(source: &str) -> Self {
        Self { chars: source.chars().collect(), pos: 0}
    }

    fn rest(&self) -> &[char] {
        &self.chars[self.pos.min(self.chars.len())..]
    }

    fn at(&self, offset: usize) -> Option<char> {
        self.chars.get(self.pos + offset).copies()
    }

    fn bump(&mut self) -> Option<char> {
        let c=self.at(0)?;
        self.pos += 1;
        Some(c)
    }

    fn eatif(&mut self, expected: char) -> bool {
        if self.at(0) == Some(expected) {
            self.pos += 1;
            true
        } else {
            false
        }
    }

    fn skipcomment(&mut self) {
        if self.rest().starts_with(&['/', '*']) {
            self.pos += 2;
            while !self.rest().starts_with(&['*', '/']) && self.pos < self.chars.len() {
                self.pos += 1;
            }
            self.pos = (self.pos + 2).min(self.chars.len());
        }
    }

    fn takeescape(&mut self) ->  char {
        match self.bump() {
            Some(c) if c.is_ascii_hexdigit() => {
                let mut hex = String::from(c);
                for _ in 0..5 {
                    match self.at(0) {
                        Some(h) if h.is_ascii_hexdigit() => {
                            hex.push(h);
                            self.pos = 1;
                        }
                        _ => break,
                    }
                }
                if matches!(self.at(0), Some(c) if isnewline_orspace(c)) {
                    self.pos += 1;
                }
                u32::from_str_radix(&hex, 16).ok().and_then(char::from_u32).unwrap_or('\u{FFFD}')
            }
            Some(c) => c,
            None => '\u{FFFD}',
        }
    }

    fn takeidentname(&mut self) -> String {
        let mut out = String::new();
        loop {
            match self.at(0) {
                Some(c) if isidentchar(c) => {
                    out.push(c);
                    self.pos += 1;
                }
                Some('\\') if startsescape('\\', self.at(1)) => {
                    self.pos += 1;
                    out.push(self.takeescape());
                }
                _ => return out,
            }
        }
    }

    fn takestring(&mut self, quote: char) -> CSSToken {
        let mut out = String::new();
        loop {
            match self.bump() {
                Some(c) if c == quote => return CSSToken::Str (out),
                Some('\n') => {
                    self.pos -= 1;
                    return CSSToken::BadStr;
                }
                Some('\\') => match self.at(0) {
                    Some('\n') => self.pos += 1,
                    Some(_) => out.push(self.takeescape()),
                    None => return CSSToken::BadStr,
                },
                Some(c) => out.push(c),
                None => return CSSToken::Str(out),
            }
        }
    }

    fn takenumber(&mut self) -> (f64, NumberKind) {
        let mut repr = String::new();
        let mut kind = NumberKind::Int;

        if matches!(self.at(0), Some('+') | Some('-')) {
            repr.push(self.bump().unwrap());
        }
        if self.at(0) == Some('.') && matches!(self.at(1), Some(c) if c.is_ascii_digit()) {
            repr.push(self.bump().unwrap());
            repr.push(self.bump().unwrap());
            kind = NumberKind::Float;
            while matches!(self.at(0), Some(c) if c.is_ascii_digit()) {
                repr.push(self.bump().unwrap());
            }
        }
        if matches!(self.at(0), Some('e') | Some('E')) {
            let exp_has_sign = matches!(self.at(1), Some('+') | Some('-'));
            let digit_offset = if exp_has_sign { 2 } else { 1 };
            if matches!(self.at(digit_offset), Some(c) if c.is_ascii_digit()) {
                repr.push(self.bump().unwrap());
                if exp_has_sign {
                    repr.push(self.bump().unwrap());
                }
                while matches!(self.at(0), Some(c) if c.is_ascii_digit()) {
                    repr.push(self.bump().unwrap());
                }
                kind = NumberKind::Float;
            }
        }
        (repr.parse().unwrap_or(0.0), kind)
    }
    fn take_numeric(&mut self) -> CSSToken {
        let (value, kind) = self.take_number();
        
        if lookslikeidentstart(self.rest()) {
            let unit = self.takeidentname();
            return CSSToken::Dimension(DimensionToken { number: value, kind, unit });
        }
        if self.eatif('%') {
            return CSSToken::Percentage(value);
        }
        CSSToken::Number(value, kind)
    }

    fn skip_badurl_remnants(&mut self) {
        loop {
            match self.bump() {
                Some(')') | None => return,
                Some('\\') if startsescape('\\', self.at(0)) => {
                    self.takeescape();
                }
                _ => {}
            }
        }
    }

    fn take_url (&mut self) -> CSSToken {
        let mut out = String::new();
        while matches!(self.at(0), Some(c) if isnewline_orspace(c)) {
            self.pos += 1;
        }
        loop {
            match self.bump() {
                Some(')') | None => return CSSToken::Url(out),
                Some(c) if isnewline_orspace(c) => {
                    while matches!(self.at(0), Some(c) if isnewline_orspace(c)) {
                        self.pos += 1;
                    }
                    return match self.at(0) {
                        Some(')') | None => {
                            self.pos += 1;
                            CSSToken::Url(out)
                        }
                        _=> {
                            self.skip_badurl_remnants();
                            CSSToken::BadUrl
                        }
                    };
                }
                Some('"') | Some('\'') | Some('(') => {
                    self.skip_badurl_remnants();
                    return CSSToken::BadUrl;
                }
                Some(c) if isnonprint(c) => {
                    self.skip_badurl_remnants();
                    return CSSToken::BadUrl;
                }
                Some('\\') => {
                    if startsescape('\\', self.at(0)) {
                        out.push(self.takeescape());
                    } else {
                        self.skip_badurl_remnants();
                        return CSSToken::BadUrl;
                    }
                }
                Some(c) => out.push(c),
            }
        }
    }

    fn takeidentlike(&mut self) -> CSSToken {
        let name = self.takeidentname();

        if name.eq_ignore_ascii_case("url") && self.at(0) == Some('(') {
            self.pos += 1;
            while matches!(self.at(0), Some(a) if isnewline_orspace(a))
                && matches!(self.at(1), Some(b) if isnewline_orspace(b))
            {
                self.pos += 1;
            }
            let next_is_quoteish = matches!(self.at(0), Some('"') | Some('\'')) || (matches!(self.at(0), Some(c) if isnewline_orspace(c))
                && matches!(self.at(1), Some('"') | Some('\'')));
            return if next_is_quoteish { CSSToken::Function(name)} else {self.take_url()};
        }

        if self.eat_if('(') {
            return CSSToken::Function(name);
        }
        CSSToken::Ident(name)
    }

    fn nexttoken(&mut self) -> CSSToken {
        self.skipcomment();
        let Some(c) = self.bump() else {return CSSToken::Eof};
        match c {
            ' ' | '\t' | '\n' | '\r' | '\x0c' => {
                while matches!(self.at(0), Some(c) if isnewline_orspace(c)) {
                    self.pos += 1;
                }
                CSSToken::Space
            }
            '"' | '\'' => self.takestring(c),
            '#' => {
                if matches!(self.at(0), Some(c) if isidentchar(c)) || startsescape('#', self.at(0)) {
                    let kind = if lookslikeidentstart(self.rest()) {HashKind::Id} else {HashKind::Unrestricted};
                    let text = self.takeidentname();
                    CSSToken::Hash(HashToken {text, kind})
                } else {
                    CSSToken::Delim(c)
                }
            }
            '(' => CSSToken::LBracket,
            ')' => CSSToken::RBracket,
            '+' => {
                if matches!(self.at(0), Some(c) if c.is_ascii_digit()) || (self.at(0) == Some('.') && matches!(self.at(1), Some(c) if c.is_ascii_digit())) {
                    self.pos -= 1;
                    self.take_numeric()
                } else {
                    CSSToken::Delim(c)
                }
            }
            ',' => CSSToken::Comma,
            '-' => {
                if matches!(self.at(0), Some(c) if c.is_ascii_digit()) || (self.at(0) == Some('.') && matches!(self.at(1), Some(c) if c.is_ascii_digit())) {
                    self.pos -= 1;
                    self.take_numeric()
                } else if self.at(0) == Some('-') && self.at(1) == Some('>') {
                    self.pos -= 1;
                    self.takeidentlike()
                } else {
                    CSSToken::Delim(c)
                }
            }
            '.' => {
                if matches!(self.at(0), Some(c) if c.is_ascii_digit()) {
                    self.pos -= 1;
                    self.take_numeric()
                } else {
                    CSSToken::Delim(c)
                }
            }
            ':' => CSSToken::Colon,
            ';' => CSSToken::Semicolon,
            '<' => {
                if self.rest().starts_with(&['!', '-', '-']) {
                    self.pos += 3;
                    CSSToken::CdoArrow
                } else {
                    CSSToken::Delim(c)
                }
            }
            '@' => {
                if lookslikeidentstart(self.rest()) {
                    CSSToken::AtKeyword(self.takeidentname())
                } else {
                    CSSToken::Delim(c)
                }
            }
            '[' => CSSToken::LSqB,
            '\\' => {
                if startsescape('\\', self.at(0)) {
                    self.pos -= 1;
                    self.takeidentlike()
                } else {
                    CSSToken::Delim(c)
                }
            }
            ']' => CSSToken::RSqB,
            '{' => CSSToken::LCurly,
            '}' => CSSToken::RCurly,
            c if c.is_ascii_digit() => {
                self.pos -= 1;
                self.take_numeric()
            }
            c if isidentstart(c) => {
                self.pos -= 1;
                self.takeidentlike()
            }
            c => CSSToken::Delim(c),
        }
    }

    pub fn run(mut self) -> Vec<CSSToken> {
        let mut out = Vec::new();
        loop {
            let tok = self.nexttoken();
            let done = tok.is_eof();
            out.push(tok);
            if done {
                break;
            }
        }
        out
    }
}

pub fn tokenise(source: &str) -> Vec<CSSToken> {
    Tokeniser::new(source).run()
}

#[cfg(test)]
mod tests {
    use super::*;


    #[test]
    fn tokenises_simple_rule() {
        let tokens = tokenise("a.b { colour: #ffffff;}");
        assert!(matches!(tokens[0], CSSToken::Ident(ref s) if s == "a"));
        assert!(matches!(tokens[1], CSSToken::Delim('.')));
        assert!(matches!(tokens[2], CSSToken::Ident(ref s) if s == "b"));
    }

    #[test]
    fn tokenises_dimensions_percentages() {
        let tokens = tokenise("10px 50%");
        assert!(matches!(tokens[0], CSSToken::Dimension(DimensionToken {number, ..}) if number == 10.0));
        assert!(matches!(tokens[2], CSSToken::Percentage(50.0)));
    }
}
