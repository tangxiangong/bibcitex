use crate::core::search::query::{self, Document, Node, NodeKind};
use crate::*;
use std::{collections::HashMap, fs};
fn diagnostic(d: query::Diagnostic) -> QueryDiagnostic {
    QueryDiagnostic {
        code: d.code,
        start: d.span.start as u32,
        end: d.span.end as u32,
        warning: false,
    }
}
fn node_kind(kind: QueryNodeKind) -> NodeKind {
    match kind {
        QueryNodeKind::Word => NodeKind::Word,
        QueryNodeKind::Phrase => NodeKind::Phrase,
        QueryNodeKind::And => NodeKind::And,
        QueryNodeKind::Or => NodeKind::Or,
        QueryNodeKind::Not => NodeKind::Not,
        QueryNodeKind::Scope => NodeKind::Scope,
    }
}
fn record_kind(kind: NodeKind) -> QueryNodeKind {
    match kind {
        NodeKind::Word => QueryNodeKind::Word,
        NodeKind::Phrase => QueryNodeKind::Phrase,
        NodeKind::And => QueryNodeKind::And,
        NodeKind::Or => QueryNodeKind::Or,
        NodeKind::Not => QueryNodeKind::Not,
        NodeKind::Scope => QueryNodeKind::Scope,
    }
}
fn document(record: QueryDocumentRecord) -> Document {
    Document {
        root: record.root.map(|n| n as usize),
        nodes: record
            .nodes
            .into_iter()
            .map(|n| Node {
                kind: node_kind(n.kind),
                value: n.text_value,
                tag_id: n.tag_id,
                children: n.children.into_iter().map(|id| id as usize).collect(),
                span: n.start as usize..n.end as usize,
            })
            .collect(),
    }
}
fn record(document: Document) -> QueryDocumentRecord {
    QueryDocumentRecord {
        root: document.root.map(|n| n as u32),
        nodes: document
            .nodes
            .into_iter()
            .map(|n| QueryNodeRecord {
                kind: record_kind(n.kind),
                text_value: n.value,
                tag_id: n.tag_id,
                children: n.children.into_iter().map(|id| id as u32).collect(),
                start: n.span.start as u32,
                end: n.span.end as u32,
            })
            .collect(),
    }
}
fn error(code: &str) -> QueryAnalysis {
    QueryAnalysis {
        status: QueryStatus::Invalid,
        diagnostics: vec![QueryDiagnostic {
            code: code.into(),
            start: 0,
            end: 0,
            warning: false,
        }],
        document: None,
        text: String::new(),
        tag_revision: None,
    }
}
/// Analyze all input with chumsky. Invalid or incomplete queries return structured diagnostics and no executable tree.
pub fn analyze_query(text: String) -> QueryAnalysis {
    let analysis = query::analyze(&text);
    QueryAnalysis {
        status: match analysis.status {
            query::Status::Valid => QueryStatus::Valid,
            query::Status::Incomplete => QueryStatus::Incomplete,
            query::Status::Invalid => QueryStatus::Invalid,
        },
        diagnostics: analysis.diagnostics.into_iter().map(diagnostic).collect(),
        document: analysis.document.map(record),
        text,
        tag_revision: None,
    }
}
fn leaves(doc: &Document) -> Vec<usize> {
    let mut result = Vec::new();
    let mut stack: Vec<_> = doc.root.into_iter().collect();
    while let Some(id) = stack.pop() {
        let n = &doc.nodes[id];
        if matches!(n.kind, NodeKind::Word | NodeKind::Phrase) {
            result.push(id);
        }
        stack.extend(n.children.iter().rev());
    }
    result
}
fn rendered(mut doc: Document) -> QueryAnalysis {
    for node in &mut doc.nodes {
        if node.kind == NodeKind::Scope
            && let Some(field) = query::canonical_field(&node.value)
        {
            node.value = field;
        }
    }
    if let Err(e) = query::validate_document(&doc) {
        return error(&e.code);
    }
    let mut stack: Vec<_> = doc.root.map(|id| (id, String::new())).into_iter().collect();
    while let Some((id, scope)) = stack.pop() {
        let node = &doc.nodes[id];
        let scope = if node.kind == NodeKind::Scope {
            node.value.clone()
        } else {
            scope
        };
        if node.tag_id.is_some()
            && (scope != "tag" || !matches!(node.kind, NodeKind::Word | NodeKind::Phrase))
        {
            return error("InvalidTagBinding");
        }
        stack.extend(node.children.iter().map(|&id| (id, scope.clone())));
    }
    let text = match query::serialize(&doc) {
        Ok(t) => t,
        Err(e) => {
            return QueryAnalysis {
                diagnostics: vec![diagnostic(e)],
                ..error("InvalidTree")
            };
        }
    };
    let mut analyzed = analyze_query(text);
    // Canonical parsing restores real source spans. Bindings follow corresponding
    // leaves in traversal order; they never resolve through the text projection.
    if let Some(ref mut output) = analyzed.document {
        let mut normalized = document(output.clone());
        let before = leaves(&doc);
        let after = leaves(&normalized);
        if before.len() != after.len() {
            return error("ProjectionMismatch");
        }
        for (a, b) in before.into_iter().zip(after) {
            normalized.nodes[b].tag_id = doc.nodes[a].tag_id.clone();
        }
        *output = record(normalized);
    }
    analyzed
}
/// Validate a bound tree and serialize it canonically while retaining tag IDs. Rejects every source and arena budget violation.
pub fn render_query(document: QueryDocumentRecord) -> QueryAnalysis {
    rendered(self::document(document))
}
/// Return canonical supported fields and bibliography types for all platform editors.
pub fn search_catalog() -> SearchCatalog {
    SearchCatalog {
        field_descriptors: [
            ("title", "标题", "text"),
            ("author", "作者", "text"),
            ("editor", "编者", "text"),
            ("journal", "期刊", "text"),
            ("booktitle", "书名", "text"),
            ("abstract", "摘要", "text"),
            ("note", "备注", "text"),
            ("publisher", "出版社", "text"),
            ("school", "学校", "text"),
            ("institution", "机构", "text"),
            ("organization", "组织", "text"),
            ("key", "引用键", "identifier"),
            ("doi", "DOI", "identifier"),
            ("isbn", "ISBN", "identifier"),
            ("year", "年份", "year"),
            ("type", "类型", "type"),
            ("tag", "标签", "tag"),
        ]
        .into_iter()
        .map(|(name, label, mode)| SearchFieldDescriptor {
            name: name.into(),
            label_key: label.into(),
            match_mode: mode.into(),
            supports_phrase: mode == "text",
        })
        .collect(),
        type_descriptors: [
            ("article", "期刊论文"),
            ("book", "书籍"),
            ("thesis", "学位论文"),
            ("mastersthesis", "硕士论文"),
            ("phdthesis", "博士论文"),
            ("booklet", "小册子"),
            ("inbook", "书籍章节"),
            ("incollection", "文集章节"),
            ("inproceedings", "会议论文"),
            ("misc", "其他"),
            ("techreport", "技术报告"),
        ]
        .into_iter()
        .map(|(name, label)| SearchTypeDescriptor {
            name: name.into(),
            label_key: label.into(),
        })
        .collect(),
        fields: query::FIELDS.iter().map(|s| s.to_string()).collect(),
        types: query::TYPES.iter().map(|s| s.to_string()).collect(),
    }
}
/// Build a validated condition. Ordinary multiword values become AND terms; phrases and identifiers retain their field semantics.
pub fn build_condition(condition: QueryCondition) -> QueryAnalysis {
    if !condition.field_name.is_empty() && !query::FIELDS.contains(&condition.field_name.as_str()) {
        return error("UnknownField");
    }
    if condition.tag_id.is_some() && condition.field_name != "tag" {
        return error("InvalidTagBinding");
    }
    let values: Vec<_> = if !condition.phrase
        && !matches!(
            condition.field_name.as_str(),
            "tag" | "doi" | "isbn" | "key" | "year" | "type"
        ) {
        condition
            .text_value
            .split_whitespace()
            .map(str::to_owned)
            .collect()
    } else {
        vec![condition.text_value]
    };
    let multiple = values.len() > 1;
    let mut doc = Document::default();
    for value in values {
        doc.nodes.push(Node {
            kind: if condition.phrase {
                NodeKind::Phrase
            } else {
                NodeKind::Word
            },
            value,
            tag_id: condition.tag_id.clone(),
            children: vec![],
            span: 0..0,
        });
    }
    if doc.nodes.is_empty() {
        return error("EmptyValue");
    }
    if condition.exclude && doc.nodes.len() > 1 {
        let count = doc.nodes.len();
        for id in 0..count {
            doc.nodes.push(Node {
                kind: NodeKind::Not,
                value: String::new(),
                children: vec![id],
                tag_id: None,
                span: 0..0,
            });
        }
        let children = (count..doc.nodes.len()).collect();
        doc.nodes.push(Node {
            kind: NodeKind::And,
            value: String::new(),
            children,
            tag_id: None,
            span: 0..0,
        });
    } else if doc.nodes.len() > 1 {
        let children = (0..doc.nodes.len()).collect();
        doc.nodes.push(Node {
            kind: NodeKind::And,
            value: String::new(),
            children,
            tag_id: None,
            span: 0..0,
        });
    }
    let mut root = doc.nodes.len() - 1;
    if !condition.field_name.is_empty() {
        doc.nodes.push(Node {
            kind: NodeKind::Scope,
            value: condition.field_name,
            children: vec![root],
            tag_id: None,
            span: 0..0,
        });
        root = doc.nodes.len() - 1;
    }
    if condition.exclude && !multiple {
        doc.nodes.push(Node {
            kind: NodeKind::Not,
            value: String::new(),
            children: vec![root],
            tag_id: None,
            span: 0..0,
        });
        root = doc.nodes.len() - 1;
    }
    doc.root = Some(root);
    rendered(doc)
}
/// Apply an atomic structural edit and canonicalize the result. Invalid targets, operands, or resource limits return Invalid.
pub fn edit_query(edit: QueryEdit) -> QueryAnalysis {
    let mut doc = document(edit.document);
    if let Err(e) = query::validate_document(&doc) {
        return error(&e.code);
    }
    let target = edit.target.map(|n| n as usize).or(doc.root);
    if target.is_some_and(|n| n >= doc.nodes.len()) {
        return error("InvalidTarget");
    }
    let mut operand = edit.operand.map(document);
    if let Some(ref operand) = operand
        && let Err(e) = query::validate_document(operand)
    {
        return error(&e.code);
    }
    let operand_root = operand
        .as_ref()
        .and_then(|d| d.root)
        .map(|r| r + doc.nodes.len());
    if let Some(ref mut operand) = operand {
        let offset = doc.nodes.len();
        for node in &mut operand.nodes {
            for child in &mut node.children {
                *child += offset;
            }
        }
        doc.nodes.append(&mut operand.nodes);
    }
    let replacement = match edit.action {
        QueryEditAction::Remove => None,
        QueryEditAction::Replace => {
            if operand_root.is_none() {
                return error("MissingOperand");
            }
            operand_root
        }
        QueryEditAction::Negate => {
            let Some(t) = target else {
                return rendered(doc);
            };
            doc.nodes.push(Node {
                kind: NodeKind::Not,
                value: String::new(),
                children: vec![t],
                tag_id: None,
                span: 0..0,
            });
            Some(doc.nodes.len() - 1)
        }
        QueryEditAction::AppendAnd | QueryEditAction::AppendOr => {
            let Some(o) = operand_root else {
                return error("MissingOperand");
            };
            if let Some(t) = target {
                doc.nodes.push(Node {
                    kind: if edit.action == QueryEditAction::AppendAnd {
                        NodeKind::And
                    } else {
                        NodeKind::Or
                    },
                    value: String::new(),
                    children: vec![t, o],
                    tag_id: None,
                    span: 0..0,
                });
                Some(doc.nodes.len() - 1)
            } else {
                Some(o)
            }
        }
    };
    if let Some(t) = target {
        // Replace the original parent edge, never the new wrapper's edge.
        let original_len = edit_document_length(&doc, operand_root, replacement, edit.action);
        for (id, node) in doc.nodes.iter_mut().enumerate().take(original_len) {
            if Some(id) == replacement {
                continue;
            }
            node.children = node
                .children
                .iter()
                .filter_map(|&c| if c == t { replacement } else { Some(c) })
                .collect();
        }
        if doc.root == Some(t) {
            doc.root = replacement;
        }
    } else {
        doc.root = replacement;
    }
    compact(&mut doc);
    rendered(doc)
}
fn edit_document_length(
    doc: &Document,
    operand: Option<usize>,
    replacement: Option<usize>,
    action: QueryEditAction,
) -> usize {
    operand.unwrap_or_else(|| {
        if matches!(action, QueryEditAction::Negate) {
            replacement.unwrap_or(doc.nodes.len())
        } else {
            doc.nodes.len()
        }
    })
}
fn compact(doc: &mut Document) {
    loop {
        let empty: Vec<_> = doc
            .nodes
            .iter()
            .enumerate()
            .filter(|(_, n)| {
                !matches!(n.kind, NodeKind::Word | NodeKind::Phrase) && n.children.is_empty()
            })
            .map(|(i, _)| i)
            .collect();
        let mut changed = false;
        for n in &mut doc.nodes {
            let before = n.children.len();
            n.children.retain(|i| !empty.contains(i));
            changed |= before != n.children.len();
        }
        if doc.root.is_some_and(|r| empty.contains(&r)) {
            doc.root = None;
        }
        if !changed {
            break;
        }
    }
    let mut order = Vec::new();
    let mut stack: Vec<_> = doc.root.into_iter().collect();
    while let Some(id) = stack.pop() {
        order.push(id);
        stack.extend(doc.nodes[id].children.iter().rev());
    }
    let map: HashMap<_, _> = order
        .iter()
        .enumerate()
        .map(|(new, &old)| (old, new))
        .collect();
    let nodes = order
        .iter()
        .map(|&i| {
            let mut n = doc.nodes[i].clone();
            n.children = n.children.iter().map(|c| map[c]).collect();
            n
        })
        .collect();
    doc.root = doc.root.map(|i| map[&i]);
    doc.nodes = nodes;
}
/// Apply a native UTF-8 replacement and preserve untouched tag identities by exact source spans. Whole replacements deliberately discard bindings.
pub fn edit_query_text(edit: QueryTextEdit) -> QueryAnalysis {
    let start = edit.start as usize;
    let end = edit.end as usize;
    if start > end || !edit.text.is_char_boundary(start) || !edit.text.is_char_boundary(end) {
        return error("InvalidEditRange");
    }
    let mut text = edit.text.clone();
    text.replace_range(start..end, &edit.replacement);
    let mut analyzed = analyze_query(text);
    if edit.full_replace {
        return analyzed;
    }
    let old = document(edit.document);
    if query::validate_document(&old).is_err() {
        return error("InvalidTree");
    }
    if analyzed.document.is_none() {
        let delta = edit.replacement.len() as isize - (end - start) as isize;
        let mut anchors = old.clone();
        for node in &mut anchors.nodes {
            if node.span.end <= start {
            } else if node.span.start >= end {
                node.span = node.span.start.saturating_add_signed(delta)
                    ..node.span.end.saturating_add_signed(delta);
            } else {
                node.tag_id = None;
            }
        }
        analyzed.document = Some(record(anchors));
        return analyzed;
    }
    if let Some(output) = &mut analyzed.document {
        let mut new = document(output.clone());
        let mut tag_leaves = std::collections::HashSet::new();
        let mut stack: Vec<_> = new.root.map(|id| (id, String::new())).into_iter().collect();
        while let Some((id, scope)) = stack.pop() {
            let n = &new.nodes[id];
            let scope = if n.kind == NodeKind::Scope {
                n.value.clone()
            } else {
                scope
            };
            if scope == "tag" && matches!(n.kind, NodeKind::Word | NodeKind::Phrase) {
                tag_leaves.insert(id);
            }
            stack.extend(n.children.iter().map(|&c| (c, scope.clone())));
        }
        let delta = edit.replacement.len() as isize - (end - start) as isize;
        for old_id in leaves(&old) {
            let node = &old.nodes[old_id];
            if node.tag_id.is_none() {
                continue;
            }
            let range = if node.span.end <= start {
                Some(node.span.clone())
            } else if node.span.start >= end {
                Some(
                    node.span.start.saturating_add_signed(delta)
                        ..node.span.end.saturating_add_signed(delta),
                )
            } else {
                None
            };
            if let Some(range) = range
                && let Some((_, n)) = new.nodes.iter_mut().enumerate().find(|(id, n)| {
                    tag_leaves.contains(id)
                        && n.span == range
                        && n.kind == node.kind
                        && n.value == node.value
                })
            {
                n.tag_id = node.tag_id.clone();
            }
        }
        *output = record(new);
    }
    analyzed
}
/// Refresh selected tag names and explicit merge redirects without rebinding by name. Storage failures return CoreError; deleted IDs return Invalid.
pub fn refresh_query(doc: QueryDocumentRecord) -> Result<QueryAnalysis, CoreError> {
    let mut doc = document(doc);
    query::validate_document(&doc).map_err(|e| CoreError::from(e.code))?;
    let _lock = crate::registry::search_lock()?;
    let metadata = crate::tags::read()?;
    for node in &mut doc.nodes {
        if let Some(id) = &node.tag_id {
            let Some(tag) = metadata.redirected(id) else {
                return Ok(error("MissingTagBinding"));
            };
            node.tag_id = Some(tag.id.clone());
            node.value = tag.name.clone();
        }
    }
    let mut result = rendered(doc);
    result.tag_revision = Some(metadata.revision);
    Ok(result)
}
/// Read a consistent tag catalog and exact-key assignments. A nonempty path reconciles current records; an empty path reads the global catalog.
pub fn tag_snapshot(path: String, cite_key: String) -> Result<TagSnapshot, CoreError> {
    if !path.is_empty() {
        let source = crate::api::references(&path)?;
        let canonical = fs::canonicalize(&path).map_err(|e| CoreError::from(e.to_string()))?;
        crate::tags::load_snapshot(
            &canonical.to_string_lossy(),
            &source.version,
            &source.references,
        )?;
    }
    crate::tags::snapshot(path, cite_key).map_err(Into::into)
}
/// Read assignments by stable library and record identities, resolving paths only for IO.
pub fn tag_snapshot_uid(library_uid: String, record_uid: String) -> Result<TagSnapshot, CoreError> {
    crate::registry::recover_search_transaction()?;
    let path = crate::registry::path_for_uid(&library_uid)?;
    let source = crate::api::references(&path)?;
    crate::tags::load_snapshot_uid(&library_uid, &path, &source.version, &source.references)?;
    crate::tags::snapshot_uid(&library_uid, &record_uid).map_err(Into::into)
}
/// Apply one revision-checked tag or association mutation atomically. Conflicts and storage failures leave existing metadata intact.
pub fn mutate_tag(mutation: TagMutation) -> Result<TagSnapshot, CoreError> {
    crate::registry::recover_search_transaction()?;
    if matches!(
        mutation.action,
        TagAction::Assign | TagAction::Unassign | TagAction::Relink
    ) || (mutation.action == TagAction::Create && !mutation.record_uid.is_empty())
    {
        let path = crate::registry::path_for_uid(&mutation.library_uid)?;
        let source = crate::api::references(&path)?;
        crate::tags::load_snapshot_uid(
            &mutation.library_uid,
            &path,
            &source.version,
            &source.references,
        )?;
    }
    crate::tags::mutate(mutation).map_err(Into::into)
}
/// Return whether a result still matches current file content and metadata revision. I/O failures return CoreError and never authorize an action.
pub fn validate_search_context(context: SearchContext) -> Result<bool, CoreError> {
    if crate::api::file_version(&context.path)? != context.file_version {
        return Ok(false);
    }
    if let Some(revision) = context.tag_revision {
        let _lock = crate::registry::search_lock()?;
        if crate::tags::read()?.revision != revision {
            return Ok(false);
        }
    }
    Ok(true)
}
/// Execute an explicit-mode request against immutable snapshots. Invalid, incomplete, and stale queries return no actionable records; I/O failures return CoreError.
pub fn advanced_search(
    request: AdvancedSearchRequest,
) -> Result<AdvancedSearchResponse, CoreError> {
    if !matches!(
        request.default_field.as_str(),
        "all" | "title" | "author" | "journal" | "year"
    ) {
        return Err("UnknownSearchField".to_string().into());
    }
    let mut analysis = match request.input_kind {
        QueryInputKind::Text => analyze_query(request.text.clone()),
        QueryInputKind::Bound => {
            if request.mode != SearchMode::Advanced {
                return Err("BoundQueryRequiresAdvancedMode".to_string().into());
            }
            let Some(doc) = request.document.clone() else {
                return Err("MissingQueryDocument".to_string().into());
            };
            let mut result = render_query(doc.clone());
            if request.text.len() > query::MAX_BYTES {
                return Err("ByteLimitExceeded".to_string().into());
            }
            let projected = analyze_query(request.text.clone());
            if projected.status != QueryStatus::Valid {
                return Ok(AdvancedSearchResponse {
                    analysis: projected,
                    references: vec![],
                    evidence: vec![],
                    context: None,
                });
            }
            let canonical = projected.document.clone().map(render_query);
            if result.status == QueryStatus::Valid
                && canonical.as_ref().is_none_or(|p| p.text != result.text)
            {
                return Err("ProjectionMismatch".to_string().into());
            }
            if result.status == QueryStatus::Valid {
                let mut projection = document(
                    projected
                        .document
                        .ok_or_else(|| CoreError::from("ProjectionMismatch".to_string()))?,
                );
                let bindings = document(
                    result
                        .document
                        .take()
                        .ok_or_else(|| CoreError::from("ProjectionMismatch".to_string()))?,
                );
                let source_leaves = leaves(&bindings);
                let target_leaves = leaves(&projection);
                if source_leaves.len() != target_leaves.len() {
                    return Err("ProjectionMismatch".to_string().into());
                }
                for (source, target) in source_leaves.into_iter().zip(target_leaves) {
                    projection.nodes[target].tag_id = bindings.nodes[source].tag_id.clone();
                }
                result.text = request.text.clone();
                result.document = Some(record(projection));
            }
            result
        }
    };
    if request.mode == SearchMode::Ordinary {
        analysis = QueryAnalysis {
            status: QueryStatus::Valid,
            diagnostics: vec![],
            document: None,
            text: request.text.clone(),
            tag_revision: None,
        };
    }
    let mut response = AdvancedSearchResponse {
        analysis: analysis.clone(),
        references: vec![],
        evidence: vec![],
        context: None,
    };
    if analysis.status != QueryStatus::Valid {
        return Ok(response);
    }
    let source = crate::api::references(&request.path)?;
    let path = fs::canonicalize(&request.path)
        .map_err(|e| CoreError::from(e.to_string()))?
        .to_string_lossy()
        .into_owned();
    let doc = analysis.document.clone().map(document).unwrap_or_default();
    let has_ids = doc.nodes.iter().any(|n| n.tag_id.is_some());
    if has_ids
        && (request.input_kind != QueryInputKind::Bound || request.expected_tag_revision.is_none())
    {
        return Err("MissingExpectedTagRevision".to_string().into());
    }
    let metadata = Some(crate::tags::load_snapshot(
        &path,
        &source.version,
        &source.references,
    )?);
    if let Some(metadata) = &metadata {
        analysis.tag_revision = Some(metadata.revision);
        if request
            .expected_tag_revision
            .is_some_and(|r| r != metadata.revision)
        {
            analysis.status = QueryStatus::StaleQuery;
            response.analysis = analysis;
            return Ok(response);
        }
        for node in &doc.nodes {
            if let Some(id) = &node.tag_id
                && metadata.resolve(id).is_none()
            {
                response.analysis = error("MissingTagBinding");
                return Ok(response);
            }
        }
    }
    if let Some(metadata) = &metadata {
        let mut stack: Vec<_> = doc.root.map(|id| (id, String::new())).into_iter().collect();
        while let Some((id, scope)) = stack.pop() {
            let node = &doc.nodes[id];
            let scope = if node.kind == NodeKind::Scope {
                node.value.clone()
            } else {
                scope
            };
            if scope == "tag"
                && matches!(node.kind, NodeKind::Word | NodeKind::Phrase)
                && node.tag_id.is_none()
                && !metadata.tags.values().any(|t| {
                    crate::tags::comparison(&t.name) == crate::tags::comparison(&node.value)
                })
            {
                analysis.diagnostics.push(QueryDiagnostic {
                    code: "UnknownTag".into(),
                    start: node.span.start as u32,
                    end: node.span.end as u32,
                    warning: true,
                });
            }
            stack.extend(node.children.iter().map(|&id| (id, scope.clone())));
        }
        if analysis.diagnostics.len() > 32 {
            analysis.diagnostics.truncate(31);
            analysis.diagnostics.push(QueryDiagnostic {
                code: "TooManyDiagnostics".into(),
                start: 0,
                end: 0,
                warning: false,
            });
            analysis.status = QueryStatus::Invalid;
            response.analysis = analysis;
            return Ok(response);
        }
    }
    // Resolve names and exact-key associations once per snapshot, not once per
    // candidate. A tag query over a large library must not scan all records for
    // every result row.
    let tag_names: HashMap<String, String> = metadata
        .as_ref()
        .map(|m| {
            m.tags
                .values()
                .map(|t| (crate::tags::comparison(&t.name), t.id.clone()))
                .collect()
        })
        .unwrap_or_default();
    let query_tags: HashMap<&str, Option<&String>> = doc
        .nodes
        .iter()
        .filter(|n| matches!(n.kind, NodeKind::Word | NodeKind::Phrase))
        .map(|n| {
            (
                n.value.as_str(),
                tag_names.get(&crate::tags::comparison(&n.value)),
            )
        })
        .collect();
    let associations: HashMap<&str, &std::collections::BTreeSet<String>> = metadata
        .as_ref()
        .and_then(|m| m.source(&path))
        .map(|s| {
            s.records
                .values()
                .filter(|r| r.present)
                .map(|r| (r.key.as_str(), &r.tags))
                .collect()
        })
        .unwrap_or_default();
    let type_filter = crate::api::search_type(&request.type_filter)?;
    if request.mode == SearchMode::Ordinary {
        response.references = crate::search(
            request.path.clone(),
            request.text,
            request.default_field,
            request.type_filter,
        )?;
    } else {
        let index = source.advanced();
        crate::api::account_advanced_index()?;
        let hits = index
            .search(
                &doc,
                &request.default_field,
                |i| {
                    type_filter.as_ref().is_none_or(|filter| {
                        let kind = &source.references[i].type_;
                        kind == filter
                            || (*filter == biblatex::EntryType::Thesis
                                && matches!(
                                    kind,
                                    biblatex::EntryType::MastersThesis
                                        | biblatex::EntryType::PhdThesis
                                ))
                    })
                },
                |i, name, id| {
                    let tag =
                        id.or_else(|| query_tags.get(name).copied().flatten().map(String::as_str));
                    tag.is_some_and(|id| {
                        associations
                            .get(source.references[i].cite_key.as_str())
                            .is_some_and(|tags| tags.contains(id))
                    })
                },
            )
            .map_err(|e| CoreError::from(e.code))?;
        for hit in hits {
            let reference =
                ReferenceRecord::from_reference(&source.references[hit.index], &request.path)?;
            response.evidence.push(SearchEvidence {
                ranges: hit
                    .ranges
                    .into_iter()
                    .map(|r| SearchMatchRange {
                        field_name: r.field_name,
                        entry_index: r.entry_index as u32,
                        start: r.start as u32,
                        end: r.end as u32,
                    })
                    .collect(),
                reference_id: reference.id.clone(),
                fields: hit.fields,
                fuzzy: hit.fuzzy,
                fuzzy_terms: hit.fuzzy_terms,
            });
            response.references.push(reference);
        }
    }
    if let Some(metadata) = &metadata {
        crate::tags::decorate(metadata, &path, &mut response.references)?;
        for (evidence, reference) in response.evidence.iter_mut().zip(&response.references) {
            evidence.reference_id = reference.id.clone();
        }
    }
    let context = SearchContext {
        path,
        file_version: source.version.clone(),
        tag_revision: metadata.map(|m| m.revision),
        query_version: request.query_version,
    };
    if !validate_search_context(context.clone())? {
        analysis.status = QueryStatus::StaleQuery;
        response.references.clear();
        response.evidence.clear();
    } else {
        response.context = Some(context);
    }
    response.analysis = analysis;
    Ok(response)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    fn library() -> tempfile::NamedTempFile {
        let mut file = tempfile::NamedTempFile::new().unwrap();
        write!(file,"@article{{A,title={{network}},year={{2020}}}}\n@article{{B,title={{netwrok}}}}\n@book{{C,title={{other}}}}").unwrap();
        file
    }
    fn request(path: &str, text: &str) -> AdvancedSearchRequest {
        AdvancedSearchRequest {
            path: path.into(),
            mode: SearchMode::Advanced,
            input_kind: QueryInputKind::Text,
            text: text.into(),
            document: None,
            expected_tag_revision: None,
            default_field: "all".into(),
            type_filter: "all".into(),
            query_version: 7,
        }
    }
    fn mutation(action: TagAction, id: &str, name: &str, path: &str, revision: u64) -> TagMutation {
        let snapshot = if path.is_empty() {
            None
        } else {
            Some(tag_snapshot(path.into(), "A".into()).unwrap())
        };
        TagMutation {
            library_uid: snapshot
                .as_ref()
                .map(|s| s.library_uid.clone())
                .unwrap_or_default(),
            record_uid: snapshot
                .as_ref()
                .map(|s| s.record_uid.clone())
                .unwrap_or_default(),
            color: None,
            action,
            id: id.into(),
            name: name.into(),
            target_id: String::new(),
            path: path.into(),
            cite_key: "A".into(),
            expected_revision: revision,
        }
    }
    #[test]
    fn bound_negative_tag_rename_never_expands_to_name_complement() {
        let file = library();
        let path = file.path().to_str().unwrap();
        let snapshot = tag_snapshot(path.into(), "A".into()).unwrap();
        let snapshot = mutate_tag(mutation(
            TagAction::Create,
            "",
            "已读",
            path,
            snapshot.revision,
        ))
        .unwrap();
        let id = snapshot.tags[0].id.clone();
        let snapshot = mutate_tag(mutation(
            TagAction::Assign,
            &id,
            "",
            path,
            snapshot.revision,
        ))
        .unwrap();
        let condition = build_condition(QueryCondition {
            field_name: "tag".into(),
            text_value: "已读".into(),
            phrase: false,
            exclude: true,
            tag_id: Some(id.clone()),
        });
        assert_eq!(condition.status, QueryStatus::Valid);
        let mut req = request(path, &condition.text);
        req.input_kind = QueryInputKind::Bound;
        req.document = condition.document.clone();
        req.expected_tag_revision = Some(snapshot.revision);
        assert_eq!(advanced_search(req.clone()).unwrap().references.len(), 2);
        let snapshot = mutate_tag(mutation(
            TagAction::Rename,
            &id,
            "已完成",
            path,
            snapshot.revision,
        ))
        .unwrap();
        let stale = advanced_search(req.clone()).unwrap();
        assert_eq!(stale.analysis.status, QueryStatus::StaleQuery);
        assert!(stale.references.is_empty());
        assert!(stale.context.is_none());
        let fresh = refresh_query(condition.document.unwrap()).unwrap();
        req.text = fresh.text;
        req.document = fresh.document;
        req.expected_tag_revision = fresh.tag_revision;
        assert_eq!(advanced_search(req.clone()).unwrap().references.len(), 2);
        let snapshot = mutate_tag(mutation(
            TagAction::Delete,
            &id,
            "",
            path,
            snapshot.revision,
        ))
        .unwrap();
        let snapshot = mutate_tag(mutation(
            TagAction::Create,
            "",
            "已完成",
            path,
            snapshot.revision,
        ))
        .unwrap();
        req.expected_tag_revision = Some(snapshot.revision);
        let response = advanced_search(req).unwrap();
        assert_eq!(response.analysis.status, QueryStatus::Invalid);
        assert_eq!(response.analysis.diagnostics[0].code, "MissingTagBinding");
    }
    #[test]
    fn bound_query_requires_revision_and_structural_validation() {
        let file = library();
        let path = file.path().to_str().unwrap();
        let condition = build_condition(QueryCondition {
            field_name: "tag".into(),
            text_value: "machine learning".into(),
            phrase: false,
            exclude: false,
            tag_id: Some("missing".into()),
        });
        assert_eq!(condition.status, QueryStatus::Valid);
        let mut req = request(path, &condition.text);
        req.input_kind = QueryInputKind::Bound;
        req.document = condition.document;
        assert!(
            advanced_search(req)
                .unwrap_err()
                .to_string()
                .contains("MissingExpectedTagRevision")
        );
        let mut doc = analyze_query("x".into()).document.unwrap();
        doc.nodes[0].children.push(0);
        assert_eq!(render_query(doc).status, QueryStatus::Invalid);
    }
    #[test]
    fn semantic_editor_preserves_groups_and_removes_empty_negations() {
        let left = analyze_query("title:graph".into()).document.unwrap();
        let right = analyze_query("author:Smith".into()).document.unwrap();
        let combined = edit_query(QueryEdit {
            document: left,
            action: QueryEditAction::AppendOr,
            target: None,
            operand: Some(right),
        });
        assert_eq!(
            combined.status,
            QueryStatus::Valid,
            "{:?}",
            combined.diagnostics
        );
        assert!(combined.text.contains(" OR "));
        let negated = edit_query(QueryEdit {
            document: combined.document.unwrap(),
            action: QueryEditAction::Negate,
            target: None,
            operand: None,
        });
        assert_eq!(negated.status, QueryStatus::Valid);
        let removed = edit_query(QueryEdit {
            document: negated.document.unwrap(),
            action: QueryEditAction::Remove,
            target: None,
            operand: None,
        });
        assert_eq!(removed.text, "");
        assert_eq!(removed.status, QueryStatus::Valid);
        let excluded = build_condition(QueryCondition {
            field_name: "title".into(),
            text_value: "graph neural".into(),
            phrase: false,
            exclude: true,
            tag_id: None,
        });
        assert!(
            excluded.text.contains("NOT graph AND NOT neural"),
            "{}",
            excluded.text
        );
    }
    #[test]
    fn local_input_edits_preserve_only_untouched_tag_bindings() {
        let bound = build_condition(QueryCondition {
            field_name: "tag".into(),
            text_value: "read".into(),
            phrase: false,
            exclude: false,
            tag_id: Some("stable".into()),
        });
        let text = bound.text.clone();
        let end = text.len() as u32;
        let edited = edit_query_text(QueryTextEdit {
            document: bound.document.clone().unwrap(),
            text: text.clone(),
            start: end,
            end,
            replacement: " graph".into(),
            full_replace: false,
        });
        assert_eq!(edited.status, QueryStatus::Valid);
        assert!(
            edited
                .document
                .unwrap()
                .nodes
                .iter()
                .any(|n| n.tag_id.as_deref() == Some("stable"))
        );
        let replaced = edit_query_text(QueryTextEdit {
            document: bound.document.unwrap(),
            text: text.clone(),
            start: 0,
            end,
            replacement: text,
            full_replace: true,
        });
        assert!(
            replaced
                .document
                .unwrap()
                .nodes
                .iter()
                .all(|n| n.tag_id.is_none())
        );
    }
    #[test]
    fn ordinary_mode_does_not_interpret_advanced_syntax_and_context_rejects_rewrites() {
        let file = library();
        let path = file.path().to_str().unwrap();
        let result = advanced_search(request(path, "network OR NOT network")).unwrap();
        assert_eq!(result.references.len(), 3);
        let context = result.context.unwrap();
        assert!(validate_search_context(context.clone()).unwrap());
        let before = fs::read_to_string(path).unwrap();
        fs::write(path, before.replace("network", "changed")).unwrap();
        assert!(!validate_search_context(context).unwrap());
        let mut req = request(path, "type:book");
        req.mode = SearchMode::Ordinary;
        assert!(advanced_search(req).unwrap().references.is_empty());
    }
}

