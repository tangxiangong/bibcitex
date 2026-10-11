use super::*;

#[test]
fn lexical_context_and_committed_fields() {
    for (text, expected) in [
        ("tag:OR", Status::Valid),
        ("tag:(OR)", Status::Invalid),
        ("tag:(\"OR\")", Status::Valid),
        ("doi:doi:10.1000/x", Status::Valid),
        ("title:-graph", Status::Valid),
        (r"title\:graph", Status::Valid),
        ("abc:def", Status::Valid),
        ("abc:\"a b\"", Status::Invalid),
        ("abc:(a OR b)", Status::Invalid),
        ("author:", Status::Incomplete),
        ("author: ", Status::Invalid),
        ("author: Knuth", Status::Invalid),
        ("year:>", Status::Incomplete),
        ("year:>=", Status::Incomplete),
        ("year:> ", Status::Invalid),
        ("year:\">\"", Status::Invalid),
        ("(year:>)", Status::Invalid),
        ("graph OR OR network author:", Status::Invalid),
        ("year:abc author:", Status::Invalid),
        ("year:..", Status::Invalid),
        ("year:2020..", Status::Valid),
        ("type:art", Status::Invalid),
        ("", Status::Valid),
        ("(a)OR(b)", Status::Valid),
        ("NOT(a)", Status::Invalid),
        ("(a)(b)", Status::Invalid),
        ("- survey", Status::Invalid),
        ("abc:\"a b", Status::Invalid),
    ] {
        let result = analyze(text);
        assert_eq!(result.status, expected, "{text}: {:?}", result.diagnostics);
        assert_eq!(
            result.document.is_some(),
            expected == Status::Valid,
            "{text}"
        );
    }
}
#[test]
fn independent_resource_limits() {
    assert_eq!(
        analyze(&vec!["title:x"; 1024].join(" ")).status,
        Status::Valid
    );
    assert_eq!(
        analyze(&vec!["x"; 1025].join(" ")).diagnostics[0].code,
        "LeafLimitExceeded"
    );
    for n in [64, 65] {
        let text = format!("{}x{}", "(".repeat(n), ")".repeat(n));
        assert_eq!(
            analyze(&text).status,
            if n == 64 {
                Status::Valid
            } else {
                Status::Invalid
            }
        );
    }
    let clauses = vec!["(((((((x)))))))"; 1024].join(" AND ");
    assert_eq!(analyze(&format!("NOT {clauses}")).status, Status::Valid);
    assert_eq!(
        analyze(&format!("NOT NOT {clauses}")).diagnostics[0].code,
        "TokenLimitExceeded"
    );
    assert_eq!(
        analyze(&format!("\"{}\"", "x".repeat(65534))).status,
        Status::Valid
    );
    assert_eq!(
        analyze(&format!("\"{}\"", "x".repeat(65535))).diagnostics[0].code,
        "ByteLimitExceeded"
    );
}
#[test]
fn unicode_diagnostics_use_original_byte_offsets() {
    let text = "中文 author: ";
    let analysis = analyze(text);
    assert_eq!(analysis.status, Status::Invalid);
    for error in analysis.diagnostics {
        assert!(text.is_char_boundary(error.span.start));
        assert!(text.is_char_boundary(error.span.end));
    }
}

