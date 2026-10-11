//! Shared advanced-query syntax. Source offsets are UTF-8 bytes, never token indices.
#![deny(missing_docs)]
use chumsky::{input::Input, prelude::*};
use serde::{Deserialize, Serialize};
use std::ops::Range;

/// Maximum raw UTF-8 query bytes and bound literal budget.
pub const MAX_BYTES: usize = 65_536;
/// Maximum lexer tokens, independent of leaf and nesting budgets.
pub const MAX_TOKENS: usize = 16_384;
/// Maximum leaf occurrences before deduplication.
pub const MAX_LEAVES: usize = 1_024;
/// Maximum flat AST nodes at any validation boundary.
pub const MAX_NODES: usize = 4_096;
/// Maximum arena path length, checked iteratively.
pub const MAX_DEPTH: usize = 256;
/// Maximum active source groups and unary prefixes before recursive parsing.
pub const MAX_NESTING: usize = 64;
/// Canonical supported type operands; thesis includes all degree variants.
pub const TYPES: &[&str] = &[
    "article",
    "book",
    "thesis",
    "mastersthesis",
    "phdthesis",
    "booklet",
    "inbook",
    "incollection",
    "inproceedings",
    "misc",
    "techreport",
];
/// Canonical advanced field names exposed to native clients.
pub const FIELDS: &[&str] = &[
    "title",
    "author",
    "editor",
    "journal",
    "booktitle",
    "abstract",
    "note",
    "publisher",
    "school",
    "institution",
    "organization",
    "key",
    "doi",
    "isbn",
    "year",
    "type",
    "tag",
];

/// Strict parser state; an incomplete suffix cannot mask a definite error.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Status {
    /// Complete query eligible for execution.
    Valid,
    /// Only a required suffix, operand, or closing delimiter is missing.
    Incomplete,
    /// A definite lexical, semantic, structural, or budget error prevents execution.
    Invalid,
}
/// Stable diagnostic code and original UTF-8 source location.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Diagnostic {
    /// Stable error or warning key for localization.
    pub code: String,
    /// Half-open original UTF-8 byte range.
    pub span: Range<usize>,
    /// True only when a required suffix or closure is missing at EOF.
    pub incomplete: bool,
}
/// Semantic operation stored in the flat query arena.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum NodeKind {
    /// Ordinary normalized text value.
    Word,
    /// Strict contiguous phrase value.
    Phrase,
    /// All ordered children must match.
    And,
    /// At least one ordered child must match.
    Or,
    /// Logical complement of the single child within external candidates.
    Not,
    /// Apply the named field to implicit descendants.
    Scope,
}
/// An arena node has one parent; scopes preserve explicit field provenance.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Node {
    /// Operation represented by this node.
    pub kind: NodeKind,
    /// Decoded literal or canonical scope name.
    pub value: String,
    /// Stable selected tag ID; None denotes a name-based condition.
    pub tag_id: Option<String>,
    /// Ordered child indices into the containing node arena.
    pub children: Vec<usize>,
    /// Half-open original UTF-8 byte range.
    pub span: Range<usize>,
}
/// Flat query tree preserving order, field provenance, and stable tag bindings.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Document {
    /// Flat node arena, validated for cycles, reachability, single parents, and resource budgets.
    pub nodes: Vec<Node>,
    /// Root node index; None denotes an empty query.
    pub root: Option<usize>,
}
/// Strict parse result; a document exists only when the entire input is valid.
#[derive(Clone, Debug)]
pub struct Analysis {
    /// Strict query validity or stale-snapshot state.
    pub status: Status,
    /// Bounded structured diagnostics; warnings do not prevent execution.
    pub diagnostics: Vec<Diagnostic>,
    /// Typed query tree, available only after successful validation.
    pub document: Option<Document>,
}
#[derive(Clone, Debug, PartialEq)]
enum Token {
    Value(String, bool),
    Field(String),
    Open,
    Close,
    Minus,
    And,
    Or,
    Not,
}
#[derive(Clone, Debug)]
struct Lexeme {
    token: Token,
    span: Range<usize>,
    gap: Range<usize>,
}
#[derive(Clone, Debug)]
enum Raw {
    Word(String),
    Phrase(String, bool),
    Open,
    Close,
}
#[derive(Clone, Debug)]
enum Op {
    Leaf(String, bool, Range<usize>),
    Unary(Token, Range<usize>),
    Join(bool, usize),
    Group(Range<usize>),
}