#[cfg(test)]
mod binding_edit_regressions {
    use super::*;
    #[test]
    fn incomplete_edit_retains_other_bindings_but_field_replacement_removes_them() {
        let start = build_condition(QueryCondition {
            field_name: "tag".into(),
            text_value: "read".into(),
            phrase: false,
            exclude: false,
            tag_id: Some("id".into()),
        });
        let end = start.text.len() as u32;
        let partial = edit_query_text(QueryTextEdit {
            document: start.document.clone().unwrap(),
            text: start.text.clone(),
            start: end,
            end,
            replacement: " author:".into(),
            full_replace: false,
        });
        assert_eq!(partial.status, QueryStatus::Incomplete);
        assert!(
            partial
                .document
                .as_ref()
                .unwrap()
                .nodes
                .iter()
                .any(|n| n.tag_id.is_some())
        );
        let end = partial.text.len() as u32;
        let complete = edit_query_text(QueryTextEdit {
            document: partial.document.unwrap(),
            text: partial.text,
            start: end,
            end,
            replacement: "Smith".into(),
            full_replace: false,
        });
        assert_eq!(complete.status, QueryStatus::Valid);
        assert!(
            complete
                .document
                .unwrap()
                .nodes
                .iter()
                .any(|n| n.tag_id.is_some())
        );
        let changed = edit_query_text(QueryTextEdit {
            document: start.document.unwrap(),
            text: start.text,
            start: 0,
            end: 3,
            replacement: "title".into(),
            full_replace: false,
        });
        assert_eq!(changed.status, QueryStatus::Valid);
        assert!(
            changed
                .document
                .unwrap()
                .nodes
                .iter()
                .all(|n| n.tag_id.is_none())
        );
    }
    #[test]
    fn bound_projection_budget_cannot_bypass_text_nesting_limit() {
        let mut doc = query::analyze("x").document.unwrap();
        for _ in 0..65 {
            let child = doc.root.unwrap();
            doc.nodes.push(Node {
                kind: NodeKind::Not,
                value: String::new(),
                tag_id: None,
                children: vec![child],
                span: 0..1,
            });
            doc.root = Some(doc.nodes.len() - 1);
        }
        assert!(query::validate_document(&doc).is_ok());
        assert_eq!(
            render_query(record(doc)).diagnostics[0].code,
            "NestingLimitExceeded"
        );
    }
}

