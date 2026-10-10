//! Ordinary, whitespace-separated AND search with deterministic relevance ranking.
use crate::core::bib::Reference;
use biblatex::Chunk;
use rayon::prelude::*;
use std::{cmp::Reverse, collections::HashSet, mem::size_of, ops::Range};
use unicode_normalization::{UnicodeNormalization, char::is_combining_mark};

const THRESHOLD_PARALLEL_SIZE: usize = 2_048;
// Supplement small result sets; never discard normal matches or limit returned records.
const FUZZY_SUPPLEMENT_THRESHOLD: usize = 20;

/// The existing ordinary-search field selector.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SearchField {
    All,
    Author,
    Title,
    Journal,
    Year,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum FieldKind {
    Key,
    Title,
    Author,
    Journal,
    Year,
    Note,
}
impl FieldKind {
    fn selected(self, field: SearchField) -> bool {
        field == SearchField::All
            || matches!(
                (self, field),
                (Self::Title, SearchField::Title)
                    | (Self::Author, SearchField::Author)
                    | (Self::Journal, SearchField::Journal)
                    | (Self::Year, SearchField::Year)
            )
    }
    fn weight(self) -> u64 {
        match self {
            Self::Key => 6,
            Self::Title => 5,
            Self::Author => 4,
            Self::Journal => 3,
            Self::Year => 2,
            Self::Note => 1,
        }
    }
    fn fuzzy(self) -> bool {
        matches!(self, Self::Title | Self::Author | Self::Journal)
    }
}

