//! Boolean evaluation and ranking over immutable bibliography snapshots.
#![deny(missing_docs)]
use super::query::{Diagnostic, Document, NodeKind, validate_document, year_bounds};
use super::{FieldKind, Rank, SearchText, chunk_text, collapse_spaces, normalize, word_character};
use crate::core::bib::Reference;
use std::collections::{HashMap, HashSet};
use std::ops::Range;
use unicode_casefold::UnicodeCaseFold;
use unicode_normalization::UnicodeNormalization;
use unicode_script::{Script, UnicodeScript};
use unicode_segmentation::UnicodeSegmentation;

struct Field {
    name: &'static str,
    raw: String,
    text: SearchText,
    strict: String,
    identifier: String,
    math: Vec<Range<usize>>,
}
struct Record {
    fields: Vec<Field>,
    year: Option<i32>,
    kind: String,
    key: String,
}
/// Preprocessed extended bibliography fields held with an immutable source snapshot.
pub struct AdvancedIndex {
    records: Vec<Record>,
}
/// A matched source index and positive field evidence from one execution batch.
#[derive(Clone, Debug)]
pub struct Hit {
    /// Source record index in the immutable bibliography snapshot.
    pub index: usize,
    /// Whether this row belongs to the spelling-supplement batch.
    pub fuzzy: bool,
    /// Positive query terms that required spelling supplementation, in contribution order.
    pub fuzzy_terms: Vec<String>,
    /// Canonical fields supporting the successful positive branch.
    pub fields: Vec<String>,
    /// Original field byte spans for successful positive conditions.
    pub ranges: Vec<MatchRange>,
}
/// A half-open UTF-8 range in an original field entry, suitable for chunk-aware rendering.
#[derive(Clone, Debug)]
pub struct MatchRange {
    /// Canonical field name.
    pub field_name: String,
    /// Zero-based entry within repeated fields such as authors or journals.
    pub entry_index: usize,
    /// Inclusive original field byte offset.
    pub start: usize,
    /// Exclusive original field byte offset.
    pub end: usize,
}
struct Predicate {
    field: String,
    value: String,
    phrase: bool,
    tag_id: Option<String>,
    typed: bool,
    strict_value: String,
    identifier: String,
    bounds: Option<(i32, i32)>,
    text: SearchText,
    fuzzy: bool,
}
#[derive(Clone)]
struct PlanNode {
    children: Vec<usize>,
    and: bool,
    predicate: Option<usize>,
    negative: bool,
}
struct Plan {
    nodes: Vec<PlanNode>,
    predicates: Vec<Predicate>,
    order: Vec<usize>,
    root: usize,
}
struct Evaluation {
    contributions: Vec<usize>,
    rank: Rank,
}