fn diagnostic(code: &str, span: Range<usize>, incomplete: bool) -> Diagnostic {
    Diagnostic {
        code: code.into(),
        span,
        incomplete,
    }
}
/// Resolve a case-insensitive field name or alias to its canonical spelling.
pub fn canonical_field(value: &str) -> Option<String> {
    let value = value.to_ascii_lowercase();
    Some(match value.as_str() {
        "intitle" => "title".into(),
        "citekey" => "key".into(),
        "before" | "after" => value,
        _ if FIELDS.contains(&value.as_str()) => value,
        _ => return None,
    })
}
fn decode(raw: &str, quoted: bool) -> (String, bool) {
    let mut chars = raw.chars().peekable();
    let mut value = String::new();
    let mut escaped = false;
    while let Some(c) = chars.next() {
        if c == '\\'
            && chars.peek().is_some_and(|&n| {
                if quoted {
                    matches!(n, '\\' | '"')
                } else {
                    matches!(n, '(' | ')' | '"' | '\\' | ':') || (n == '-' && value.is_empty())
                }
            })
        {
            if let Some(n) = chars.next() {
                value.push(n);
                escaped = true;
            }
        } else {
            value.push(c);
        }
    }
    (value, escaped)
}
fn lex(text: &str) -> Result<(Vec<Lexeme>, Vec<Diagnostic>), Vec<Diagnostic>> {
    // This combinator consumes escaped pairs before delimiters. Unknown escapes
    // stay intact and are decoded separately according to the quote context.
    let pair = just::<_, &str, extra::Err<EmptyErr>>('\\')
        .then(any())
        .ignored();
    let phrase = just('"')
        .ignore_then(pair.or(none_of('"').ignored()).repeated().to_slice())
        .then(just('"').or_not())
        .map(|(s, end): (&str, _)| Raw::Phrase(s.into(), end.is_some()));
    let word = just::<_, &str, extra::Err<EmptyErr>>('\\')
        .then(any().filter(|c: &char| !c.is_whitespace()))
        .ignored()
        .or(any()
            .filter(|c: &char| !c.is_whitespace() && !matches!(c, '(' | ')' | '"'))
            .ignored())
        .repeated()
        .at_least(1)
        .to_slice()
        .map(|s: &str| Raw::Word(s.into()));
    let raw = choice((
        phrase,
        just('(').to(Raw::Open),
        just(')').to(Raw::Close),
        word,
    ))
    .map_with(|raw, e| (raw, e.span()))
    .padded()
    .repeated()
    .collect::<Vec<_>>()
    .then_ignore(end());
    let raw: Vec<(Raw, SimpleSpan)> =
        raw.parse(text)
            .into_result()
            .map_err(|errors: Vec<EmptyErr>| {
                errors
                    .iter()
                    .map(|_| diagnostic("LexicalError", 0..text.len(), false))
                    .collect::<Vec<_>>()
            })?;
    let mut out: Vec<Lexeme> = Vec::new();
    let mut errors = Vec::new();
    let mut previous_end = 0;
    let mut scalar = false;
    for (raw, span) in raw {
        let span = span.start..span.end;
        let gap = previous_end..span.start;
        previous_end = span.end;
        if scalar && !gap.is_empty() {
            errors.push(diagnostic("FieldWhitespace", gap.clone(), false));
        }
        match raw {
            Raw::Open => {
                out.push(Lexeme {
                    token: Token::Open,
                    span,
                    gap,
                });
                scalar = false;
            }
            Raw::Close => {
                out.push(Lexeme {
                    token: Token::Close,
                    span,
                    gap,
                });
                scalar = false;
            }
            Raw::Phrase(raw, closed) => {
                let (value, _) = decode(&raw, true);
                if !closed {
                    errors.push(diagnostic("UnclosedQuote", span.clone(), true));
                }
                if closed && value.trim().is_empty() {
                    errors.push(diagnostic("EmptyValue", span.clone(), false));
                }
                out.push(Lexeme {
                    token: Token::Value(value, true),
                    span,
                    gap,
                });
                scalar = false;
            }
            Raw::Word(raw) => {
                let mut raw = raw.as_str();
                let mut start = span.start;
                let mut gap = gap;
                if !scalar {
                    while raw.starts_with('-') {
                        out.push(Lexeme {
                            token: Token::Minus,
                            span: start..start + 1,
                            gap: gap.clone(),
                        });
                        start += 1;
                        raw = &raw[1..];
                        gap = start..start;
                    }
                    if let Some((prefix, rest)) = raw.split_once(':')
                        && !prefix.contains('\\')
                        && let Some(name) = canonical_field(prefix)
                    {
                        let end = start + prefix.len() + 1;
                        out.push(Lexeme {
                            token: Token::Field(name),
                            span: start..end,
                            gap: gap.clone(),
                        });
                        start = end;
                        raw = rest;
                        gap = end..end;
                        scalar = true;
                    }
                }
                if !raw.is_empty() {
                    let (value, escaped) = decode(raw, false);
                    let token = if scalar || escaped {
                        Token::Value(value, false)
                    } else {
                        match value.as_str() {
                            "AND" => Token::And,
                            "OR" => Token::Or,
                            "NOT" => Token::Not,
                            _ => Token::Value(value, false),
                        }
                    };
                    out.push(Lexeme {
                        token,
                        span: start..span.end,
                        gap,
                    });
                    scalar = false;
                }
            }
        }
        if out.len() > MAX_TOKENS {
            return Err(vec![diagnostic(
                "TokenLimitExceeded",
                span_end(text),
                false,
            )]);
        }
    }
    if scalar && previous_end < text.len() {
        errors.push(diagnostic(
            "FieldWhitespace",
            previous_end..text.len(),
            false,
        ));
    }
    Ok((out, errors))
}
fn source_range(span: SimpleSpan) -> Range<usize> {
    span.into_range()
}
fn span_end(text: &str) -> Range<usize> {
    text.len()..text.len()
}

