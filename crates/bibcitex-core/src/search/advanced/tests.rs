use super::super::{SearchField, SearchIndex, query};
use super::*;
fn references(source: &str) -> Vec<Reference> {
    let bib = biblatex::Bibliography::parse(source).unwrap();
    crate::utils::read_bibliography(bib)
}
fn hits(index: &AdvancedIndex, text: &str, field: &str) -> Vec<usize> {
    let analysis = query::analyze(text);
    assert_eq!(
        analysis.status,
        query::Status::Valid,
        "{text}: {:?}",
        analysis.diagnostics
    );
    index
        .search(
            &analysis.document.unwrap(),
            field,
            |_| true,
            |_, _, _| false,
        )
        .unwrap()
        .into_iter()
        .map(|h| h.index)
        .collect()
}
#[test]
fn shared_predicates_preserve_boolean_complements_and_field_provenance() {
    let refs = references(
        "@article{A,title={network}} @article{B,title={netwrok}} @article{C,title={other},author={network}}",
    );
    let index = AdvancedIndex::new(&refs);
    assert!(hits(&index, "network -network", "all").is_empty());
    assert_eq!(hits(&index, "network OR NOT network", "all").len(), 3);
    assert_eq!(
        hits(&index, "NOT NOT network", "all"),
        hits(&index, "network", "all")
    );
    assert_eq!(hits(&index, "network OR title:network", "title"), vec![0]);
    let doc = query::analyze("network OR title:network").document.unwrap();
    let rendered = query::serialize(&doc).unwrap();
    assert_eq!(hits(&index, &rendered, "author"), vec![0, 2]);
}
#[test]
fn equivalent_and_forms_reuse_the_ordinary_rank_dimensions() {
    let refs = references(
        "@article{A,title={Graph Neural Networks}} @article{B,title={Neural Graph Networks}} @article{C,title={Graph methods for Neural Networks}}",
    );
    let index = AdvancedIndex::new(&refs);
    let ordinary = SearchIndex::new(&refs).search("graph neural", SearchField::All);
    assert_eq!(ordinary, vec![0, 1, 2]);
    for text in [
        "graph neural",
        "graph AND neural",
        "(graph neural)",
        "graph AND (neural)",
        "title:(graph neural)",
        "title:graph title:neural",
    ] {
        assert_eq!(hits(&index, text, "all"), ordinary, "{text}");
    }
}
#[test]
fn precedence_scopes_and_strict_phrases() {
    let refs = references(
        "@article{A,title={graph neural},author={Smith}} @article{B,title={network},author={Jones}} @article{C,title={graph methods},author={Jones}}",
    );
    let index = AdvancedIndex::new(&refs);
    assert_eq!(
        hits(&index, "graph OR network author:Smith", "all"),
        vec![0, 2]
    );
    assert_eq!(
        hits(&index, "(graph OR network) author:Smith", "all"),
        vec![0]
    );
    assert_eq!(hits(&index, "\"graph neural\"", "all"), vec![0]);
    assert_eq!(
        hits(&index, "title:(graph OR author:Smith)", "all"),
        vec![0, 2]
    );
}
#[test]
fn range_identifier_and_missing_field_semantics() {
    let refs = references(
        "@article{A,title={A},year={2020},doi={10.1000/x},isbn={978-1-23}} @book{B,title={B},year={2024}} @article{C,title={C}}",
    );
    let index = AdvancedIndex::new(&refs);
    assert_eq!(hits(&index, "year:2020..2024", "all"), vec![0, 1]);
    assert_eq!(hits(&index, "-year:>=2020", "all"), vec![2]);
    assert_eq!(
        hits(&index, "doi:https://doi.org/10.1000/X", "all"),
        vec![0]
    );
    assert_eq!(hits(&index, "isbn:978123", "all"), vec![0]);
    assert_eq!(hits(&index, "type:book", "all"), vec![1]);
}

#[test]
fn evidence_points_to_original_unicode_and_excludes_negative_terms() {
    let refs = references(
        "@article{A,title={Café learning and learnings},author={Smith, Jane and Jones, Bob}}",
    );
    let index = AdvancedIndex::new(&refs);
    let query = query::analyze("cafe \"learning\" -absent")
        .document
        .unwrap();
    let hit = &index
        .search(&query, "all", |_| true, |_, _, _| false)
        .unwrap()[0];
    let title = "Café learning and learnings";
    let ranges: Vec<_> = hit
        .ranges
        .iter()
        .filter(|r| r.field_name == "title")
        .map(|r| &title[r.start..r.end])
        .collect();
    assert!(ranges.contains(&"Café"));
    assert!(ranges.contains(&"learning"));
    assert_eq!(ranges.iter().filter(|&&r| r == "learning").count(), 1);
}
#[test]
fn phrases_do_not_cross_author_entries_or_fold_accents_and_symbols() {
    let refs = references(
        "@article{A,title={Café C++ a+b},author={Smith, Jane and Jones, Bob}} @article{B,title={Cafe C# a-b}}",
    );
    let index = AdvancedIndex::new(&refs);
    assert_eq!(hits(&index, "\"café\"", "all"), vec![0]);
    assert_eq!(hits(&index, "\"cafe\"", "all"), vec![1]);
    assert!(hits(&index, "author:\"Jane Smith Bob Jones\"", "all").is_empty());
    assert_eq!(hits(&index, "author:Smith author:Jones", "all"), vec![0]);
    assert_eq!(hits(&index, "\"C++\"", "all"), vec![0]);
}
#[test]
fn fuzzy_threshold_and_external_filter_apply_to_whole_expression() {
    let mut text = String::new();
    for i in 0..20 {
        text.push_str(&format!("@article{{p{i},title={{network}},year={{2020}}}}"));
    }
    text.push_str("@book{typo,title={netwrok},year={2024}}");
    let refs = references(&text);
    let index = AdvancedIndex::new(&refs);
    assert_eq!(hits(&index, "network", "all").len(), 20);
    assert_eq!(hits(&index, "network year:2024", "all"), vec![20]);
    assert!(
        hits(&index, "network -year:2024", "all")
            .iter()
            .all(|&i| i < 20)
    );
    assert!(hits(&index, "year:2020..2024", "year").len() == 21);
    assert!(hits(&index, "2020..2024", "year").is_empty());
}

