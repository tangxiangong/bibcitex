use super::*;

fn reference(key: &str, title: &str, author: &str) -> Reference {
    let mut reference: Reference = serde_json::from_value(serde_json::json!({
        "cite_key": key, "source": format!("@misc{{{key}}}"), "type_": "Misc"
    }))
    .unwrap();
    reference.title = Some(vec![Chunk::Normal(title.into())]);
    reference.author = Some(vec![author.into()]);
    reference
}
fn keys<'a>(source: &'a [Reference], indices: &[usize]) -> Vec<&'a str> {
    indices
        .iter()
        .map(|&i| source[i].cite_key.as_str())
        .collect()
}

#[test]
fn and_terms_match_noncontinuous_reversed_and_cross_field_text() {
    let source = vec![
        reference("cross", "Neural methods for graph analysis", "Smith"),
        reference("missing", "Graph analysis", "Smith"),
    ];
    let index = SearchIndex::new(&source);
    for query in ["graph neural", "NEURAL   graph", "  Smith\tgraph neural\n"] {
        assert_eq!(
            keys(&source, &index.search(query, SearchField::All)),
            ["cross"],
            "{query}"
        );
    }
    assert!(index.search("Smith graph", SearchField::Title).is_empty());
    assert_eq!(index.search("neural graph", SearchField::Title), [0]);
}

#[test]
fn accents_unicode_and_punctuation_are_tolerated_without_changing_metadata() {
    let source = vec![
        reference(
            "accent",
            "Gödel’s machine–learning ＡＬＧＯＲＩＴＨＭＳ",
            "Gödel",
        ),
        reference("plain", "Godel", "Nobody"),
    ];
    let index = SearchIndex::new(&source);
    assert_eq!(
        keys(&source, &index.search("Godel", SearchField::All)),
        ["plain", "accent"]
    );
    for query in [
        "go\u{308}del",
        "machine learning",
        "machine-learning",
        "algorithms",
        "Gödel's",
    ] {
        assert!(
            index.search(query, SearchField::Title).contains(&0),
            "{query}"
        );
    }
    assert_eq!(
        source[0].title.as_ref().unwrap()[0].get(),
        "Gödel’s machine–learning ＡＬＧＯＲＩＴＨＭＳ"
    );
}

#[test]
fn chinese_is_literal_and_math_chunks_keep_their_existing_boundaries() {
    let mut source = vec![
        reference("math", "", "张小明"),
        reference("other", "机器算法", "王"),
    ];
    source[0].title = Some(vec![
        Chunk::Normal("机器研究 micro".into()),
        Chunk::Verbatim("scope".into()),
        Chunk::Normal(" ".into()),
        Chunk::Math("x^2".into()),
        Chunk::Normal("算法".into()),
    ]);
    source[0].note = Some(vec![
        Chunk::Normal("边界".into()),
        Chunk::Math("y_1".into()),
        Chunk::Normal("说明".into()),
    ]);
    let index = SearchIndex::new(&source);
    for query in ["机器 算法 张", "microscope", "x^2算法", "边界y_1说明"] {
        assert_eq!(index.search(query, SearchField::All), [0], "{query}");
    }
    assert!(index.search("机气", SearchField::Title).is_empty());
}

#[test]
fn meaningful_symbols_and_punctuation_only_queries_remain_literal() {
    let source = vec![
        reference("cpp", "C++", ""),
        reference("sharp", "C#", ""),
        reference("c", "C", ""),
        reference("dash", "A-B", ""),
        reference("plain", "AB", ""),
    ];
    let index = SearchIndex::new(&source);
    assert_eq!(index.search("C++", SearchField::Title), [0]);
    assert_eq!(index.search("C#", SearchField::Title), [1]);
    assert_eq!(index.search("-", SearchField::Title), [3]);
    assert!(index.search("!", SearchField::Title).is_empty());
    assert!(index.search("$", SearchField::All).is_empty());
    assert!(index.search("author:Smith", SearchField::All).is_empty());
}