/// Parse with strict execution semantics: any diagnostic prevents returning an executable tree.
pub fn analyze(text: &str) -> Analysis {
    if text.len() > MAX_BYTES {
        return failed(vec![diagnostic("ByteLimitExceeded", span_end(text), false)]);
    }
    let (tokens, mut errors) = match lex(text) {
        Ok(v) => v,
        Err(e) => return failed(e),
    };
    let mut stream = Vec::new();
    let mut expecting = true;
    let mut stack: Vec<usize> = Vec::new();
    let mut active: usize = 0;
    let mut pending = 0;
    let mut leaves = 0;
    for (i, item) in tokens.iter().enumerate() {
        let previous = i.checked_sub(1).and_then(|i| tokens.get(i));
        let starts = matches!(
            item.token,
            Token::Value(..) | Token::Open | Token::Field(_) | Token::Minus | Token::Not
        );
        if starts && !expecting {
            if item.gap.is_empty() {
                errors.push(diagnostic("MissingSeparator", item.span.clone(), false));
            }
            stream.push((Token::And, SimpleSpan::from(item.gap.clone())));
            expecting = true;
        }
        if let Some(prev) = previous {
            if matches!(prev.token, Token::Field(_) | Token::Minus) && !item.gap.is_empty() {
                errors.push(diagnostic("ForbiddenWhitespace", item.gap.clone(), false));
            }
            if prev.token == Token::Not && item.gap.is_empty() {
                errors.push(diagnostic(
                    "NotRequiresWhitespace",
                    item.span.clone(),
                    false,
                ));
            }
        }
        match &item.token {
            Token::Field(_) => {}
            Token::Minus | Token::Not => {
                active += 1;
                pending += 1;
            }
            Token::Open => {
                active += 1;
                stack.push(pending);
                pending = 0;
            }
            Token::Close => {
                if expecting {
                    errors.push(diagnostic("UnexpectedClose", item.span.clone(), false));
                }
                if let Some(prefixes) = stack.pop() {
                    active = active.saturating_sub(prefixes + 1);
                } else {
                    errors.push(diagnostic("UnexpectedClose", item.span.clone(), false));
                }
                expecting = false;
            }
            Token::Value(..) => {
                leaves += 1;
                active = active.saturating_sub(pending);
                pending = 0;
                expecting = false;
            }
            Token::And | Token::Or => {
                if expecting {
                    errors.push(diagnostic("MissingOperand", item.span.clone(), false));
                }
                expecting = true;
            }
        }
        if active > MAX_NESTING {
            return failed(vec![diagnostic(
                "NestingLimitExceeded",
                item.span.clone(),
                false,
            )]);
        }
        if leaves > MAX_LEAVES {
            return failed(vec![diagnostic(
                "LeafLimitExceeded",
                item.span.clone(),
                false,
            )]);
        }
        stream.push((item.token.clone(), SimpleSpan::from(item.span.clone())));
    }
    if !tokens.is_empty() && (expecting || !stack.is_empty()) {
        errors.push(diagnostic("UnexpectedEof", span_end(text), true));
    }
    // Validate completed typed scalars independently of the full grammar. An EOF
    // error elsewhere must not hide an already-invalid year or type.
    let mut scopes: Vec<Option<&str>> = Vec::new();
    let mut next_field = None;
    for item in &tokens {
        match &item.token {
            Token::Field(f) => next_field = Some(f.as_str()),
            Token::Open => {
                scopes.push(
                    next_field
                        .take()
                        .or_else(|| scopes.last().copied().flatten()),
                );
            }
            Token::Close => {
                scopes.pop();
                next_field = None;
            }
            Token::Value(value, quoted) => {
                let scope = next_field
                    .take()
                    .or_else(|| scopes.last().copied().flatten());
                if let Some(f) = scope
                    && !errors
                        .iter()
                        .any(|e| e.code == "UnclosedQuote" && e.span == item.span)
                    && !valid_value(f, value)
                {
                    let incomplete = f == "year"
                        && !quoted
                        && item.span.end == text.len()
                        && matches!(value.as_str(), ">" | ">=" | "<" | "<=" | "=");
                    errors.push(diagnostic(
                        "InvalidFieldValue",
                        item.span.clone(),
                        incomplete,
                    ));
                }
            }
            _ => {}
        }
    }
    if !errors.is_empty() {
        return failed(errors);
    }
    if tokens.is_empty() {
        return Analysis {
            status: Status::Valid,
            diagnostics: vec![],
            document: Some(Document::default()),
        };
    }
    let parser = recursive::<_, _, extra::Err<Rich<Token>>, _, _>(|expr| {
        let value = select! { Token::Value(v, q) = e => vec![Op::Leaf(v, q, source_range(e.span()))] };
        let prefix = select! { Token::Field(f) = e => (Token::Field(f), source_range(e.span())), Token::Not = e => (Token::Not, source_range(e.span())), Token::Minus = e => (Token::Minus, source_range(e.span())) };
        let group = expr.delimited_by(just(Token::Open), just(Token::Close)).map_with(|mut ops: Vec<Op>, e| {
            ops.push(Op::Group(source_range(e.span())));
            ops
        });
        let atom = value.or(group);
        let unary = prefix.repeated().collect::<Vec<_>>().then(atom).map(|(prefixes, mut ops)| {
            for (token, span) in prefixes.into_iter().rev() { ops.push(Op::Unary(token, span)); } ops
        });
        let and = unary.separated_by(just(Token::And)).at_least(1).collect::<Vec<_>>().map(|groups| join_ops(groups, true));
        and.separated_by(just(Token::Or)).at_least(1).collect::<Vec<_>>().map(|groups| join_ops(groups, false))
    }).then_ignore(end());
    let input = stream
        .as_slice()
        .split_token_span(SimpleSpan::from(span_end(text)));
    let parsed: Result<Vec<Op>, Vec<Rich<'_, Token>>> = parser.parse(input).into_result();
    match parsed {
        Ok(ops) => match build(ops) {
            Ok(document) => Analysis {
                status: Status::Valid,
                diagnostics: vec![],
                document: Some(document),
            },
            Err(e) => failed(vec![e]),
        },
        Err(es) => failed(
            es.into_iter()
                .map(|e| diagnostic("SyntaxError", source_range(*e.span()), false))
                .collect(),
        ),
    }
}
fn failed(mut errors: Vec<Diagnostic>) -> Analysis {
    if errors.len() > 32 {
        errors.truncate(31);
        errors.push(diagnostic("TooManyDiagnostics", 0..0, false));
    }
    let status = if errors.iter().any(|e| !e.incomplete) {
        Status::Invalid
    } else {
        Status::Incomplete
    };
    Analysis {
        status,
        diagnostics: errors,
        document: None,
    }
}
fn join_ops(groups: Vec<Vec<Op>>, and: bool) -> Vec<Op> {
    let count = groups.len();
    let mut ops: Vec<_> = groups.into_iter().flatten().collect();
    if count > 1 {
        ops.push(Op::Join(and, count));
    }
    ops
}
fn build(ops: Vec<Op>) -> Result<Document, Diagnostic> {
    let mut document = Document::default();
    let mut stack: Vec<(usize, Range<usize>)> = Vec::new();
    for op in ops {
        let node = match op {
            Op::Group(span) => {
                let (id, extent) = stack
                    .last_mut()
                    .ok_or_else(|| diagnostic("InvalidTree", span.clone(), false))?;
                *extent = span.clone();
                // Keep leaf ranges exact for text edits, while composite nodes
                // expose the whole source expression for atomic condition editing.
                if !matches!(document.nodes[*id].kind, NodeKind::Word | NodeKind::Phrase) {
                    document.nodes[*id].span = span;
                }
                continue;
            }
            Op::Leaf(value, phrase, span) => Node {
                kind: if phrase {
                    NodeKind::Phrase
                } else {
                    NodeKind::Word
                },
                value,
                span,
                children: vec![],
                tag_id: None,
            },
            Op::Unary(token, span) => {
                let (child, extent) = stack
                    .pop()
                    .ok_or_else(|| diagnostic("InvalidTree", span.clone(), false))?;
                let (kind, value) = match token {
                    Token::Field(f) => (NodeKind::Scope, f),
                    _ => (NodeKind::Not, String::new()),
                };
                Node {
                    kind,
                    value,
                    span: span.start..extent.end,
                    children: vec![child],
                    tag_id: None,
                }
            }
            Op::Join(and, count) => {
                let operands = stack.split_off(stack.len().saturating_sub(count));
                let span = operands.first().map_or(0, |(_, range)| range.start)
                    ..operands.last().map_or(0, |(_, range)| range.end);
                let children = operands.into_iter().map(|(id, _)| id).collect();
                Node {
                    kind: if and { NodeKind::And } else { NodeKind::Or },
                    value: String::new(),
                    children,
                    span,
                    tag_id: None,
                }
            }
        };
        if document.nodes.len() == MAX_NODES {
            return Err(diagnostic("NodeLimitExceeded", node.span, false));
        }
        stack.push((document.nodes.len(), node.span.clone()));
        document.nodes.push(node);
    }
    document.root = stack.pop().map(|(id, _)| id);
    validate_document(&document)?;
    Ok(document)
}
/// Validate untrusted flat trees before traversal or binding execution.
pub fn validate_document(document: &Document) -> Result<(), Diagnostic> {
    let fail = |code| diagnostic(code, 0..0, false);
    if document.nodes.len() > MAX_NODES {
        return Err(fail("NodeLimitExceeded"));
    }
    let Some(root) = document.root else {
        return if document.nodes.is_empty() {
            Ok(())
        } else {
            Err(fail("InvalidTree"))
        };
    };
    let mut seen = vec![false; document.nodes.len()];
    let mut stack = vec![(root, 1)];
    let mut leaves = 0;
    let mut bytes = 0;
    while let Some((id, depth)) = stack.pop() {
        if depth > MAX_DEPTH {
            return Err(fail("DepthLimitExceeded"));
        }
        let node = document.nodes.get(id).ok_or_else(|| fail("InvalidTree"))?;
        if seen[id] {
            return Err(fail("InvalidTree"));
        }
        seen[id] = true;
        bytes += node.value.len() + node.tag_id.as_ref().map_or(0, String::len);
        if bytes > MAX_BYTES {
            return Err(fail("ByteLimitExceeded"));
        }
        let valid = match node.kind {
            NodeKind::Word | NodeKind::Phrase => {
                leaves += 1;
                node.children.is_empty() && !node.value.trim().is_empty()
            }
            NodeKind::Scope => node.children.len() == 1 && canonical_field(&node.value).is_some(),
            NodeKind::Not => node.children.len() == 1,
            NodeKind::And | NodeKind::Or => !node.children.is_empty(),
        };
        if !valid {
            return Err(fail("InvalidTree"));
        }
        if leaves > MAX_LEAVES {
            return Err(fail("LeafLimitExceeded"));
        }
        stack.extend(node.children.iter().map(|&id| (id, depth + 1)));
    }
    if seen.iter().any(|v| !v) {
        return Err(fail("InvalidTree"));
    }
    Ok(())
}
/// Parse a year comparison or inclusive range within 1–9999. Returns None for malformed or reversed ranges.
pub fn year_bounds(value: &str) -> Option<(i32, i32)> {
    let year = |s: &str| {
        s.bytes()
            .all(|c| c.is_ascii_digit())
            .then(|| s.parse::<i32>().ok())
            .flatten()
            .filter(|n| (1..=9999).contains(n))
    };
    if let Some((a, b)) = value.split_once("..") {
        if a.is_empty() && b.is_empty() {
            return None;
        }
        let a = if a.is_empty() { 1 } else { year(a)? };
        let b = if b.is_empty() { 9999 } else { year(b)? };
        return (a <= b).then_some((a, b));
    }
    for (prefix, lower, inclusive) in [
        (">=", true, true),
        ("<=", false, true),
        (">", true, false),
        ("<", false, false),
    ] {
        if let Some(v) = value.strip_prefix(prefix) {
            let n = year(v)?;
            return Some(if lower {
                (n + i32::from(!inclusive), 9999)
            } else {
                (1, n - i32::from(!inclusive))
            });
        }
    }
    let n = year(value.strip_prefix('=').unwrap_or(value))?;
    Some((n, n))
}
fn valid_value(field: &str, value: &str) -> bool {
    match field {
        "year" => year_bounds(value).is_some(),
        "before" | "after" => {
            value.bytes().all(|c| c.is_ascii_digit()) && year_bounds(value).is_some()
        }
        "type" => TYPES.contains(&value.to_ascii_lowercase().as_str()),
        _ => !value.trim().is_empty(),
    }
}