struct SearchText {
    normal: String,
    folded: Option<String>,
    fuzzy_words: Vec<Range<usize>>,
    fuzzy_source: Option<Box<str>>,
}
impl SearchText {
    fn new(value: &str) -> Self {
        let mut normal = normalize(value);
        let mut folded = fold(&normal);
        // Immutable library snapshots do not need spare string/vector capacity.
        normal.shrink_to_fit();
        folded.shrink_to_fit();
        let folded = (folded != normal && !folded.is_empty()).then_some(folded);
        let text = folded.as_deref().unwrap_or(&normal);
        let mut offset = 0;
        let mut fuzzy_words: Vec<_> = text
            .split_inclusive(|c: char| !word_character(c))
            .filter_map(|part| {
                let start = offset;
                offset += part.len();
                let word = part.trim_end_matches(|c: char| !word_character(c));
                (word.len() >= 4
                    && word.len() <= 65
                    && word.bytes().all(|b| b.is_ascii_alphabetic()))
                .then_some(start..start + word.len())
            })
            .collect();
        fuzzy_words.shrink_to_fit();
        Self {
            normal,
            folded,
            fuzzy_words,
            fuzzy_source: None,
        }
    }
    fn from_chunks(chunks: Option<&[Chunk]>) -> Self {
        let mut text = Self::new(&chunk_text(chunks));
        if chunks
            .into_iter()
            .flatten()
            .any(|chunk| matches!(chunk, Chunk::Math(_)))
        {
            // Fold prose without erasing operators inside formula chunks. Preserve
            // chunk whitespace until the whole field has been assembled.
            let mut folded = String::new();
            for chunk in chunks.into_iter().flatten() {
                let normal: String = normalized_chars(chunk.get()).collect();
                if matches!(chunk, Chunk::Math(_)) {
                    folded.push_str(&normal);
                } else {
                    folded.extend(folded_chars(&normal));
                }
            }
            let folded = collapse_spaces(folded.chars());
            text.folded = (folded != text.normal && !folded.is_empty()).then_some(folded);
            // Preserve mathematical text for literal search, but do not spell-correct it.
            let prose: String = chunks
                .into_iter()
                .flatten()
                .map(|chunk| match chunk {
                    Chunk::Math(_) => " ",
                    chunk => chunk.get(),
                })
                .collect();
            let fuzzy = Self::new(&prose);
            text.fuzzy_words = fuzzy.fuzzy_words;
            text.fuzzy_source = Some(fuzzy.folded.unwrap_or(fuzzy.normal).into_boxed_str());
        }
        text
    }
    fn folded(&self) -> &str {
        self.folded.as_deref().unwrap_or(&self.normal)
    }
    fn exact(&self, term: &Self, kind: FieldKind) -> Option<TextMatch> {
        if kind == FieldKind::Year {
            return (self.normal == term.normal).then_some(TextMatch {
                quality: 10,
                start: 0,
                end: self.normal.len(),
            });
        }
        let normal = text_match(&self.normal, &term.normal);
        let folded = if self.folded.is_some() || term.folded.is_some() {
            text_match(self.folded(), term.folded()).map(|m| TextMatch {
                quality: m.quality - 1,
                ..m
            })
        } else {
            None
        };
        normal.into_iter().chain(folded).max_by_key(|m| m.quality)
    }
    fn fuzzy_match(&self, term: &Self) -> bool {
        let needle = term.folded();
        let source = self.fuzzy_source.as_deref().unwrap_or(self.folded());
        eligible_fuzzy(needle)
            && self
                .fuzzy_words
                .iter()
                .any(|range| one_edit_apart(needle.as_bytes(), source[range.clone()].as_bytes()))
    }
    fn allocated_bytes(&self) -> usize {
        self.normal.capacity()
            + self.folded.as_ref().map_or(0, String::capacity)
            + self.fuzzy_words.capacity() * size_of::<Range<usize>>()
            + self.fuzzy_source.as_ref().map_or(0, |text| text.len())
    }
}
struct IndexedField {
    kind: FieldKind,
    text: SearchText,
}
struct SearchDocument {
    fields: Vec<IndexedField>,
}
impl SearchDocument {
    fn new(reference: &Reference) -> Self {
        let mut fields = Vec::new();
        let mut add = |kind, text: SearchText| {
            if !text.normal.is_empty() {
                fields.push(IndexedField { kind, text });
            }
        };
        add(FieldKind::Key, SearchText::new(&reference.cite_key));
        add(
            FieldKind::Title,
            SearchText::from_chunks(reference.title.as_deref()),
        );
        for author in reference.author.iter().flatten() {
            add(FieldKind::Author, SearchText::new(author));
        }
        for journal in [
            reference.journal.as_deref(),
            reference.full_journal.as_deref(),
        ]
        .into_iter()
        .flatten()
        {
            add(FieldKind::Journal, SearchText::new(journal));
        }
        if let Some(year) = reference.year {
            add(FieldKind::Year, SearchText::new(&year.to_string()));
        }
        add(
            FieldKind::Note,
            SearchText::from_chunks(reference.note.as_deref()),
        );
        fields.shrink_to_fit();
        Self { fields }
    }
    fn score(&self, query: &Query, field: SearchField, fuzzy: bool) -> Option<Rank> {
        let mut rank = Rank {
            weakest: 10,
            ..Rank::default()
        };
        for term in &query.terms {
            let mut best = None;
            for f in self.fields.iter().filter(|f| f.kind.selected(field)) {
                let quality =
                    f.text.exact(term, f.kind).map(|m| m.quality).or_else(|| {
                        (fuzzy && f.kind.fuzzy() && f.text.fuzzy_match(term)).then_some(1)
                    });
                if let Some(quality) = quality {
                    best = best.max(Some((quality, f.kind.weight())));
                }
            }
            let (quality, weight) = best?;
            rank.weakest = rank.weakest.min(quality);
            rank.field_score = rank.field_score.saturating_add(weight * u64::from(quality));
        }
        for f in self.fields.iter().filter(|f| f.kind.selected(field)) {
            if f.kind == FieldKind::Key {
                rank.exact_key = if f.text.normal == query.whole.normal {
                    2
                } else if f.text.folded() == query.whole.folded() {
                    1
                } else {
                    0
                };
            }
            if let Some(m) = f.text.exact(&query.whole, f.kind) {
                rank.phrase = rank.phrase.max(m.quality);
            }
            if query.terms.len() > 1 && f.kind != FieldKind::Year {
                let mut start = usize::MAX;
                let mut end = 0;
                let mut same_field = true;
                for term in &query.terms {
                    if let Some(m) = f.text.exact(term, f.kind) {
                        start = start.min(m.start);
                        end = end.max(m.end);
                    } else {
                        same_field = false;
                        break;
                    }
                }
                if same_field {
                    rank.same_field = true;
                    rank.proximity = rank.proximity.max(usize::MAX - end.saturating_sub(start));
                }
            }
        }
        Some(rank)
    }
}
#[derive(Default, PartialEq, Eq, PartialOrd, Ord)]
struct Rank {
    exact_key: u8,
    weakest: u8,
    phrase: u8,
    same_field: bool,
    field_score: u64,
    proximity: usize,
}
struct Query {
    whole: SearchText,
    terms: Vec<SearchText>,
}
impl Query {
    fn new(query: &str) -> Self {
        let normal = normalize(query);
        let mut seen = HashSet::new();
        let words: Vec<_> = normal
            .split_whitespace()
            .filter(|word| seen.insert(*word))
            .collect();
        // Phrase and exact-key ranking must use the same deduplicated query as
        // term matching; repeated input must not change the relevance order.
        let whole = SearchText::new(&words.join(" "));
        let terms = words.into_iter().map(SearchText::new).collect();
        Self { whole, terms }
    }
}