#[test]
fn full_journal_and_complete_years_obey_field_limits() {
    let mut source = vec![reference("paper", "Example", "Doe")];
    source[0].journal = Some("J. Test".into());
    source[0].full_journal = Some("International Testing Journal".into());
    source[0].year = Some(2026);
    let index = SearchIndex::new(&source);
    assert_eq!(
        index.search("international journal", SearchField::Journal),
        [0]
    );
    assert_eq!(index.search("Doe 2026", SearchField::All), [0]);
    assert_eq!(index.search("2026", SearchField::Year), [0]);
    assert!(index.search("202", SearchField::Year).is_empty());
    assert!(index.search("2025", SearchField::Year).is_empty());
    assert!(index.search("2026 2025", SearchField::Year).is_empty());
}

#[test]
fn exact_key_word_prefix_substring_and_fuzzy_matches_have_stable_priority() {
    let source = vec![
        reference("fuzzy", "Netwrok", ""),
        reference("substring", "Internetwork", ""),
        reference("prefix", "Networking", ""),
        reference("title", "Network", ""),
        reference("network", "Unrelated", ""),
    ];
    let index = SearchIndex::new(&source);
    assert_eq!(
        keys(&source, &index.search("network", SearchField::All)),
        ["network", "title", "prefix", "substring", "fuzzy"]
    );
}

#[test]
fn phrase_field_weight_and_proximity_break_equally_strong_matches() {
    let source = vec![
        reference("far", "Graph methods with many different neural models", ""),
        reference("near", "Graph and neural models", ""),
        reference("phrase", "Graph neural models", ""),
        reference("author", "Unrelated", "Graph neural models"),
    ];
    let index = SearchIndex::new(&source);
    assert_eq!(
        keys(&source, &index.search("graph neural", SearchField::All)),
        ["phrase", "author", "near", "far"]
    );
}

#[test]
fn fuzzy_matching_allows_one_edit_and_never_drops_an_and_term() {
    let source = vec![
        reference("hit", "Network algorithms", "Smith"),
        reference("missing", "Network", "Smith"),
    ];
    let index = SearchIndex::new(&source);
    for query in [
        "netwrok algorithms",
        "netwrk algorithms",
        "netwoork algorithms",
        "netwark algorithms",
    ] {
        assert_eq!(index.search(query, SearchField::Title), [0], "{query}");
    }
    for query in ["nexxork", "netwrok absent", "alg", "netwrok 2026"] {
        let found = index.search(query, SearchField::Title);
        if query == "alg" {
            assert_eq!(found, [0]);
        } else {
            assert!(found.is_empty(), "{query}");
        }
    }
    assert_eq!(index.search("Smtih", SearchField::Author).len(), 2);
}

#[test]
fn fuzzy_matching_excludes_keys_notes_short_words_numbers_and_formulas() {
    let mut source = vec![reference("network", "cat x^2 2026", "")];
    source[0].note = Some(vec![Chunk::Normal("algorithms".into())]);
    let index = SearchIndex::new(&source);
    for query in ["netwrok", "algoritms", "cta", "x^3", "2025"] {
        assert!(index.search(query, SearchField::All).is_empty(), "{query}");
    }
}

#[test]
fn mathematical_words_are_literal_but_surrounding_title_prose_can_be_corrected() {
    let mut source = vec![reference("formula", "", "")];
    source[0].title = Some(vec![
        Chunk::Normal("Algorithms ".into()),
        Chunk::Math("network".into()),
    ]);
    let index = SearchIndex::new(&source);
    assert!(index.search("netwrok", SearchField::Title).is_empty());
    assert_eq!(index.search("algoritms network", SearchField::Title), [0]);
}

#[test]
fn sufficient_normal_results_skip_fuzzy_supplementation_after_filtering() {
    let mut source: Vec<_> = (0..20)
        .map(|i| reference(&format!("normal{i}"), "Network", ""))
        .collect();
    source.push(reference("fuzzy", "Netwrok", ""));
    let index = SearchIndex::new(&source);
    assert_eq!(index.search("network", SearchField::Title).len(), 20);
    assert_eq!(
        keys(
            &source,
            &index.search_filtered("network", SearchField::Title, |i| i == 0 || i == 20)
        ),
        ["normal0", "fuzzy"]
    );
}

