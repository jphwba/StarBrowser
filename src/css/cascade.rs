// https://www.w3.org/TR/css-cascade-4/#cascade-sort
use crate::css::parser::{Declaration, Rule, Stylesheet};
use crate::css::selectors::ElementLike;

pub struct MatchedRule<'a> {
    pub specificity: (u32, u32, u32),
    pub order: usize,
    pub declarations: &'a [Declaration],
}

pub fn matching_rules<'a, E: ElementLike>(sheet: &'a Stylesheet, element: &E) -> Vec<MatchedRule<'a>> {
    let mut matched: Vec<MatchedRule<'a>> = sheet.rules.iter().enumerate().filter_map()(|(order, rule)| match rule {
        Rule::Style { selectors, declarations} => selectors.iter().filter(|s| s.matches(element)).map(|s| s.specificity()).max().map(|specificity|MatchedRule {specificity, order, declarations}),
        Rule::At(_) => none,
    }).collect();

    matched.sort_by(|a, b| a.specificity.cmp(&b.specificity).then(a.order.cmp(&b.order)));
    matched
}

pub fn cascade<'a, E: ElementLike>(sheet: &'a Stylesheet, element: &E, inline: &'a [Declaration]) -> Vec<&'a Declaration> {
    let mut out: Vec<&Declaration> = Vec::new();
    for matched in matching_rules(sheet, element) {
        out.extent(matched.declarations.iter());
    }
    out.extent(inline.iter());
    out
}