#[cfg(test)]
mod tests;

/// Render through an explicit work stack; ordinary terms never become phrases.
pub fn serialize(document: &Document) -> Result<String, Diagnostic> {
    validate_document(document)?;
    let Some(root) = document.root else {
        return Ok(String::new());
    };
    enum Work {
        Node(usize),
        Text(String),
    }
    let mut scalar = vec![false; document.nodes.len()];
    let mut pending = vec![(root, false)];
    while let Some((id, inherited)) = pending.pop() {
        let node = &document.nodes[id];
        let inherited = if node.kind == NodeKind::Scope {
            matches!(
                node.value.as_str(),
                "tag" | "key" | "doi" | "isbn" | "year" | "type" | "before" | "after"
            )
        } else {
            inherited
        };
        scalar[id] = inherited;
        pending.extend(node.children.iter().map(|&child| (child, inherited)));
    }
    let mut work = vec![Work::Node(root)];
    let mut out = String::new();
    while let Some(item) = work.pop() {
        match item {
            Work::Text(text) => out.push_str(&text),
            Work::Node(id) => {
                let node = &document.nodes[id];
                match node.kind {
                    NodeKind::Word
                        if node.value.chars().any(char::is_whitespace)
                            || (scalar[id]
                                && matches!(node.value.as_str(), "AND" | "OR" | "NOT")) =>
                    {
                        if !scalar[id] {
                            return Err(diagnostic(
                                "InvalidWordWhitespace",
                                node.span.clone(),
                                false,
                            ));
                        }
                        out.push('"');
                        for c in node.value.chars() {
                            if matches!(c, '"' | '\\') {
                                out.push('\\');
                            }
                            out.push(c);
                        }
                        out.push('"');
                    }
                    NodeKind::Word => {
                        let value = if matches!(node.value.as_str(), "AND" | "OR" | "NOT") {
                            node.value.to_lowercase()
                        } else {
                            node.value.clone()
                        };
                        for (i, c) in value.chars().enumerate() {
                            if matches!(c, '(' | ')' | '"' | '\\' | ':') || (i == 0 && c == '-') {
                                out.push('\\');
                            }
                            out.push(c);
                        }
                    }
                    NodeKind::Phrase => {
                        out.push('"');
                        for c in node.value.chars() {
                            if matches!(c, '"' | '\\') {
                                out.push('\\');
                            }
                            out.push(c);
                        }
                        out.push('"');
                    }
                    NodeKind::Scope => {
                        out.push_str(&node.value);
                        out.push(':');
                        let child = node.children[0];
                        // A field consumes one scalar or group, never another prefix.
                        // Preserve nested scope overrides and scoped negation explicitly.
                        if matches!(document.nodes[child].kind, NodeKind::Scope | NodeKind::Not) {
                            out.push('(');
                            work.push(Work::Text(")".into()));
                        }
                        work.push(Work::Node(child));
                    }
                    NodeKind::Not => {
                        out.push_str("NOT ");
                        work.push(Work::Node(node.children[0]));
                    }
                    NodeKind::And | NodeKind::Or => {
                        out.push('(');
                        work.push(Work::Text(")".into()));
                        for (i, &child) in node.children.iter().enumerate().rev() {
                            work.push(Work::Node(child));
                            if i > 0 {
                                work.push(Work::Text(
                                    if node.kind == NodeKind::And {
                                        " AND "
                                    } else {
                                        " OR "
                                    }
                                    .into(),
                                ));
                            }
                        }
                    }
                }
            }
        }
        if out.len() > MAX_BYTES {
            return Err(diagnostic("ByteLimitExceeded", 0..0, false));
        }
    }
    Ok(out)
}