/// Normalize strict phrases with NFC, Unicode lowercase, and collapsed whitespace while preserving punctuation and accents.
pub fn strict(value: &str) -> String {
    collapse_spaces(value.nfc().flat_map(char::to_lowercase))
}
/// Canonical tag equality shared with metadata: Unicode whitespace, NFC and default case folding.
pub fn normalize_tag_name(value: &str) -> String {
    value
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .nfc()
        .case_fold()
        .collect()
}
fn phrase_boundary_character(c: char) -> bool {
    c.is_numeric() || (c.is_alphabetic() && c.script() == Script::Latin)
}
fn identifier(field: &str, value: &str) -> String {
    let value: String = (if field == "key" { value } else { value.trim() })
        .nfc()
        .flat_map(char::to_lowercase)
        .collect();
    match field {
        "doi" => [
            "https://doi.org/",
            "http://doi.org/",
            "https://dx.doi.org/",
            "http://dx.doi.org/",
            "doi:",
        ]
        .iter()
        .find_map(|p| value.strip_prefix(p))
        .unwrap_or(&value)
        .trim()
        .to_owned(),
        "isbn" => value
            .chars()
            .filter(|c| !c.is_whitespace() && *c != '-')
            .collect(),
        _ => value,
    }
}
fn ordinary(name: &str) -> bool {
    matches!(
        name,
        "key" | "title" | "author" | "journal" | "year" | "note"
    )
}
fn weight(name: &str) -> u64 {
    match name {
        "key" | "doi" | "isbn" => 6,
        "title" => 5,
        "author" => 4,
        "journal" | "booktitle" => 3,
        "year" | "editor" => 2,
        _ => 1,
    }
}
fn selected(field: &Field, predicate: &Predicate) -> bool {
    if predicate.field == "all" {
        ordinary(field.name)
    } else {
        field.name == predicate.field
    }
}
fn phrase_matches(text: &str, value: &str) -> bool {
    text.match_indices(value).any(|(start, _)| {
        let end = start + value.len();
        (!value.chars().next().is_some_and(phrase_boundary_character)
            || text[..start]
                .chars()
                .next_back()
                .is_none_or(|c| !word_character(c)))
            && (!value
                .chars()
                .next_back()
                .is_some_and(phrase_boundary_character)
                || text[end..]
                    .chars()
                    .next()
                    .is_none_or(|c| !word_character(c)))
    })
}
impl AdvancedIndex {
    /// Preprocess all extended fields without modifying source text, chunks, or record identity.
    pub fn new(references: &[Reference]) -> Self {
        Self {
            records: references
                .iter()
                .map(|r| {
                    let mut fields = Vec::new();
                    let mut add = |name, raw: String, chunks: Option<&[biblatex::Chunk]>| {
                        if !raw.is_empty() {
                            let mut offset = 0;
                            let math = chunks
                                .into_iter()
                                .flatten()
                                .filter_map(|chunk| {
                                    let start = offset;
                                    offset += chunk.get().len();
                                    matches!(chunk, biblatex::Chunk::Math(_))
                                        .then_some(start..offset)
                                })
                                .collect();
                            fields.push(Field {
                                name,
                                math,
                                strict: strict(&raw),
                                identifier: if matches!(name, "doi" | "isbn" | "key") {
                                    identifier(name, &raw)
                                } else {
                                    String::new()
                                },
                                text: chunks.map_or_else(
                                    || SearchText::new(&raw),
                                    |c| SearchText::from_chunks(Some(c)),
                                ),
                                raw,
                            });
                        }
                    };
                    add("key", r.cite_key.clone(), None);
                    for (name, chunks) in [
                        ("title", &r.title),
                        ("note", &r.note),
                        ("abstract", &r.abstract_),
                        ("booktitle", &r.book_title),
                    ] {
                        add(name, chunk_text(chunks.as_deref()), chunks.as_deref());
                    }
                    for (name, values) in [
                        ("author", &r.author),
                        ("publisher", &r.publisher),
                        ("organization", &r.organization),
                    ] {
                        for value in values.iter().flatten() {
                            add(name, value.clone(), None);
                        }
                    }
                    for (name, value) in [
                        ("journal", &r.journal),
                        ("journal", &r.full_journal),
                        ("school", &r.school),
                        ("institution", &r.institution),
                        ("doi", &r.doi),
                        ("isbn", &r.isbn),
                    ] {
                        if let Some(v) = value {
                            add(name, v.clone(), None);
                        }
                    }
                    for (name, _) in r.editor.iter().flatten() {
                        add("editor", name.clone(), None);
                    }
                    if let Some(year) = r.year {
                        add("year", year.to_string(), None);
                    }
                    Record {
                        fields,
                        year: r.year,
                        kind: format!("{:?}", r.type_).to_ascii_lowercase(),
                        key: normalize(&r.cite_key),
                    }
                })
                .collect(),
        }
    }
    /// Estimate owned index memory for the shared cache budget.
    pub fn allocated_bytes(&self) -> usize {
        self.records.capacity() * std::mem::size_of::<Record>()
            + self
                .records
                .iter()
                .map(|r| {
                    r.fields.capacity() * std::mem::size_of::<Field>()
                        + r.key.capacity()
                        + r.kind.capacity()
                        + r.fields
                            .iter()
                            .map(|f| {
                                f.raw.capacity()
                                    + f.strict.capacity()
                                    + f.identifier.capacity()
                                    + f.text.allocated_bytes()
                                    + f.math.capacity() * std::mem::size_of::<Range<usize>>()
                            })
                            .sum::<usize>()
                })
                .sum::<usize>()
    }
    /// Tag lookup is supplied by the service's immutable metadata snapshot.
    pub fn search(
        &self,
        document: &Document,
        default_field: &str,
        include: impl Fn(usize) -> bool,
        tags: impl Fn(usize, &str, Option<&str>) -> bool,
    ) -> Result<Vec<Hit>, Diagnostic> {
        validate_document(document)?;
        if document.root.is_none() {
            return Ok((0..self.records.len())
                .filter(|&i| include(i))
                .map(|index| Hit {
                    index,
                    fuzzy: false,
                    fuzzy_terms: vec![],
                    fields: vec![],
                    ranges: vec![],
                })
                .collect());
        }
        let plan = compile(document, default_field);
        let mut need_rank = vec![false; plan.nodes.len()];
        need_rank[plan.root] = true;
        for node in &plan.nodes {
            if node.predicate.is_none() && !node.and {
                for &child in &node.children {
                    need_rank[child] = true;
                }
            }
        }
        let mut found = Vec::new();
        let mut seen = HashSet::new();
        let mut score_queries: HashMap<Vec<usize>, SearchText> = HashMap::new();
        for fuzzy in [false, true] {
            if fuzzy && (found.len() >= 20 || !plan.predicates.iter().any(|p| p.fuzzy)) {
                break;
            }
            for (index, record) in self.records.iter().enumerate() {
                if !include(index) || seen.contains(&index) {
                    continue;
                }
                let matches: Vec<_> = plan
                    .predicates
                    .iter()
                    .map(|p| record.matches(p, fuzzy, |name, id| tags(index, name, id)))
                    .collect();
                let mut results: Vec<Option<Evaluation>> =
                    (0..plan.nodes.len()).map(|_| None).collect();
                for &id in &plan.order {
                    let node = &plan.nodes[id];
                    let contribution = if let Some(p) = node.predicate {
                        (matches[p].0 != node.negative).then(|| {
                            if node.negative
                                || (plan.predicates[p].typed
                                    && matches!(
                                        plan.predicates[p].field.as_str(),
                                        "tag" | "year" | "before" | "after" | "type"
                                    ))
                            {
                                vec![]
                            } else {
                                vec![p]
                            }
                        })
                    } else if node.and {
                        if node.children.iter().all(|&c| results[c].is_some()) {
                            let mut ids = HashSet::new();
                            Some(
                                node.children
                                    .iter()
                                    .flat_map(|&c| {
                                        results[c]
                                            .as_ref()
                                            .into_iter()
                                            .flat_map(|e| e.contributions.iter().copied())
                                    })
                                    .filter(|p| ids.insert(*p))
                                    .collect(),
                            )
                        } else {
                            None
                        }
                    } else {
                        let mut best: Option<&Evaluation> = None;
                        for &c in &node.children {
                            if let Some(e) = &results[c]
                                && best.is_none_or(|b| {
                                    (!e.contributions.is_empty(), &e.rank)
                                        > (!b.contributions.is_empty(), &b.rank)
                                })
                            {
                                best = Some(e);
                            }
                        }
                        best.map(|e| e.contributions.clone())
                    };
                    results[id] = contribution.map(|contributions| {
                        let rank = if need_rank[id] {
                            record.score(
                                &contributions,
                                &plan.predicates,
                                fuzzy,
                                &mut score_queries,
                            )
                        } else {
                            Rank::default()
                        };
                        Evaluation {
                            contributions,
                            rank,
                        }
                    });
                }
                if let Some(result) = results[plan.root].take() {
                    let fields = result
                        .contributions
                        .iter()
                        .flat_map(|&p| matches[p].1.clone())
                        .collect::<HashSet<_>>();
                    let mut fields: Vec<_> = fields.into_iter().collect();
                    fields.sort();
                    found.push((
                        Hit {
                            index,
                            fuzzy,
                            fuzzy_terms: if fuzzy {
                                result
                                    .contributions
                                    .iter()
                                    .filter_map(|&p| {
                                        let predicate = &plan.predicates[p];
                                        (predicate.fuzzy
                                            && !record.matches(predicate, false, |_, _| false).0)
                                            .then(|| predicate.value.clone())
                                    })
                                    .collect()
                            } else {
                                vec![]
                            },
                            fields,
                            ranges: record.ranges(&result.contributions, &plan.predicates, fuzzy),
                        },
                        result,
                    ));
                    seen.insert(index);
                }
            }
        }
        found.sort_by(|(a, ar), (b, br)| {
            a.fuzzy
                .cmp(&b.fuzzy)
                .then_with(|| (!br.contributions.is_empty()).cmp(&(!ar.contributions.is_empty())))
                .then_with(|| br.rank.cmp(&ar.rank))
                .then_with(|| {
                    if ar.contributions.is_empty() && br.contributions.is_empty() {
                        a.index.cmp(&b.index)
                    } else {
                        self.records[a.index]
                            .key
                            .cmp(&self.records[b.index].key)
                            .then(a.index.cmp(&b.index))
                    }
                })
        });
        Ok(found.into_iter().map(|(hit, _)| hit).collect())
    }
}
impl Record {
    fn ranges(&self, ids: &[usize], predicates: &[Predicate], fuzzy: bool) -> Vec<MatchRange> {
        let mut output = Vec::new();
        let mut entries = HashMap::new();
        for f in &self.fields {
            let index = entries.entry(f.name).or_insert(0);
            let entry_index = *index;
            *index += 1;
            for &id in ids {
                let p = &predicates[id];
                if !selected(f, p) || self.quality(f, p, fuzzy).is_none() {
                    continue;
                }
                let ranges = if matches!(p.field.as_str(), "key" | "doi" | "isbn") {
                    std::iter::once(0..f.raw.len()).collect()
                } else {
                    source_ranges(f, p, fuzzy)
                };
                for range in ranges {
                    output.push(MatchRange {
                        field_name: f.name.into(),
                        entry_index,
                        start: range.start,
                        end: range.end,
                    });
                }
            }
        }
        output.sort_by(|a, b| {
            (&a.field_name, a.entry_index, a.start, a.end).cmp(&(
                &b.field_name,
                b.entry_index,
                b.start,
                b.end,
            ))
        });
        output.dedup_by(|a, b| {
            a.field_name == b.field_name
                && a.entry_index == b.entry_index
                && a.start == b.start
                && a.end == b.end
        });
        output
    }
    fn matches(
        &self,
        p: &Predicate,
        fuzzy: bool,
        tag: impl Fn(&str, Option<&str>) -> bool,
    ) -> (bool, Vec<String>) {
        let condition = match p.field.as_str() {
            "tag" => Some(tag(&p.value, p.tag_id.as_deref())),
            "type" => Some(
                self.kind == p.value
                    || (p.value == "thesis"
                        && matches!(self.kind.as_str(), "mastersthesis" | "phdthesis")),
            ),
            "year" | "before" | "after" if p.typed => Some(
                p.bounds
                    .is_some_and(|(a, b)| self.year.is_some_and(|y| y >= a && y <= b)),
            ),
            _ => None,
        };
        if let Some(m) = condition {
            return (m, vec![]);
        }
        let fields: Vec<_> = self
            .fields
            .iter()
            .filter(|f| selected(f, p))
            .filter(|f| self.quality(f, p, fuzzy).is_some())
            .map(|f| f.name.to_owned())
            .collect();
        (!fields.is_empty(), fields)
    }
    fn quality(&self, f: &Field, p: &Predicate, fuzzy: bool) -> Option<u8> {
        if matches!(p.field.as_str(), "key" | "doi" | "isbn") {
            return (f.identifier == p.identifier).then_some(10);
        }
        if p.phrase {
            return phrase_matches(&f.strict, &p.strict_value)
                .then_some(if f.strict == p.strict_value { 10 } else { 8 });
        }
        f.text
            .exact(
                &p.text,
                if f.name == "year" {
                    FieldKind::Year
                } else {
                    FieldKind::Title
                },
            )
            .map(|m| m.quality)
            .or_else(|| {
                (fuzzy
                    && p.fuzzy
                    && matches!(f.name, "title" | "author" | "journal")
                    && f.text.fuzzy_match(&p.text))
                .then_some(1)
            })
    }
    fn score(
        &self,
        ids: &[usize],
        predicates: &[Predicate],
        fuzzy: bool,
        cache: &mut HashMap<Vec<usize>, SearchText>,
    ) -> Rank {
        if cache.len() >= 256 && !cache.contains_key(ids) {
            cache.clear();
        }
        let whole = cache.entry(ids.to_vec()).or_insert_with(|| {
            SearchText::new(
                &ids.iter()
                    .map(|&id| predicates[id].value.as_str())
                    .collect::<Vec<_>>()
                    .join(" "),
            )
        });
        self.rank(ids, predicates, fuzzy, whole)
    }
    fn rank(
        &self,
        ids: &[usize],
        predicates: &[Predicate],
        fuzzy: bool,
        whole: &SearchText,
    ) -> Rank {
        if ids.is_empty() {
            return Rank::default();
        }
        let first = &predicates[ids[0]];
        if matches!(
            first.field.as_str(),
            "all" | "title" | "author" | "journal" | "year" | "note"
        ) && ids.iter().all(|&id| {
            let p = &predicates[id];
            !p.phrase && !p.typed && p.field == first.field
        }) {
            let fields = self
                .fields
                .iter()
                .filter(|f| selected(f, first))
                .filter_map(|f| {
                    let kind = match f.name {
                        "key" => FieldKind::Key,
                        "title" => FieldKind::Title,
                        "author" => FieldKind::Author,
                        "journal" => FieldKind::Journal,
                        "year" => FieldKind::Year,
                        "note" => FieldKind::Note,
                        _ => return None,
                    };
                    Some((kind, &f.text))
                });
            return super::rank_fields(
                fields,
                whole,
                ids.iter()
                    .map(|&id| (&predicates[id].text, predicates[id].fuzzy)),
                ids.len(),
                fuzzy,
            )
            .unwrap_or_default();
        }
        let mut rank = Rank {
            weakest: 10,
            ..Rank::default()
        };
        for &id in ids {
            let p = &predicates[id];
            let best = self
                .fields
                .iter()
                .filter(|f| selected(f, p))
                .filter_map(|f| self.quality(f, p, fuzzy).map(|q| (q, weight(f.name))))
                .max();
            if let Some((quality, weight)) = best {
                rank.weakest = rank.weakest.min(quality);
                rank.field_score += u64::from(quality) * weight;
            }
        }
        let first = &predicates[ids[0]];
        let contiguous = ids.iter().all(|&id| {
            !predicates[id].phrase
                && predicates[id].field == first.field
                && !matches!(predicates[id].field.as_str(), "doi" | "isbn" | "key")
        });
        for f in &self.fields {
            if !ids.iter().all(|&id| selected(f, &predicates[id])) {
                continue;
            }
            if contiguous {
                if f.name == "key" {
                    rank.exact_key = if f.text.normal == whole.normal {
                        2
                    } else if f.text.folded() == whole.folded() {
                        1
                    } else {
                        0
                    };
                }
                if let Some(m) = f.text.exact(
                    whole,
                    if f.name == "year" {
                        FieldKind::Year
                    } else {
                        FieldKind::Title
                    },
                ) {
                    rank.phrase = rank.phrase.max(m.quality);
                }
            }
            let mut start = usize::MAX;
            let mut end = 0;
            let mut same = true;
            for &id in ids {
                let p = &predicates[id];
                if p.phrase
                    && let Some(q) = self.quality(f, p, false)
                {
                    rank.phrase = rank.phrase.max(q);
                }
                if let Some(m) = f.text.exact(&p.text, FieldKind::Title) {
                    start = start.min(m.start);
                    end = end.max(m.end);
                } else {
                    same = false;
                }
                if p.field == "key" && self.quality(f, p, false).is_some() {
                    rank.exact_key = 2;
                }
            }
            if same && ids.len() > 1 && f.name != "year" {
                rank.same_field = true;
                rank.proximity = rank.proximity.max(usize::MAX - end.saturating_sub(start));
            }
        }
        rank
    }
}
fn source_ranges(field: &Field, p: &Predicate, fuzzy: bool) -> Vec<Range<usize>> {
    for folded in [false, true] {
        if p.phrase && folded {
            break;
        }
        let mut normalized = String::new();
        let mut origins: Vec<Range<usize>> = Vec::new();
        let mut pending = None;
        for (start, g) in field.raw.grapheme_indices(true) {
            let range = start..start + g.len();
            let value = if p.phrase {
                g.nfc().flat_map(char::to_lowercase).collect::<String>()
            } else {
                super::normalized_chars(g).collect()
            };
            let value = if folded
                && !field
                    .math
                    .iter()
                    .any(|m| m.start < range.end && m.end > range.start)
            {
                super::folded_chars(&value).collect::<String>()
            } else {
                value
            };
            for c in value.chars() {
                if c.is_whitespace() {
                    if !normalized.is_empty() {
                        pending = Some(range.clone());
                    }
                    continue;
                }
                if let Some(space) = pending.take() {
                    normalized.push(' ');
                    origins.push(space);
                }
                normalized.push(c);
                origins.extend(std::iter::repeat_n(range.clone(), c.len_utf8()));
            }
        }
        let needle = if p.phrase {
            p.strict_value.clone()
        } else if folded {
            p.text.folded().into()
        } else {
            p.text.normal.clone()
        };
        if needle.is_empty() {
            return vec![];
        }
        let mut found = Vec::new();
        for (start, _) in normalized.match_indices(&needle) {
            if p.phrase {
                let end = start + needle.len();
                if (needle.chars().next().is_some_and(phrase_boundary_character)
                    && normalized[..start]
                        .chars()
                        .next_back()
                        .is_some_and(word_character))
                    || (needle
                        .chars()
                        .next_back()
                        .is_some_and(phrase_boundary_character)
                        && normalized[end..].chars().next().is_some_and(word_character))
                {
                    continue;
                }
            }
            let end = start + needle.len();
            if let (Some(a), Some(b)) = (origins.get(start), origins.get(end - 1)) {
                found.push(a.start..b.end);
            }
        }
        if !found.is_empty() {
            return found;
        }
        if fuzzy && p.fuzzy && folded && matches!(field.name, "title" | "author" | "journal") {
            let mut offset = 0;
            for part in normalized.split_inclusive(|c: char| !word_character(c)) {
                let start = offset;
                offset += part.len();
                let word = part.trim_end_matches(|c: char| !word_character(c));
                if !word.is_empty()
                    && super::eligible_fuzzy(&needle)
                    && super::one_edit_apart(word.as_bytes(), needle.as_bytes())
                    && let (Some(a), Some(b)) =
                        (origins.get(start), origins.get(start + word.len() - 1))
                    && !field
                        .math
                        .iter()
                        .any(|m| m.start < b.end && m.end > a.start)
                {
                    found.push(a.start..b.end);
                }
            }
        }
        if !found.is_empty() {
            return found;
        }
    }
    vec![]
}
fn compile(document: &Document, default_field: &str) -> Plan {
    let root = document.root.unwrap_or(0);
    let mut plan = Plan {
        nodes: (0..document.nodes.len())
            .map(|_| PlanNode {
                children: vec![],
                and: true,
                predicate: None,
                negative: false,
            })
            .collect(),
        predicates: vec![],
        order: vec![],
        root,
    };
    let mut intern = HashMap::new();
    let mut work = vec![(root, default_field.to_owned(), false, false, false)];
    while let Some((id, scope, explicit, negative, visited)) = work.pop() {
        if visited {
            plan.order.push(id);
            continue;
        }
        let node = &document.nodes[id];
        work.push((id, scope.clone(), explicit, negative, true));
        let target = &mut plan.nodes[id];
        target.children = node.children.clone();
        match node.kind {
            NodeKind::Word | NodeKind::Phrase => {
                let phrase = node.kind == NodeKind::Phrase;
                let text = SearchText::new(&node.value);
                let typed = explicit
                    && matches!(
                        scope.as_str(),
                        "year" | "before" | "after" | "type" | "tag" | "key" | "doi" | "isbn"
                    );
                let key = (
                    scope.clone(),
                    typed,
                    phrase && !typed,
                    if typed {
                        // Exact identifiers and tag names must never share ordinary
                        // punctuation/compatibility folding: doing so changes Boolean truth.
                        match scope.as_str() {
                            "key" | "doi" | "isbn" => identifier(&scope, &node.value),
                            "type" => node.value.to_ascii_lowercase(),
                            "tag" => normalize_tag_name(&node.value),
                            "year" | "before" | "after" => {
                                let value = match scope.as_str() {
                                    "before" => format!("<{}", node.value),
                                    "after" => format!(">{}", node.value),
                                    _ => node.value.clone(),
                                };
                                format!("{:?}", year_bounds(&value))
                            }
                            _ => node.value.clone(),
                        }
                    } else if phrase {
                        strict(&node.value)
                    } else {
                        text.normal.clone()
                    },
                    if typed || phrase {
                        String::new()
                    } else {
                        text.folded().to_owned()
                    },
                    node.tag_id.clone(),
                );
                let next = plan.predicates.len();
                let pid = *intern.entry(key).or_insert(next);
                let fuzzy = !explicit && !negative && !phrase;
                if pid == next {
                    plan.predicates.push(Predicate {
                        value: if scope == "type" {
                            node.value.to_ascii_lowercase()
                        } else {
                            node.value.clone()
                        },
                        field: scope.clone(),
                        phrase,
                        tag_id: node.tag_id.clone(),
                        strict_value: strict(&node.value),
                        identifier: identifier(&scope, &node.value),
                        bounds: if scope == "before" {
                            year_bounds(&format!("<{}", node.value))
                        } else if scope == "after" {
                            year_bounds(&format!(">{}", node.value))
                        } else {
                            year_bounds(&node.value)
                        },
                        typed,
                        text,
                        fuzzy,
                    });
                } else {
                    plan.predicates[pid].fuzzy &= fuzzy;
                }
                target.predicate = Some(pid);
                target.negative = negative;
            }
            NodeKind::Scope => work.push((
                node.children[0],
                super::query::canonical_field(&node.value).unwrap_or_else(|| node.value.clone()),
                true,
                negative,
                false,
            )),
            NodeKind::Not => work.push((node.children[0], scope, explicit, !negative, false)),
            NodeKind::And | NodeKind::Or => {
                target.and = (node.kind == NodeKind::And) != negative;
                for &child in node.children.iter().rev() {
                    work.push((child, scope.clone(), explicit, negative, false));
                }
            }
        }
    }
    // Execution interning occurs after query-wide fuzzy policy collection. The
    // editable document remains untouched, including explicit field provenance.
    let mut compact: Vec<PlanNode> = Vec::new();
    let mut mapping = vec![0; plan.nodes.len()];
    let mut intern = HashMap::new();
    for &id in &plan.order {
        let original = &plan.nodes[id];
        let mut node = original.clone();
        if node.predicate.is_none() {
            let mut children = Vec::new();
            let mut seen = HashSet::new();
            for &child in &original.children {
                let mapped = mapping[child];
                let child_node = &compact[mapped];
                let candidates = if child_node.predicate.is_none() && child_node.and == node.and {
                    child_node.children.clone()
                } else {
                    vec![mapped]
                };
                for c in candidates {
                    if seen.insert(c) {
                        children.push(c);
                    }
                }
            }
            if children.len() == 1 {
                mapping[id] = children[0];
                continue;
            }
            node.children = children;
        }
        let key = (
            node.predicate,
            node.negative,
            node.and,
            node.children.clone(),
        );
        let next = compact.len();
        let mapped = *intern.entry(key).or_insert(next);
        if mapped == next {
            compact.push(node);
        }
        mapping[id] = mapped;
    }
    plan.root = mapping[plan.root];
    plan.nodes = compact;
    plan.order = (0..plan.nodes.len()).collect();
    plan
}

#[cfg(test)]
mod tests;