#[cfg(test)]
mod snapshot_regressions {
    use super::*;
    #[test]
    fn same_content_rewrite_invalidates_result_incarnation() {
        let file = tempfile::NamedTempFile::new().unwrap();
        fs::write(file.path(), "@article{A,title={graph}}").unwrap();
        let path = file.path().to_str().unwrap().to_owned();
        let version = crate::api::file_version(&path).unwrap();
        let context = SearchContext {
            path: path.clone(),
            file_version: version,
            tag_revision: None,
            query_version: 0,
        };
        let modified = fs::metadata(file.path()).unwrap().modified().unwrap();
        fs::File::options()
            .write(true)
            .open(file.path())
            .unwrap()
            .set_times(
                fs::FileTimes::new().set_modified(modified + std::time::Duration::from_secs(1)),
            )
            .unwrap();
        assert!(!validate_search_context(context).unwrap());
    }
    #[test]
    fn broken_tag_storage_never_displays_empty_tags_or_overwrites_metadata() {
        let file = tempfile::NamedTempFile::new().unwrap();
        fs::write(file.path(), "@article{A,title={graph}}").unwrap();
        let _lock = crate::tags::lock().unwrap();
        fs::write(crate::tags::path(), "{broken").unwrap();
        drop(_lock);
        let request = AdvancedSearchRequest {
            path: file.path().to_str().unwrap().into(),
            mode: SearchMode::Advanced,
            input_kind: QueryInputKind::Text,
            text: "graph".into(),
            document: None,
            expected_tag_revision: None,
            default_field: "all".into(),
            type_filter: "all".into(),
            query_version: 0,
        };
        assert!(
            advanced_search(request.clone())
                .unwrap_err()
                .to_string()
                .contains("InvalidSearchMetadata")
        );
        let mut tagged = request;
        tagged.text = "-tag:read".into();
        assert!(
            advanced_search(tagged)
                .unwrap_err()
                .to_string()
                .contains("InvalidSearchMetadata")
        );
        assert_eq!(fs::read_to_string(crate::tags::path()).unwrap(), "{broken");
    }
}

