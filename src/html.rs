use crate::client::{HTMLAttribute, HTMLDocument, HTMLNode, HTMLTag};

fn is_void_element(tag: &HTMLTag) -> bool {
    matches!(
        tag,
        HTMLTag::Area
            | HTMLTag::Base
            | HTMLTag::Br
            | HTMLTag::Col
            | HTMLTag::Embed
            | HTMLTag::Hr
            | HTMLTag::Img
            | HTMLTag::Input
            | HTMLTag::Link
            | HTMLTag::Meta
            | HTMLTag::Param
            | HTMLTag::Source
            | HTMLTag::Track
            | HTMLTag::Wbr
    )
}

fn is_raw_text_element(tag: &HTMLTag) -> bool {
    matches!(tag, HTMLTag::Script | HTMLTag::Style)
}

fn decode_entities(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();
    while let Some(c) = chars.next() {
        if c != '&' {
            out.push(c);
            continue;
        }
        let mut entity = String::new();
        let mut consumed = Vec::new();
        while let Some(&n) = chars.peek() {
            if n == ';' || entity.len() > 8 {
                break;
            }
            entity.push(n);
            consumed.push(n);
            chars.next();
        }
        if chars.peek() == Some(&';') {
            chars.next();
            match entity.as_str() {
                "amp" => out.push('&'),
                "lt" => out.push('<'),
                "gt" => out.push('>'),
                "quot" => out.push('"'),
                "apos" | "#39" => out.push('\''),
                "nbsp" => out.push(' '),
                "#39" => out.push('\''),
                other if other.starts_with('#') => {
                    let code = if let Some(hex) = other.strip_prefix("#x").or(other.strip_prefix("#X")) {
                        u32::from_str_radix(hex, 16).ok()
                    } else {
                        other[1..].parse::<u32>().ok()
                    };
                    match code.and_then(char::from_u32) {
                        Some(ch) => out.push(ch),
                        None => {
                            out.push('&');
                            out.push_str(&entity);
                            out.push(';')
                        }
                    }
                }
                _ => {
                    out.push('&');
                    out.push_str(&entity);
                    out.push(';');
                }
            }
        } else {
            out.push('&');
            out.push_str(&entity);
        }
    }
    out
}

struct Scanner <'a> {
    chars: Vec<char>,
    pos: usize,
    _src: std::marker::PhantomData<&'a str>,
}

impl<'a> Scanner<'a> {
    fn new(src: &'a str) -> Self {
        Self { chars: src.chars().collect(), pos: 0, _src: std::marker::PhantomData}
    }

    fn rest_starts_with_ci(&self, needle: &str) -> bool {
        let needle: Vec<char> = needle.chars().collect();
        if self.pos + needle.len() > self.chars.len() {
            return false;
        }
        self.chars[self.pos..self.pos + needle.len()].iter().zip(needle.iter()).all(|(a,b)|a.eq_ignore_ascii_case(b))
    }

    fn peek(&self) -> Option<char> {
        self.chars.get(self.pos).copied()
    }

    fn bump(&mut self) -> Option<char> {
        let c = self.peek()?;
        self.pos += 1;
        Some(c)
    }

    fn eof(&self) -> bool {
        self.pos >= self.chars.len()
    }
}

