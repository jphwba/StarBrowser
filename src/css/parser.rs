// https://www.w3.org/TR/css-syntax-3/#parsing
use crate::css::selector::{parse_selector_list, SelectorList};
use crate::css::tokenise::{tokenise, CSSToken};

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionValue {
    pub name: String,
    pub args: Vec<ComponentValue>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BlockValue {
    pub open: CSSToken,
    pub contents: Vec<ComponentValue>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ComponentValue {
    Token(CSSToken),
    Function(FunctionValue),
    Block(BlockValue),
}

impl ComponentValue {
    pub fn is_space(&self) -> bool {
        matches!(self, ComponentValue::Token(CSSToken::Space))
    }
}

#[derive(Debug, Clone)]
pub struct Declaration {
    pub property: String,
    pub value: Vec<ComponentValue>,
    pub important: bool,
}

#[derive(Debug, Clone)]
pub struct AtRule {
    pub name: String,
    pub prelude: Vec<ComponentValue>,
    pub block: Option<Vec<ComponentValue>>,
}

#[derive(Debug, Clone)]
pub enum Rule {
    Style {selectors: SelectorList, declarations: Vec<Declaration>},
    At(AtRule),
}

#[derive(Debug, Clone, Default)]
pub struct Stylesheet {
    pub rules: Vec<Rule>,
}

fn preprocess(input: &str) -> String {
    let normalised = input.replace("\r\n", "\n").replace('\r', "\n").replace('\u{000C}', "\n").replace('\0', "\u{FFFD}");
    normalised.chars().map(|c| if (0xD800..=0xDFFF).contains(&(c as u32)) {'\u{FFFD}'} else { c}).collect()
}

struct Cursor{
    tokens: Vec<CSSToken>,
    pos: usize,
}

impl Cursor {
    fn from_source(source: &str) -> Self {
        Self {tokens: tokenise(&preprocess(source)), pos: 0}
    }

    fn from_tokens(tokens: Vec<CSSToken>) -> Self {
        Self {tokens, pos: 0}
    }
    fn peek(&self) -> Option<&CSSToken> {
        self.tokens.get(self.pos)
    }
    fn bump(&mut self) -> Option<&CSSToken> {
        let t = self.tokens.get(self.pos).cloned();
        if t.is_some() {
            self.pos += 1;
        }
        t
    }
    fn at_end(&self) -> bool {
        matches!(self.peek(), None | Some(CSSToken::Eof))
    }
}

fn matching_clone(open: &CSSToken) -> CSSToken {
    match open {
        CSSToken::LCurly => CSSToken::RCurly,
        CSSToken::LSqB => CSSToken::RSqB,
        CSSToken::LBracket => CSSToken::RBracket,
        _ => unreachable!("not block opening token"),
    }
}

fn consume_block(cur: &mut Cursor) -> BlockValue {
    let open = cur.bump().expect("caller checked block opening token next");
    let close = matching_close(&open);
    let mut contents = Vec::new();

    loop {
        match cur.peek() {
            None => break,
            Some(t) if *t == close => {
                cur.bump();
                break;
            }
            _ => contents.push(consume_component_value(cur)),
        }
    }
    BlockValue{open, contents}
}

fn consume_function(cur: &mut Cursor, name:String) -> FunctionValue {
    let mut args = Vec::new();
    loop {
        match cur.peek() {
            None | Some(CSSToken::RBracket) => {
                cur.bump();
                break;
            }
            _ => args.push(consume_component_value(cur)),
        }
    }
    FunctionValue {name, args}
}

fn consume_component_value(cur: &mut Cursor) -> ComponentValue {
    match cur.bump() {
        Some(t@ (CSSToken::LCurly | CSSToken::LSqB | CSSToken::LBracket)) => {
            cur.pos -= 1;
            ComponentValue::Block(consume_block(cur))
        }
        Some(CSSToken::Function(name)) => ComponentValue::Function(consume_function(cur, name)),
        Some(other) => ComponentValue::Token(other),
        None => ComponentValue::Token(CSSToken::Eof)
    }
}

fn consume_at_rule(cur: &mut Cursor) -> AtRule {
    let name = match cur.bump() {
        Some(CSSToken::AtKeyword(name)) => name,
        _ => String::new(),
    };
    let mut prelude = Vec::new();
    let mut block = None;

    loop {
        match cur.peek() {
            None | Some(CSSToken::Semicolon) => {
                cur.bump();
                break;
            }
            Some(CSSToken::LCurly) => {
                block = Some(consume_block(cur).contents);
                break
            }
            _ => prelude.push(consume_component_value(cur)),
        }
    }
    AtRule {name, prelude, block}
}

struct QualifiedRule {
    prelude: Vec<ComponentValue>,
    block: Vec<ComponentValue>,
}

fn consume_qualified_rule(cur: &mut Cursor) -> Option<QualifiedRule> {
    let mut prelude = Vec::new();
    loop {
        match cur.peek() {
            None => return None,
            Some(CSSToken::LCurly) => {
                let block = consume_block(cur).contents;
                return Some(QualifiedRule {prelude, block});
            }
            _ => prelude.push(consume_component_value(cur)),
        }
    }
}

fn component_values_to_tokens(values: &[ComponentValue]) -> Vec<CSSToken> {
    let mut out = Vec::new();
    for value in values {
        match value {
            ComponentValue::Token(t) => out.push(t.clone()),
            ComponentValue::Function(f) => {
                out.push(CSSToken::Function(f.name.clone()));
                out.extend(component_values_to_tokens(&f.args));
                out.push(CSSToken::RBracket);
            }
            ComponentValue::Block(b) => {
                out.push(b.open.clone());
                out.extend(component_values_to_tokens(&b.contents));
                out.push(matching_close(&b.open));
            }
        }
    }
    out
}

fn split_on_semicolons(values: Vec<ComponentValue>) -> Vec<Vec<ComponentValue>> {
    let mut groups = Vec::new();
    let mut current = Vec::new();
    for value in values {
        if maches!(value, ComponentValue::Token(CSSToken::Semicolon)) {
            groups.push(std::mem::take(&mut current));
        } else {
            current.push(value);
        }
    }
    if !current.is_empty() {
        groups.push(current);
    }
    groups
}

fn parse_one_declaration(mut values: Vec<ComponentValue>) -> Option<Declaration> {
    while value.first().is_some_and(ComponentValue::is_space) {
        values.remove(0);
    }
    let ComponentValue::Token(CSSToken::Ident(property)) = values.first()?.clone() else { return None};
    values.remove(0);

    while values.first().is_some_and(ComponentValue::is_space) {
        values.remove(0);
    }
    if !matches!(values.first(), Some(ComponentValue::Token(CSSToken::Colon))) {
        return None;
    }
    value.remove(0);
    
    while values.last().is_some_and(ComponentValue::is_space) {
        values.pop();
    }
    while values.first().is_some_and(ComponentValue::is_space) {
        values.remove(0);
    }

    let important = values.len() >= 2 &&
        matches!(values[values.len() -1], ComponentValue::Token(CSSToken::Ident(ref s)) if s.eq_ignore_ascii_case("important")) &&
        matches!(values[values.len() -2], ComponentValue::Token(CSSToken::Delim('!')));
    if important {
        values.truncate(values.len() -2);
        while values.last().is_some_and(ComponentValue::is_space) {
            values.pop();
        }
    }
    Some(Declaration {property, value: values, important})
}

pub fn parse_declaration_block(source: &str) -> Vec<Declaration> {
    let mut cur = Cursor::from_source(source);
    let mut all = Vec::new();
    while !cur.at_end() {
        all.push(consume_component_value(&mut cur));
    }
    parse_declarations_from_component_values(all)
}

fn parse_declarations_from_component_values(values: Vec<ComponentValue>) -> Vec<Declaration> {
    split_on_semicolons(values).into_iter().filter_map(|group| {
        let trimmed: Vec<ComponentValue> = group.into_iter().filter(|v| !v.is_space()).collect();
        if trimmed.is_empty() {
            None
        } else {
            parse_one_declaration_preserving_spaces(trimmed)
        }
    }).collect()
}

fn parse_one_declaration_preserving_spaces(values: Vec<ComponentValue>) -> Option<Declaration> {
    parse_one_declaration(values)
}

fn consume_rules(cur: &mut Cursor, top_level: bool) -> Vec<Rule> {
    let mut rules = Vec::new();

    loop {
        match cur.peek() {
            Some(CSSToken::Space) => {
                cur.bump();
            }
            None | Some(CSSToken::Eof) => break,
            Some(CSSToken::CdoArrow) | Some(CSSToken::CdcArrow) => {
                if top_level {
                    cur.bump();
                } else if let Some(q) = consume_qualified_rule(cur) {
                    push_style_rule(&mut rules, q);
                }
            }
            Some(CSSToken::AtKeyword(_)) => rules.push(Rule::At(consume_at_rule(cur))),
            _ => {
                if let Some(q) = consume_qualified_rule(cur) {
                    push_style_rule(&mut rules, q);
                } else {
                    break;
                }
            }
        }
    }
    rules
}

fn push_style_rule(rules: &mut Vec<Rule>, q: QualifiedRule) {
    let prelude_tokens = component_values_to_tokens(&q.prelude);
    let selectors = parse_selector_list(&prelude_tokens);
    let declarations = parse_declarations_from_component_values(q.block);
    rules.push(Rule::Style {selectors, declarations});
}

pub fn parse_stylesheet(source: &str) -> Stylesheet {
    let mut cur = Cursor::from_source(source);
    Styesheet {rules: consume_rules(&mut cur, true)}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_a_single_rule_with_declarations(){
        let sheet = parse_stylesheet("body { colour: red; margin: 0 !important; }");
        assert_eq!(sheet.rules.len(), 1);
        let Rule::Style {selectors, declarations} = &sheet.rules[0] else {panic!("expected a style rule")};
        assert_eq!(selectors.len(), 1);
        assert_eq!(declarations.len(), 2);
        assert_eq!(declarations[0].property, "colour");
        assert!(declarations[1].important);
    }
}

