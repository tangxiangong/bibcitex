use crate::workbench::chunks;
use bibcitex_service::ReferenceRecord;

pub fn fields(r: &ReferenceRecord) -> Vec<(&'static str, String)> {
    vec![
        ("引用键", r.cite_key.clone()),
        ("类型", r.entry_type.clone()),
        ("作者", r.author.join(", ")),
        ("年份", r.year.map(|v| v.to_string()).unwrap_or_default()),
        ("月份", r.month.clone().unwrap_or_default()),
        ("期刊", r.journal.clone().unwrap_or_default()),
        ("期刊全称", r.full_journal.clone().unwrap_or_default()),
        ("卷号", r.volume.map(|v| v.to_string()).unwrap_or_default()),
        ("编号", r.number.clone().unwrap_or_default()),
        (
            "页码",
            r.pages
                .as_ref()
                .map(|p| format!("{}–{}", p.start, p.end))
                .unwrap_or_default(),
        ),
        ("总页数", r.book_pages.clone().unwrap_or_default()),
        ("出版社", r.publisher.join(", ")),
        ("版本", r.edition.map(|v| v.to_string()).unwrap_or_default()),
        ("丛书", r.series.clone().unwrap_or_default()),
        (
            "metadata.editor",
            r.editor
                .iter()
                .map(|e| {
                    if e.role.is_empty() {
                        e.name.clone()
                    } else {
                        format!("{} ({})", e.name, e.role)
                    }
                })
                .collect::<Vec<_>>()
                .join(", "),
        ),
        ("学校", r.school.clone().unwrap_or_default()),
        ("地址", r.address.clone().unwrap_or_default()),
        ("组织", r.organization.join(", ")),
        ("机构", r.institution.clone().unwrap_or_default()),
        ("DOI", r.doi.clone().unwrap_or_default()),
        ("ISBN", r.isbn.clone().unwrap_or_default()),
        ("MR 分类", r.mrclass.clone().unwrap_or_default()),
        ("URL", r.url.clone().unwrap_or_default()),
        ("文件", r.file.clone().unwrap_or_default()),
        ("Eprint", r.eprint.clone().unwrap_or_default()),
        (
            "Archive Prefix",
            r.archive_prefix.clone().unwrap_or_default(),
        ),
        (
            "arXiv 分类",
            r.arxiv_primary_class.clone().unwrap_or_default(),
        ),
        ("发表方式", r.how_published.clone().unwrap_or_default()),
        ("摘要", chunks(&r.abstract_text)),
        ("书名", chunks(&r.book_title)),
        ("期号", chunks(&r.issue)),
        ("备注", chunks(&r.note)),
    ]
}