/// Preprocessed search texts in the same order as the source references.
/// Keep this index and its source bibliography in one immutable cache snapshot.
pub struct SearchIndex {
    documents: Vec<SearchDocument>,
}
impl SearchIndex {
    /// Builds search texts without altering display metadata or BibTeX source.
    pub fn new(references: &[Reference]) -> Self {
        Self {
            documents: references.iter().map(SearchDocument::new).collect(),
        }
    }
    /// Returns all matching source indices, ranked for nonempty queries.
    pub fn search(&self, query: &str, field: SearchField) -> Vec<usize> {
        self.search_filtered(query, field, |_| true)
    }
    /// Applies the existing type filter before matching and fuzzy supplementation.
    /// Empty queries return eligible source indices in their original order.
    pub fn search_filtered(
        &self,
        query: &str,
        field: SearchField,
        include: impl Fn(usize) -> bool + Sync,
    ) -> Vec<usize> {
        self.execute(
            query,
            field,
            &include,
            self.documents.len() >= THRESHOLD_PARALLEL_SIZE,
        )
    }
    fn execute(
        &self,
        query: &str,
        field: SearchField,
        include: &(impl Fn(usize) -> bool + Sync),
        parallel: bool,
    ) -> Vec<usize> {
        let query = Query::new(query);
        if query.terms.is_empty() {
            return (0..self.documents.len()).filter(|&i| include(i)).collect();
        }
        let score = |(i, document): (usize, &SearchDocument)| {
            include(i)
                .then(|| document.score(&query, field, false))
                .flatten()
                .map(|rank| (i, rank))
        };
        let mut matches: Vec<_> = if parallel {
            self.documents
                .par_iter()
                .enumerate()
                .filter_map(score)
                .collect()
        } else {
            self.documents
                .iter()
                .enumerate()
                .filter_map(score)
                .collect()
        };
        if matches.len() < FUZZY_SUPPLEMENT_THRESHOLD
            && query.terms.iter().any(|term| eligible_fuzzy(term.folded()))
        {
            let fuzzy_score = |(i, document): (usize, &SearchDocument)| {
                if !include(i) {
                    return None;
                }
                let rank = document.score(&query, field, true)?;
                (rank.weakest == 1).then_some((i, rank))
            };
            let fuzzy_matches: Vec<_> = if parallel {
                self.documents
                    .par_iter()
                    .enumerate()
                    .filter_map(fuzzy_score)
                    .collect()
            } else {
                self.documents
                    .iter()
                    .enumerate()
                    .filter_map(fuzzy_score)
                    .collect()
            };
            matches.extend(fuzzy_matches);
        }
        matches.sort_unstable_by(|(a, ar), (b, br)| {
            br.cmp(ar)
                .then_with(|| {
                    self.documents[*a]
                        .fields
                        .first()
                        .map(|f| &f.text.normal)
                        .cmp(&self.documents[*b].fields.first().map(|f| &f.text.normal))
                })
                .then_with(|| a.cmp(b))
        });
        matches.into_iter().map(|(i, _)| i).collect()
    }
    /// Estimated owned allocation size, used to bound the search-text cache.
    pub fn allocated_bytes(&self) -> usize {
        self.documents.capacity() * size_of::<SearchDocument>()
            + self
                .documents
                .iter()
                .map(|d| {
                    d.fields.capacity() * size_of::<IndexedField>()
                        + d.fields
                            .iter()
                            .map(|f| f.text.allocated_bytes())
                            .sum::<usize>()
                })
                .sum::<usize>()
    }
}
fn chunk_text(chunks: Option<&[Chunk]>) -> String {
    // Chunks already carry their spacing; joining with new spaces breaks styled words.
    chunks.into_iter().flatten().map(Chunk::get).collect()
}
fn normalize(value: &str) -> String {
    collapse_spaces(normalized_chars(value))
}
fn normalized_chars(value: &str) -> impl Iterator<Item = char> + '_ {
    value.nfkc().flat_map(char::to_lowercase).map(|c| match c {
        '\u{2010}'..='\u{2015}' | '\u{2212}' => '-',
        '\u{2018}' | '\u{2019}' => '\'',
        '\u{201c}' | '\u{201d}' => '"',
        c => c,
    })
}
fn fold(value: &str) -> String {
    collapse_spaces(folded_chars(value))
}
fn folded_chars(value: &str) -> impl Iterator<Item = char> + '_ {
    value
        .nfd()
        .filter(|&c| !is_combining_mark(c))
        .map(|c| match c {
            '-' | ',' | '.' | ':' | ';' | '(' | ')' | '[' | ']' | '{' | '}' | '/' | '\\' | '\''
            | '"' | '!' | '?' | '=' | '|' => ' ',
            c => c,
        })
}
fn collapse_spaces(chars: impl Iterator<Item = char>) -> String {
    let mut text = String::new();
    let mut space = false;
    for c in chars {
        if c.is_whitespace() {
            space = !text.is_empty();
        } else {
            if space {
                text.push(' ');
                space = false;
            }
            text.push(c);
        }
    }
    text
}
fn word_character(c: char) -> bool {
    c.is_alphanumeric() || is_combining_mark(c) || matches!(c, '+' | '#' | '_' | '$' | '@' | '^')
}
#[derive(Clone, Copy)]
struct TextMatch {
    quality: u8,
    start: usize,
    end: usize,
}
fn text_match(text: &str, needle: &str) -> Option<TextMatch> {
    if needle.is_empty() {
        return None;
    }
    if text == needle {
        return Some(TextMatch {
            quality: 10,
            start: 0,
            end: text.len(),
        });
    }
    text.match_indices(needle)
        .map(|(start, _)| {
            let end = start + needle.len();
            let before = text[..start]
                .chars()
                .next_back()
                .is_none_or(|c| !word_character(c));
            let after = text[end..]
                .chars()
                .next()
                .is_none_or(|c| !word_character(c));
            TextMatch {
                quality: if before && after {
                    8
                } else if before {
                    6
                } else {
                    4
                },
                start,
                end,
            }
        })
        .max_by_key(|m| (m.quality, Reverse(m.start)))
}
fn eligible_fuzzy(word: &str) -> bool {
    (5..=64).contains(&word.len()) && word.bytes().all(|b| b.is_ascii_alphabetic())
}
fn one_edit_apart(a: &[u8], b: &[u8]) -> bool {
    if a.len().abs_diff(b.len()) > 1 {
        return false;
    }
    let shared = a.iter().zip(b).take_while(|(a, b)| a == b).count();
    if shared == a.len().min(b.len()) {
        return true;
    }
    match a.len().cmp(&b.len()) {
        std::cmp::Ordering::Less => a[shared..] == b[shared + 1..],
        std::cmp::Ordering::Greater => a[shared + 1..] == b[shared..],
        std::cmp::Ordering::Equal => {
            a[shared + 1..] == b[shared + 1..]
                || (shared + 1 < a.len()
                    && a[shared] == b[shared + 1]
                    && a[shared + 1] == b[shared]
                    && a[shared + 2..] == b[shared + 2..])
        }
    }
}
fn search_owned(
    references: &[Reference],
    query: &str,
    field: SearchField,
    parallel: bool,
) -> Vec<Reference> {
    SearchIndex::new(references)
        .execute(query, field, &|_| true, parallel)
        .into_iter()
        .map(|i| references[i].clone())
        .collect()
}
/// Search all existing ordinary-search fields with AND terms and relevance ranking.
pub fn search_references(references: &[Reference], query: &str) -> Vec<Reference> {
    search_owned(
        references,
        query,
        SearchField::All,
        references.len() >= THRESHOLD_PARALLEL_SIZE,
    )
}
/// Search ordinary fields in parallel, with the same ordering as sequential search.
pub fn par_search_references(references: &[Reference], query: &str) -> Vec<Reference> {
    search_owned(references, query, SearchField::All, true)
}
/// Search ordinary fields sequentially.
pub fn seq_search_references(references: &[Reference], query: &str) -> Vec<Reference> {
    search_owned(references, query, SearchField::All, false)
}
/// Search authors sequentially.
pub fn seq_search_references_by_author(references: &[Reference], query: &str) -> Vec<Reference> {
    search_owned(references, query, SearchField::Author, false)
}
/// Search authors in parallel.
pub fn par_search_references_by_author(references: &[Reference], query: &str) -> Vec<Reference> {
    search_owned(references, query, SearchField::Author, true)
}
/// Search authors with relevance ranking.
pub fn search_references_by_author(references: &[Reference], query: &str) -> Vec<Reference> {
    search_owned(
        references,
        query,
        SearchField::Author,
        references.len() >= THRESHOLD_PARALLEL_SIZE,
    )
}
/// Search titles sequentially.
pub fn seq_search_references_by_title(references: &[Reference], query: &str) -> Vec<Reference> {
    search_owned(references, query, SearchField::Title, false)
}
/// Search titles in parallel.
pub fn par_search_references_by_title(references: &[Reference], query: &str) -> Vec<Reference> {
    search_owned(references, query, SearchField::Title, true)
}
/// Search titles with relevance ranking.
pub fn search_references_by_title(references: &[Reference], query: &str) -> Vec<Reference> {
    search_owned(
        references,
        query,
        SearchField::Title,
        references.len() >= THRESHOLD_PARALLEL_SIZE,
    )
}
/// Search short and full journal names sequentially.
pub fn seq_search_references_by_journal(references: &[Reference], query: &str) -> Vec<Reference> {
    search_owned(references, query, SearchField::Journal, false)
}
/// Search short and full journal names in parallel.
pub fn par_search_references_by_journal(references: &[Reference], query: &str) -> Vec<Reference> {
    search_owned(references, query, SearchField::Journal, true)
}
/// Search short and full journal names with relevance ranking.
pub fn search_references_by_journal(references: &[Reference], query: &str) -> Vec<Reference> {
    search_owned(
        references,
        query,
        SearchField::Journal,
        references.len() >= THRESHOLD_PARALLEL_SIZE,
    )
}
/// Search complete years sequentially.
pub fn seq_search_references_by_year(references: &[Reference], query: &str) -> Vec<Reference> {
    search_owned(references, query, SearchField::Year, false)
}
/// Search complete years in parallel.
pub fn par_search_references_by_year(references: &[Reference], query: &str) -> Vec<Reference> {
    search_owned(references, query, SearchField::Year, true)
}
/// Search complete years without partial or fuzzy year matching.
pub fn search_references_by_year(references: &[Reference], query: &str) -> Vec<Reference> {
    search_owned(
        references,
        query,
        SearchField::Year,
        references.len() >= THRESHOLD_PARALLEL_SIZE,
    )
}

#[cfg(test)]
mod tests;