#[test]
fn empty_queries_keep_missing_fields_and_source_order() {
    let mut source = vec![reference("z", "", ""), reference("a", "", "")];
    source[0].title = None;
    source[0].author = None;
    let index = SearchIndex::new(&source);
    for field in [
        SearchField::All,
        SearchField::Author,
        SearchField::Title,
        SearchField::Journal,
        SearchField::Year,
    ] {
        assert_eq!(index.search(" \t\n", field), [0, 1]);
    }
    assert_eq!(
        index.search_filtered("", SearchField::Year, |i| i == 1),
        [1]
    );
}

#[test]
fn parallel_sequential_and_cached_search_agree_without_duplicate_results() {
    let source: Vec<_> = (0..150)
        .rev()
        .map(|i| {
            reference(
                &format!("key{i:03}"),
                if i % 2 == 0 {
                    "Neural network graph"
                } else {
                    "Neural netwrok graph"
                },
                "Smith",
            )
        })
        .collect();
    let index = SearchIndex::new(&source);
    for query in [
        "network",
        "netwrok",
        "Smith graph",
        "NEURAL  graph",
        "",
        "nothing",
        "network network",
    ] {
        let sequential = index.execute(query, SearchField::All, &|_| true, false);
        assert_eq!(
            index.execute(query, SearchField::All, &|_| true, true),
            sequential,
            "{query}"
        );
        assert_eq!(
            par_search_references(&source, query),
            seq_search_references(&source, query),
            "{query}"
        );
        assert_eq!(
            sequential.iter().copied().collect::<HashSet<_>>().len(),
            sequential.len()
        );
    }
}

#[test]
fn folded_substrings_are_normal_matches_and_are_never_duplicated_as_fuzzy_results() {
    let source = vec![reference("accent", "Métagraphic network", "")];
    let index = SearchIndex::new(&source);
    assert_eq!(index.search("etagraph", SearchField::Title), [0]);
}

#[test]
fn long_inputs_are_not_truncated_and_duplicate_terms_do_not_change_membership() {
    let source = vec![reference("hit", "Graph", "")];
    let index = SearchIndex::new(&source);
    assert_eq!(
        index.search(&"graph ".repeat(2_000), SearchField::Title),
        [0]
    );
    assert!(
        index
            .search(&"a".repeat(20_000), SearchField::Title)
            .is_empty()
    );
    assert!(
        index
            .search(
                &format!("{} absent", "graph ".repeat(2_000)),
                SearchField::Title
            )
            .is_empty()
    );
}

#[test]
fn mathematical_operators_remain_distinct_in_titles_and_notes() {
    let mut source = vec![
        reference("equal", "", ""),
        reference("minus", "", ""),
        reference("divide", "", ""),
    ];
    for (record, formula) in source.iter_mut().zip(["x=1", "x-1", "x/1"]) {
        record.title = Some(vec![
            Chunk::Normal("Gödel machine-learning ".into()),
            Chunk::Math(formula.into()),
            Chunk::Normal(" methods".into()),
        ]);
        record.note = Some(vec![Chunk::Math(formula.into())]);
    }
    let index = SearchIndex::new(&source);
    for (i, formula) in ["x=1", "x-1", "x/1"].into_iter().enumerate() {
        assert_eq!(index.search(formula, SearchField::All), [i], "{formula}");
        assert_eq!(
            index.search(
                &format!("godel learning {formula} methods"),
                SearchField::Title
            ),
            [i]
        );
    }
}

#[test]
fn repeated_keywords_keep_the_same_relevance_order() {
    let source = vec![
        reference("near", "Graph neural models", ""),
        reference("repeat", "Graph graph neural models", ""),
    ];
    let index = SearchIndex::new(&source);
    let expected = index.search("graph neural", SearchField::All);
    for query in [
        "graph graph neural",
        "GRAPH graph neural GRAPH",
        "graph\tgraph neural",
    ] {
        assert_eq!(index.search(query, SearchField::All), expected, "{query}");
    }
}