#[test]
fn bound_tree_validation_counts_nodes_and_depth_independently() {
    let leaf = || Node {
        kind: NodeKind::Word,
        value: "x".into(),
        tag_id: None,
        children: vec![],
        span: 0..1,
    };
    let mut doc = Document {
        nodes: vec![leaf()],
        root: Some(0),
    };
    for id in 0..255 {
        doc.nodes.push(Node {
            kind: NodeKind::Not,
            value: String::new(),
            tag_id: None,
            children: vec![id],
            span: 0..1,
        });
    }
    doc.root = Some(255);
    assert!(validate_document(&doc).is_ok());
    doc.nodes.push(Node {
        kind: NodeKind::Not,
        value: String::new(),
        tag_id: None,
        children: vec![255],
        span: 0..1,
    });
    doc.root = Some(256);
    assert_eq!(
        validate_document(&doc).unwrap_err().code,
        "DepthLimitExceeded"
    );
    let mut doc = Document::default();
    let mut children = Vec::new();
    for _ in 0..1023 {
        let leaf_id = doc.nodes.len();
        doc.nodes.push(leaf());
        for child in leaf_id..leaf_id + 3 {
            doc.nodes.push(Node {
                kind: NodeKind::Not,
                value: String::new(),
                tag_id: None,
                children: vec![child],
                span: 0..1,
            });
        }
        children.push(doc.nodes.len() - 1);
    }
    doc.nodes.push(Node {
        kind: NodeKind::And,
        value: String::new(),
        tag_id: None,
        children,
        span: 0..1,
    });
    while doc.nodes.len() < 4096 {
        let child = doc.nodes.len() - 1;
        doc.nodes.push(Node {
            kind: NodeKind::Not,
            value: String::new(),
            tag_id: None,
            children: vec![child],
            span: 0..1,
        });
    }
    doc.root = Some(4095);
    assert!(validate_document(&doc).is_ok());
    doc.nodes.push(leaf());
    assert_eq!(
        validate_document(&doc).unwrap_err().code,
        "NodeLimitExceeded"
    );
}
#[test]
fn cycles_shared_children_and_unreachable_nodes_are_rejected() {
    for children in [vec![0], vec![1, 1], vec![3]] {
        let doc = Document {
            nodes: vec![
                Node {
                    kind: NodeKind::And,
                    value: String::new(),
                    tag_id: None,
                    children,
                    span: 0..0,
                },
                Node {
                    kind: NodeKind::Word,
                    value: "x".into(),
                    tag_id: None,
                    children: vec![],
                    span: 0..1,
                },
            ],
            root: Some(0),
        };
        assert_eq!(validate_document(&doc).unwrap_err().code, "InvalidTree");
    }
}
#[test]
fn scoped_nesting_and_unicode_byte_budget_follow_source_rules() {
    let text = format!("{}x{}", "title:(".repeat(64), ")".repeat(64));
    assert_eq!(analyze(&text).status, Status::Valid);
    let text = format!("\"{}x\"", "中".repeat(21844));
    assert_eq!(text.len(), 65535);
    assert_eq!(analyze(&text).status, Status::Valid);
    assert_eq!(
        analyze(&(text + "xx")).diagnostics[0].code,
        "ByteLimitExceeded"
    );
    assert_eq!(analyze(&vec!["OR"; 100].join(" ")).diagnostics.len(), 32);
}
#[test]
fn canonical_roundtrips_keep_punctuation_and_field_provenance() {
    for text in [
        r"title\:graph OR tag:OR",
        r"title:(graph author:Smith)",
        r#"doi:doi:10.1000/x isbn:"978-1-23""#,
        r"f\(x\) NOT NOT network",
        r#"title:"café 中文" -tag:"machine learning""#,
        r"title:\alpha",
        r"title:a\ b",
    ] {
        let first = analyze(text);
        assert_eq!(first.status, Status::Valid, "{text}");
        let projection = serialize(&first.document.unwrap()).unwrap();
        let second = analyze(&projection);
        assert_eq!(second.status, Status::Valid, "{text} -> {projection}");
        assert_eq!(projection, serialize(&second.document.unwrap()).unwrap());
    }
}

#[test]
fn unclosed_quotes_wait_for_content_without_hiding_prior_errors() {
    for text in ["\"", "\" ", "title:\"", "year:\"", "year:\"abc"] {
        assert_eq!(analyze(text).status, Status::Incomplete, "{text}");
    }
    for text in ["\"\"", "title:\" \"", "year:\"abc\"", "year:abc title:\""] {
        assert_eq!(analyze(text).status, Status::Invalid, "{text}");
    }
}

#[test]
fn scoped_prefixes_roundtrip_without_becoming_scalar_values() {
    for text in [
        "title:(author:Smith)",
        "title:(NOT graph)",
        "tag:(-read)",
        "title:(author:(NOT Smith))",
    ] {
        let original = analyze(text).document.unwrap();
        let rendered = serialize(&original).unwrap();
        let reparsed = analyze(&rendered);
        assert_eq!(reparsed.status, Status::Valid, "{text} -> {rendered}");
        let shape = |d: Document| {
            d.nodes
                .into_iter()
                .map(|n| (n.kind, n.value, n.children))
                .collect::<Vec<_>>()
        };
        assert_eq!(
            shape(original),
            shape(reparsed.document.unwrap()),
            "{text} -> {rendered}"
        );
    }
}

#[test]
fn minus_escape_is_structural_only_at_the_start_of_a_value() {
    for (text, value) in [
        (r"\-graph", "-graph"),
        (r"a\-b", r"a\-b"),
        (r"title:a\-b", r"a\-b"),
    ] {
        let doc = analyze(text).document.unwrap();
        assert_eq!(
            doc.nodes
                .iter()
                .find(|n| n.kind == NodeKind::Word)
                .unwrap()
                .value,
            value
        );
        let again = analyze(&serialize(&doc).unwrap()).document.unwrap();
        assert_eq!(
            again
                .nodes
                .iter()
                .find(|n| n.kind == NodeKind::Word)
                .unwrap()
                .value,
            value
        );
    }
}

#[test]
fn composite_ranges_cover_complete_conditions_without_expanding_leaf_tokens() {
    for text in [
        "tag:read",
        "-tag:read",
        "title:(graph OR neural)",
        "NOT (tag:已读)",
        "title:((graph))",
    ] {
        let doc = analyze(text).document.unwrap();
        let root = &doc.nodes[doc.root.unwrap()];
        assert_eq!(root.span, 0..text.len(), "{text}");
        for node in &doc.nodes {
            if matches!(node.kind, NodeKind::Word | NodeKind::Phrase) {
                assert_eq!(&text[node.span.clone()], node.value, "{text}");
            }
        }
    }
    let text = "graph (tag:已读) author:(Knuth Smith)";
    let doc = analyze(text).document.unwrap();
    let tag = doc
        .nodes
        .iter()
        .find(|n| n.kind == NodeKind::Scope && n.value == "tag")
        .unwrap();
    assert_eq!(&text[tag.span.clone()], "(tag:已读)");
    assert_eq!(doc.nodes[doc.root.unwrap()].span, 0..text.len());
}