fn parse_attributes(sc: &mut Scanner) -> Option<HTMLAttribute> {
    while matches!(sc.peek(), Some(c) if c.is_whitespace()) {
        sc.bump();
    }
    if matches!(sc.peek(), None | Some('>') | Some('/')) {
        return None;
    }

    let mut name = String::new();
    while let Some(c) = sc.peek() {
        if c.is_whitespace() || c == '=' || c == '>' || c == '/' {
            break;
        }
        name.push(c);
        sc.bump();
    }
    if name.is_empty() {
        sc.bump();
        return None;
    }

    while matches!(sc.peek(), Some(c) if c.is_whitespace()) {
        sc.bump();
    }

    if sc.peek() != Some('=') {
        return Some(HTMLAttribute { name, value: None });
    }
    sc.bump();
    while matches!(sc.peek(), Some(c) if c.is_whitespace()) {
        sc.bump();
    }

    let value = match sc.peek() {
        Some(q @ ('"' | '\'')) => {
            sc.bump();
            let mut v = String::new();
            while let Some(c) = sc.peek() {
                if c == q {
                    sc.bump();
                    break;
                }
                v.push(c);
                sc.bump();
            }
            v
        }
        _ => {
            let mut v = String::new();
            while let Some(c) = sc.peek() {
                if c.is_whitespace() || c == '>' {
                    break;
                }
                v.push(c);
                sc.bump();
            }
            v
        }
    };
    Some(HTMLAttribute { name, value: Some(decode_entities(&value)) })
}

pub fn parse(source: &str) -> HTMLDocument {
    let mut sc = Scanner::new(source);
    let mut doctype = None;
    let mut root = HTMLNode::new(HTMLTag::Html);
    let mut stack: Vec<HTMLNode> = vec![];
    let mut open_tags: Vec<HTMLTag> = vec![];

    macro_rules! current_children {
        () => {
            match stack.last_mut() {
                Some(node) => &mut node.children,
                None => &mut root.children,
            }
        };
    }

    while !sc.eof() {
        if sc.peek() != Some('<') {
            let mut text = String::new();
            while let Some(c) = sc.peek() {
                if c == '<' {
                    break;
                }
                text.push(c);
                sc.bump();
            }
            let trimmed = decode_entities(&text);
            if !trimmed.trim().is_empty() {
                let mut node = HTMLNode::new(HTMLTag::Custom("#text".to_string()));
                node.text_content = Some(trimmed);
                current_children!().push(node);
            }
            continue;
        }

        if sc.rest_starts_with_ci("<!-->") {
            sc.pos += 4;
            while !sc.eof() && !sc.rest_starts_with_ci("-->") {
                sc.bump();
            }
            sc.pos = (sc.pos + 3).min(sc.chars.len());
            continue;
        }

        if sc.rest_starts_with_ci("<!") {
            let mut decl = String::new();
            sc.bump();
            sc.bump();
            while let Some(c) = sc.peek() {
                if c == '>' {
                    sc.bump();
                    break;
                }
                decl.push(c);
                sc.bump();
            }
            if doctype.is_none() {
                doctype = Some(decl.trim().to_string());
            }
            continue;
        }

        if sc.chars.get(sc.pos + 1) == Some(&'/') {
            sc.pos += 2;
            let mut name = String::new();
            while let Some(c) = sc.peek() {
                if c == '>' {
                    sc.bump();
                    break;
                }
                name.push(c);
                sc.bump();
            }
            let closing_tag = HTMLTag::from_string(name.trim());
            if let Some(pos) = open_tags.iter().rposition(|t| *t == closing_tag) {
                while stack.len() > pos {
                    let finished = stack.pop().unwrap();
                    open_tags.pop();
                    match stack.last_mut() {
                        Some(parent) => parent.children.push(finished),
                        None => root.children.push(finished),
                    }
                }
            }
            continue;
        }

        sc.bump();
        let mut name = String::new();
        while let Some(c) = sc.peek() {
            if c.is_whitespace() || c == '>' || c == '/' {
                break;
            }
            name.push(c);
            sc.bump();
        }
        if name.is_empty() {
            let mut node = HTMLNode::new(HTMLTag::Custom("#text".to_string()));
            node.text_content = Some("<".to_string());
            current_children!().push(node);
            continue;
        }

        let tag = HTMLTag::from_string(&name);
        let mut node = HTMLNode::new(tag.clone());
        while let Some(attr) = parse_attributes(&mut sc) {
            node.attributes.push(attr);
        }

        let mut self_closing = false;
        while matches!(sc.peek(), Some(c) if c.is_whitespace()) {
            sc.bump();
        }
        if sc.peek() == Some('/') {
            self_closing = true;
            sc.bump();
        }
        if sc.peek() == Some('>') {
            sc.bump();
        }

        if is_raw_text_element(&tag) {
            let close_marker = format!("</{}", name.to_ascii_lowercase());
            let mut raw = String::new();
            loop {
                if sc.eof() || sc.rest_starts_with_ci(&close_marker) {
                    break;
                }
                raw.push(sc.bump().unwrap());
            }
            while !sc.eof() && sc.peek() != Some('>') {
                sc.bump();
            }
            sc.bump();
            node.text_content = Some(raw);
            current_children!().push(node);
            continue;
        }

        if self_closing || is_void_element(&tag) {
            current_children!().push(node);
        } else {
            stack.push(node);
            open_tags.push(tag);
        }
    }

    while let Some(finished) = stack.pop() {
        match stack.last_mut() {
            Some(parent) => parent.children.push(finished),
            None => root.children.push(finished),
        }
    }

    HTMLDocument { doctype, root }
}