#[cfg(test)]
mod metadata_identity_regressions {
    use super::*;
    fn mutation(
        action: TagAction,
        id: &str,
        name: &str,
        target_id: &str,
        path: &str,
        revision: u64,
    ) -> TagMutation {
        let snapshot = if path.is_empty() {
            None
        } else {
            Some(tag_snapshot(path.into(), "A".into()).unwrap())
        };
        TagMutation {
            library_uid: snapshot
                .as_ref()
                .map(|s| s.library_uid.clone())
                .unwrap_or_default(),
            record_uid: snapshot
                .as_ref()
                .map(|s| s.record_uid.clone())
                .unwrap_or_default(),
            color: None,
            action,
            id: id.into(),
            name: name.into(),
            target_id: target_id.into(),
            path: path.into(),
            cite_key: "A".into(),
            expected_revision: revision,
        }
    }
    #[test]
    fn merge_redirects_are_explicit_and_text_names_keep_their_own_semantics() {
        let file = tempfile::NamedTempFile::new().unwrap();
        fs::write(
            file.path(),
            "@article{A,title={graph}} @article{B,title={other}}",
        )
        .unwrap();
        let path = file.path().to_str().unwrap();
        let mut snapshot = tag_snapshot(path.into(), "A".into()).unwrap();
        snapshot = mutate_tag(mutation(
            TagAction::Create,
            "",
            "Read",
            "",
            path,
            snapshot.revision,
        ))
        .unwrap();
        let old = snapshot.tags[0].id.clone();
        snapshot = mutate_tag(mutation(
            TagAction::Assign,
            &old,
            "",
            "",
            path,
            snapshot.revision,
        ))
        .unwrap();
        snapshot = mutate_tag(mutation(
            TagAction::Create,
            "",
            "Done",
            "",
            path,
            snapshot.revision,
        ))
        .unwrap();
        let target = snapshot
            .tags
            .iter()
            .find(|t| t.name == "Done")
            .unwrap()
            .id
            .clone();
        let bound = build_condition(QueryCondition {
            field_name: "tag".into(),
            text_value: "Read".into(),
            phrase: false,
            exclude: true,
            tag_id: Some(old.clone()),
        });
        snapshot = mutate_tag(mutation(
            TagAction::Merge,
            &old,
            "",
            &target,
            path,
            snapshot.revision,
        ))
        .unwrap();
        let refreshed = refresh_query(bound.document.unwrap()).unwrap();
        assert_eq!(refreshed.tag_revision, Some(snapshot.revision));
        assert!(
            refreshed
                .document
                .as_ref()
                .unwrap()
                .nodes
                .iter()
                .any(|n| n.tag_id.as_deref() == Some(&target))
        );
        let req = AdvancedSearchRequest {
            path: path.into(),
            mode: SearchMode::Advanced,
            input_kind: QueryInputKind::Bound,
            text: refreshed.text,
            document: refreshed.document,
            expected_tag_revision: refreshed.tag_revision,
            default_field: "all".into(),
            type_filter: "all".into(),
            query_version: 0,
        };
        let response = advanced_search(req.clone()).unwrap();
        assert_eq!(response.references.len(), 1);
        let mut text = req;
        text.input_kind = QueryInputKind::Text;
        text.text = "-tag:Read".into();
        text.document = None;
        text.expected_tag_revision = None;
        let by_name = advanced_search(text).unwrap();
        assert_eq!(by_name.references.len(), 2);
        assert!(
            by_name
                .analysis
                .diagnostics
                .iter()
                .any(|d| d.warning && d.code == "UnknownTag")
        );
    }
    #[test]
    fn automatic_unique_relocation_updates_registration_and_retains_source_id() {
        let directory = tempfile::tempdir().unwrap();
        let old = directory.path().join("old.bib");
        let new = directory.path().join("new.bib");
        fs::write(&old, "@article{A,title={graph}}").unwrap();
        crate::add_library("moving".into(), old.to_str().unwrap().into(), None).unwrap();
        let snapshot = tag_snapshot(old.to_str().unwrap().into(), "A".into()).unwrap();
        let id = snapshot.sources[0].id.clone();
        fs::rename(&old, &new).unwrap();
        crate::update_library(
            "moving".into(),
            "moving".into(),
            Some(new.to_str().unwrap().into()),
            None,
        )
        .unwrap();
        let after = tag_snapshot(new.to_str().unwrap().into(), "A".into()).unwrap();
        assert_eq!(after.sources.len(), 1);
        assert_eq!(after.sources[0].id, id);
        assert_eq!(
            crate::libraries().unwrap()[0].path,
            fs::canonicalize(new).unwrap().to_str().unwrap()
        );
    }
}

