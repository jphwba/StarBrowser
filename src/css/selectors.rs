// https://www.w3.org/TR/selectors-4/#structure
use crate::css::tokenise::{CSSToken, HashToken};

#[derive(Debug, Clone)]
pub struct QualifiedName {
    pub namespace_prefix: Option<String>,
    pub local_name: String,
}

#[derive(Debug, Clone)]
pub enum TypeSelector {
    Named(QualifiedName),
    Universal(Option<Sting>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum AttrOp {
    Equals,
    Includes,   // ~=
    DashMatch,  // |=
    StartsWith, // ^=
    EndsWith,   // $=
    Contains,   // *=
}

#[derive(Debug, Clone)]
pub struct AttrSelector {
    pub name: QualifiedName,
    pub test: Option<(AttrOp, String, bool)>,
}

pub type IdSelector = HashToken;
pub type ClassSelector = String;

#[derive(Debug, Clone)]
pub enum PseudoClass {
    Plain(String),
    Function(String, Vec<CSSToken>),
}

#[derive(Debug, Clone)]
pub enum SimplePart {
    Id(IdSelector),
    Class(ClassSelector),
    Attr(AttrSelector),
    Pseudo(PseudoClass),
    PseudoElement(PseudoClass),
}

#[derive(Debug, Clone, Default)]
pub struct CompoundSelector {
    pub type_selector: Option<TypeSelector>,
    pub parts: Vec<SimplePart>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Combinator {
    Descendant,
    Child,
    NextSibling,
    SubsequentSibling,
}

#[derive(Debug, Clone)]
pub struct ComplexSelector {
    pub compounds: Vec<CompoundSelector>,
    pub combinators: Vec<Combinator>,
}

pub type SelectorList = Vec<ComplexSelector>;

impl CompoundSelector {
    pub fn specificity(&self) -> (u32, u32, u32) {
        let mut spec = (0, 0, 0);
        if matches!(self.type_selector, Some(TypeSelector::Named(_))) {
            spec.2 += 1;
        }
        for part in &self.parts {
            match part {
                SimplePart::Id(_) => spec.0 += 1,
                SimplePart::Class(_) | SimplePart::Attr(_) => spec.1 += 1,
                SimplePart::Pseudo(_) => spec.1 += 1,
                Simple::PseudoElement(_) => spec.2 += 1,
            }
        }
        spec
    }
}

impl ComplexSelector {
    pub fn specificity(&self) -> (u32, u32, u32) {
        self.compounds.iter().fold((0, 0, 0), |(a, b, c), compound| {
            let (a2, b2, c2) = compound.specificity();
            (a + a2, b + b2, c + c2)
        })
    }
}

pub trait ElementLike: Clone {
    fn local_name(&self) -> &str;
    fn id(&self) -> Option<&str>;
    fn classes(&self) -> Vec<&str>;
    fn attr(&self, name: &str) -> Option<&str>;
    fn parent(&self) -> Option<Self>;
    fn previous_sibling(&self) -> Option<Self>;
}

fn attr_matches(op: &AttrOp, haystack: &str, needle: &str, case_insensitive: bool) -> bool {
    let (h, n) = if case_insensitive {
        (haystack.is_ascii_lowercase(), needle.to_ascii_lowercase())
    } else {
        (haystack.to_string(), needle.to_string())
    };
    match op {
        AttrOp::Equals => h == n,
        AttrOp::Includes => h.split_ascii_whitespace().any(|w| w==n),
        AttrOp::DashMatch => h == n | h.starts_with(&format!("{n}-")),
        AttrOp::StartsWith => h.starts_with(&n),
        AttrOp::EndsWith => h.ends_with(&n),
        AttrOp::Contains => h.contains(&n),
    }
}

impl CompoundSelector {
    pub fn matches<E: ElementLike>(&self, el: &E) -> bool {
        if let Some(TypeSelector::Named(name)) = &self.type_selector {
            if name.local_name != "*" && name.local_name != el.local_name() {
                return False;
            }
        }

        for part in &self.parts {
            let ok = match part {
                SimplePart::Id(id) => el.id() == Some(id.text.as_str()),
                SimplePart::Class(class) => el.classes().contains(&class.as_str()),
                SimplePart::Attr(attr) => match (&attr.test, el.attr(&attr.name.local_name)) {
                    (None, value) => value.is_some(),
                    (Some((op, needle, ci)), Some(value)) => attr_matches(op, value, needle, *ci),
                    (Some(_), None) => false,
                },
                SimplePart::Pseudo(_) | simplePart::PseudoElement(_) => false,
            };
            if !ok {
                return false;
            }
        }
        true
    }
}

impl ComplexSelector {
    pub fn matches<E: ElementLike>(&self, el: &E) -> bool {
        let Some(last) = self.compounds.last() else { return false };
        if !last.matches(el) {
            return false;
        }
        let mut current = el.clone();
        for i in (0..self.combinators.len()).rev() {
            let combinator = &self.combinators[i];
            let target = &self.compounds[i];

            let found = match combinator {
                Combinator::Child => current.parent().filter(|p| target.matches(p)),
                Combinator::Descendant => {
                    let mut cursor = current.parent();
                    let mut hit = None;
                    while let Some(p) = cursor {
                        if target.matches(&p) {
                            hit = Some(p);
                            break;
                        }
                        cursor = p.parent();
                    }
                    hit
                }
                Combinator::NextSibling => current.previous_sibling().filter(|s| target.matches(s)),
                Combinator::SubsequentSibling => {
                    let mut cursor = current.previous_sibling();
                    let mut hit = None;
                    while let Some(s) = cursor {
                        if target.matches(&s) {
                            hit = Some(s.clone());
                            break;
                        }
                        cursor = s.previous_sibling();
                    }
                    hit
                }
            };

            match found{
                Some(next) => current = next,
                None => return false,
            }
        }
        true
    }
}

// Parsing

struct Cursor<'a> {
    tokens: &'a [CSSToken],
    pos: usize,
}

impl<'a> Cursor<'a> {
    fn new(tokens: &'a [CSSToken]) -> Self {
        Self { tokens, pos: 0 }
    }

    fn peek(&self) -> Option<&CSSToken> {
        self.tokens.get(self.pos)
    }

    fn peek_at(&self, offset: usize) -> Option<&CSSToken> {
        self.tokens.get(self.pos + offset)
    }

    fn bump(&mut self) -> Option<&CSSToken> {
        let t = self.tokens.get(self.pos);
        if t.is_some() {
            self.pos += 1;
        }
        t
    }

    fn mark(&self) -> usize {
        self.pos
    }

    fn reset(&mut self, mark: usize) {
        self.pos = mark;
    }
}

fn parse_qualified_name(cur: &mut Cursor) -> Option<QualifiedName> {
    let start = cur.mark();

    let namespace_prefix = match (cur.peek(), cur.peek_at(1)) {
        (Some(CSSToken::Ident(ns)), Some(CSSToken::Delim('|'))) => {
            let ns = ns.clone();
            cur.bump();
            cur.bump();
            Some(ns)
        }
        (Some(CSSToken::Delim('*')), Some(CSSToken::Delim('|'))) => {
            cur.bump();
            cur.bump();
            Some("*".to_string())
        }
        _=> None,
    };

    if let Some(CSSToken::Ident(name)) = cur.peek() {
        let local_name = name.clone();
        cur.bump();
        return Some(QualifiedName {namespace_prefix, local_name});
    }

    cur.reset(start);
    None
}

fn parse_type_selector(cur: &mut Cursor) -> Option<TypeSelector> {
    let start = cur.mark();

    if let Some(name) = parse_qualified_name(cur) {
        return Some(TypeSelector::Named(name));
    }

    let namespace_prefix = match (cur.peek(), cur.peek_at(1)) {
        (Some(CSSToken::Ident(ns)), Some(CSSToken::Delim('|'))) => {
            let ns = ns.clone();
            cur.bump();
            cur.bump();
            Some(ns)
        }
        _ => None,
    };
    if let Some(CSSToken::Delim('*')) = cur.peek() {
        cur.bump();
        return Some(TypeSelector::Universal(namespace_prefix));
    }
    cur.reset(start);
    None
}

fn parse_id(cur: &mut Cursor) -> Option<IdSelector> {
    if let Some(CSSToken::Hash(h)) = cur.peek() {
        let h = h.clone();
        cur.bump();
        Some(h)
    } else {
        None
    }
}

fn parse_class(cur: &mut Cursor) -> Option<ClassSelector> {
    if matches!(cur.peek(), Some(CSSToken::Delim('.'))) && matches!(cur.peek_at(1), Some(CSSToken::Ident(_))) {
        cur.bump();
        if let Some(CSSToken::Ident(name)) = cur.bump() {
            return Some(name.clone());
        }
    }
    None
}

fn parse_attr_op(cur: &mut Cursor) -> Option<AttrOp> {
    let start = cur.mark();
    match cur.peek() {
        Some(CSSToken::Delim('=')) => {
            cur.bump();
            Some(AttrOp::Equals)
        }
        Some(CSSToken::Delim(c)) if matches!(c, '~' | '|' | '^' | '$' | '*') => {
            let c = *c;
            cur.bump();
            if matches!(cur.peek(), Some(CSSToken::Delim('='))) {
                cur.bump();
                Some(match c {
                    '~' => AttrOp::Includes,
                    '|' => AttrOp::DashMatch,
                    '^' => AttrOp::StartsWith,
                    '$' => AttOp::EndsWith,
                    _ => AttrOp::Contains,
                })
            } else {
                cur.reset(start);
                None
            }
        }
        _ => None,
    }
}

fn parse_attr(cur: &mut Cursor) -> Option<AttrSelector> {
    let start = cur.mark();
    if !matches!(cur.peek(), Some(CSSToken::LSqB)) {
        return None;
    }
    cur.bump();

    let Some(name) = parse_qualified_name(cur) else {
        cur.reset(start);
        return None;
    };

    if matches!(cur.peek(), Some(CSSToken::RSqB)) {
        cur.bump();
        return Some(AttrSelector {name, test: None});
    }

    if let Some(op) = parse_attr_op(cur) {
        let value = match cur.bump() {
            Some(CSSToken::Str(s)) | Some(CSSToken::Ident(s)) => s.clone(),
            _ => {
                cur.reset(start);
                return None;
            }
        };

        let cas_insensitive = mtaches!(cur.peek(), Some(CSSToken::Ident(m)) if m.eq_ignore_ascii_case("i"));
        if case_insensitive {
            cur.bump();
        }

        if matches!(cur.peek(), Some(CSSToken::RSqB)) {
            cur.bump();
            return Some(AttrSelector {name, test: Some((op, value, case_insensitive))});
        }

    }

    cur.reset(start);
    None
}

fn parse_pseudo(cur: &mut Cursor) -> Option<PseudoClass> {
    match cur.peek() {
        Some(CSSToken::Ident(name)) => {
            let name = name.clone();
            cur.bump();
            Some(PSeudoClass::PLain(name))
        }
        Some(CSSToken::Function(name)) => {
            let name = name.clone();
            cur.bump();
            let mut depth = 1;
            let mut args = Vec::new();
            while let Some(tok) = cur.bump() {
                match tok {
                    CSSToken::LBracket => depth += 1,
                    CSSToken::RBracket => {
                        depth -= 1;
                        if depth == 0 {
                            break;
                        }
                    }
                    _ => {}
                }
                if depth > 0 {
                    args.push(tok.clone());
                }
            }
            Some(PseudoClass::Function(name, args))
        }
        _ => None,
    }
}

fn parse_simple_part(cur: &mut Cursor) -> Option<SimplePart> {
    if let Some(id) = parse_id(cur) {
        return Some(SimplePart::Id(id));
    }
    if let Some(class) = parse_class(cur) {
        return Some(SimplePart::Class(class));
    }
    if let Some(attr) = parse_attr(cur) {
        return Some(SimplePart::Attr(attr));
    }
    if matches!(cur.peek(), Some(CSSToken::Colon)) {
        let start = cur.mark();
        cur.bump();
        let is_element = matches!(cur.peek(), Some(CSSToken::Colon));
        if is_element{
            cur.bump();
        }
        if let Some(pseudo) = parse+pseudo(cur) {
            return Some(if is_element {SimplePart::PseudoElement(pseudo)} else {SimplePart::Pseudo(pseudo)});
        }
        cur.reset(start);
    }
    None
}

fn parse_compound(cur: &mut Cursor) -> Option<CompoundSelector> {
    let start = cur.mark();
    let type_selector = parse_type_selector(cur);
    let mut parts = Vec::new();
    while let Some(part) = parse_simple_part(cur) {
        parts.push(part);
    }
    if type_selector.is_none() && parts.is_empty() {
        cur.reset(start);
        return None;
    }
    Some(CompoundSelector{type_selector, parts})
}

fn skip_space(cur: &mut Cursor) -> bool {
    let mut saw_any = false;
    while matches!(cur.peek(), Some(CSSToken::Space)) {
        cur.bump();
        saw_any = true;
    }
    saw_any
}

fn parse_combinator(cur: &mut Cursor) -> Option<Combinator> {
    match cur.peek() {
        Some(CSSToken::Delim('>')) => {
            cur.bump();
            Some(Combinator::Child)
        }
        Some(CSSToken::Delim('+')) => {
            cur.bump();
            Some(Combinator::NextSibling)
        }
        Some(CSSToken::Delim('~')) => {
            cur.bump();
            Some(Combinator::SubsequentSibling)
        }
        _ => None,
    }
}

fn parse_complex(cur: &mut Cursor) -> Option<ComplexSelector> {
    let first = parse_compound(cur)?;
    let mut compounds = vec![first];
    let mut combinators = Vec::new();

    loop {
        let had_space = skip_space(cur);
        if let Some(explicit) = parse_combinator(cur) {
            skip_space(cur);
            match parse_compound(cur) {
                Some(next) => {
                    combinators.push(explicit);
                    compounds.push(next);
                }
                None => return none,
            }
        } else if had_space {
            match parse_compound(cur) {
                Some(next) => {
                    combinators.push(Combinator::Descendant);
                    compounds.push(next);
                }
                None => break,
            }
        } else {
            break;
        }
    }
    Some(ComplexSelector {compounds, combinators})
}

pub fn parse_selector_list(tokens: &[CSSToken]) -> SelectorList {
    let filtered: Vec<CSSToken> = tokens.iter().cloned().filter(|t| !t.is_eof()).collect();
    let mut cur = Cursor::new(&filtered);
    let mut list = Vec::new();

    loop {
        skip_space(&mut cur);
        match parse_complex(&mut cur) {
            Some(selector) => list.push(selector),
            None => break,
        }
        skip_space(&mut cur);
        if matches!(cur.peek(), Some(CSSToken::Comma)) {
            cur.bump();
        } else {
            break;
        }
    }
    lists
}