fn is_block_level(tag: &HTMLTag) -> bool {
    matches!(
        tag,
        HTMLTag::Address
            | HTMLTag::Article
            | HTMLTag::Aside
            | HTMLTag::Blockquote
            | HTMLTag::Br
            | HTMLTag::Details
            | HTMLTag::Dialog
            | HTMLTag::Dd
            | HTMLTag::Div
            | HTMLTag::Dl
            | HTMLTag::Dt
            | HTMLTag::Fieldset
            | HTMLTag::Figcaption
            | HTMLTag::Figure
            | HTMLTag::Footer
            | HTMLTag::Form
            | HTMLTag::H1
            | HTMLTag::H2
            | HTMLTag::H3
            | HTMLTag::H4
            | HTMLTag::H5
            | HTMLTag::H6
            | HTMLTag::Header
            | HTMLTag::Hgroup
            | HTMLTag::Hr
            | HTMLTag::Li
            | HTMLTag::Main
            | HTMLTag::Nav
            | HTMLTag::Ol
            | HTMLTag::P
            | HTMLTag::Pre
            | HTMLTag::Section
            | HTMLTag::Table
            | HTMLTag::Tr
            | HTMLTag::Ul
    )
}

pub fn extract_visible_text(doc: &HTMLDocument) -> String {
    let mut out = String::new();
    walk(&doc.root, &mut out);

    let mut lines: Vec<&str> = Vec::new();
    for line in out.lines() {
        let trimmed = line.trim_end();
        if trimmed.is_empty() && lines.last().is_some_and(|l| l.is_empty()) {
            continue;
        }
        lines.push(trimmed);
    }
    lines.join("\n").trim().to_string()
}

fn walk(node: &HTMLNode, out: &mut String) {
    if matches!(node.tag, HTMLTag::Head | HTMLTag::Script | HTMLTag::Style | HTMLTag::Noscript) {
        return;
    }

    if let HTMLTag::Custom(name) = &node.tag {
        if name == "#text" {
            if let Some(text) = &node.text_content {
                let collapsed: String = text.split_whitespace().collect::<Vec<_>>().join(" ");
                if !collapsed.is_empty() {
                    if !out.is_empty() && !out.ends_with(['\n', ' ']) {
                        out.push(' ');
                    }
                    out.push_str(&collapsed);
                }
            }
            return;
        }
    }

    let block = is_block_level(&node.tag);
    if block && !out.is_empty() {
        out.push('\n');
    }

    for child in &node.children {
        walk(child, out);
    }

    if block {
        out.push('\n');
    }
}

pub fn find_title(doc: &HTMLDocument) -> Option<String> {
    fn search(node: &HTMLNode) -> Option<String> {
        if node.tag == HTMLTag::Title {
            return node.children.iter().find_map(|c| c.text_content.clone()).or_else(|| node.text_content.clone());
        }
        node.children.iter().find_map(search)
    }
    search(&doc.root)
}