#[cfg(test)]
mod scoped_binding_review_regressions {
    use super::*;

    #[test]
    fn negating_a_bound_tag_leaf_preserves_the_id_and_field_scope() {
        let built = build_condition(QueryCondition {
            field_name: "tag".into(),
            text_value: "read".into(),
            phrase: false,
            exclude: false,
            tag_id: Some("stable-tag".into()),
        });
        let doc = built.document.unwrap();
        let leaf = doc.nodes.iter().position(|n| n.tag_id.is_some()).unwrap() as u32;
        let edited = edit_query(QueryEdit {
            document: doc,
            action: QueryEditAction::Negate,
            target: Some(leaf),
            operand: None,
        });
        assert_eq!(edited.status, QueryStatus::Valid);
        assert_eq!(edited.text, "tag:(NOT read)");
        let doc = edited.document.unwrap();
        assert_eq!(
            doc.nodes
                .iter()
                .filter_map(|n| n.tag_id.as_deref())
                .collect::<Vec<_>>(),
            vec!["stable-tag"]
        );
        assert_eq!(render_query(doc).text, edited.text);
    }
}

fn refresh_tag_targets(targets: &[TagTarget]) -> Result<(), CoreError> {
    crate::registry::recover_search_transaction()?;
    for uid in targets
        .iter()
        .map(|target| &target.library_uid)
        .collect::<std::collections::BTreeSet<_>>()
    {
        let path = crate::registry::path_for_uid(uid)?;
        let source = crate::api::references(&path)?;
        crate::tags::load_snapshot_uid(uid, &path, &source.version, &source.references)?;
    }
    Ok(())
}

/// Read all/partial tag assignments for a stable, possibly cross-library selection.
pub fn batch_tag_snapshot(targets: Vec<TagTarget>) -> Result<BatchTagSnapshot, CoreError> {
    refresh_tag_targets(&targets)?;
    crate::tags::snapshot_batch(&targets).map_err(Into::into)
}

/// Apply one tag operation atomically; any invalid target or stale revision aborts it.
pub fn mutate_batch_tag(mutation: BatchTagMutation) -> Result<BatchTagSnapshot, CoreError> {
    refresh_tag_targets(&mutation.targets)?;
    crate::tags::mutate_batch(mutation).map_err(Into::into)
}