#[test]
fn semantic_and_forms_preserve_every_legacy_rank_component() {
    let refs = references(
        "@article{A,title={Graph Neural Networks}} @article{B,title={Neural Graph Networks}} @article{C,title={Graph methods for Neural Networks}}",
    );
    let ordinary = SearchIndex::new(&refs);
    let advanced = AdvancedIndex::new(&refs);
    for text in [
        "graph neural",
        "graph AND neural",
        "(graph neural)",
        "graph AND (neural)",
    ] {
        let doc = query::analyze(text).document.unwrap();
        let plan = compile(&doc, "all");
        let mut cache = HashMap::new();
        for (id, record) in advanced.records.iter().enumerate() {
            let actual = record.score(&[0, 1], &plan.predicates, false, &mut cache);
            let expected = ordinary.documents[id]
                .score(
                    &super::super::Query::new("graph neural"),
                    SearchField::All,
                    false,
                )
                .unwrap();
            assert_eq!(actual, expected, "{text} record {id}");
        }
    }
}
#[test]
fn complete_keys_preserve_significant_query_whitespace() {
    let refs = references("@article{Exact,title={graph}}");
    let index = AdvancedIndex::new(&refs);
    assert_eq!(hits(&index, "key:exact", "all"), vec![0]);
    assert!(hits(&index, "key:\" Exact \"", "all").is_empty());
}

#[test]
fn exact_predicates_do_not_intern_distinct_punctuation_or_compatibility_values() {
    let refs = references("@article{A-B,title={one}} @article{C,title={two}}");
    let index = AdvancedIndex::new(&refs);
    assert_eq!(hits(&index, "key:A-B -key:A–B", "all"), vec![0]);
    assert!(hits(&index, "key:A-B key:A–B", "all").is_empty());
    assert_eq!(hits(&index, "key:A-B -key:Ａ-B", "all"), vec![0]);
    let doc = query::analyze("tag:A-B -tag:A–B").document.unwrap();
    let found = index
        .search(&doc, "all", |_| true, |i, name, _| i == 0 && name == "A-B")
        .unwrap();
    assert_eq!(found.iter().map(|h| h.index).collect::<Vec<_>>(), vec![0]);
}

#[test]
fn nested_scopes_and_negation_keep_results_after_serialization() {
    let refs = references(
        "@article{A,title={graph},author={Jones}} @article{B,title={other},author={Smith}}",
    );
    let index = AdvancedIndex::new(&refs);
    for text in [
        "title:(author:Smith)",
        "title:(NOT graph)",
        "title:(author:(NOT Smith))",
    ] {
        let doc = query::analyze(text).document.unwrap();
        assert_eq!(
            hits(&index, text, "all"),
            hits(&index, &query::serialize(&doc).unwrap(), "all")
        );
    }
}

#[test]
fn phrase_boundaries_cover_extended_latin_without_treating_math_as_letters() {
    assert!(!phrase_matches("ḁbc", "ḁ"));
    assert!(!phrase_matches("aḁ", "ḁ"));
    assert!(phrase_matches("a×b", "×"));
    assert!(phrase_matches("中文词", "中文"));
    let index = AdvancedIndex::new(&references("@article{A,title={ḁbc}} @article{B,title={ḁ}}"));
    let doc = query::analyze("\"ḁ\"").document.unwrap();
    let found = index
        .search(&doc, "all", |_| true, |_, _, _| false)
        .unwrap();
    assert_eq!(found.iter().map(|h| h.index).collect::<Vec<_>>(), vec![1]);
    assert!(
        found[0]
            .ranges
            .iter()
            .any(|r| r.field_name == "title" && r.start == 0 && r.end == "ḁ".len())
    );
}

#[test]
fn spelling_evidence_identifies_only_successful_positive_supplement_terms() {
    let refs = references("@article{A,title={netwrok graph}} @article{B,title={network graph}}");
    let index = AdvancedIndex::new(&refs);
    let doc = query::analyze("(network OR absent) graph -survey")
        .document
        .unwrap();
    let found = index
        .search(&doc, "all", |_| true, |_, _, _| false)
        .unwrap();
    assert_eq!(found[0].index, 1);
    assert!(found[0].fuzzy_terms.is_empty());
    assert_eq!(found[1].index, 0);
    assert_eq!(found[1].fuzzy_terms, vec!["network"]);
}
