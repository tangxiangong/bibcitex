import type { MatchRangeDto, ReferenceDto, TextChunkDto } from "bibcitex/native";
import { tr } from "../i18n";

/** A result's stable identity across searches: library and record UUIDs. */
export function referenceKey(reference: Pick<ReferenceDto, "libraryUid" | "recordUid">): string {
  return `${reference.libraryUid}:${reference.recordUid}`;
}

/** The text of an error a native call rejected with. Messages that are
 * catalog keys (`error.desktopBusy`, `StaleQuery`) are translated. */
export function errorText(error: unknown): string {
  const message =
    typeof error === "string"
      ? error
      : error && typeof error === "object" && "message" in error
        ? String((error as { message: unknown }).message)
        : String(error);
  return tr(message);
}

/** The highlight ranges of one field, entry 0 (titles are one entry). */
export function fieldRanges(matches: readonly MatchRangeDto[], field: string): { start: number; end: number }[] {
  return matches.filter((m) => m.field === field && m.entryIndex === 0).map((m) => ({ start: m.start, end: m.end }));
}

/** Authors joined as the workbench shows them, with each author's matches
 * moved to its offset in the joined text. */
export function authorLine(reference: ReferenceDto): { chunks: TextChunkDto[]; ranges: { start: number; end: number }[] } {
  const ranges: { start: number; end: number }[] = [];
  let offset = 0;
  reference.authors.forEach((author, index) => {
    for (const m of reference.matchRanges) {
      if (m.field === "author" && m.entryIndex === index) ranges.push({ start: offset + m.start, end: offset + m.end });
    }
    offset += author.length + 2;
  });
  return { chunks: [{ kind: "normal", text: reference.authors.join(", ") }], ranges };
}

/** “类型 · 年份 · 刊物”. */
export function metaLine(reference: ReferenceDto): string {
  return [reference.displayType, reference.year != null ? String(reference.year) : "", reference.venueText]
    .filter((part) => part !== "")
    .join(" · ");
}

export type DetailLinkKind = "doi" | "url" | "file";

export interface DetailField {
  /** Stable id of the row (also its catalog key or literal label). */
  id: string;
  label: string;
  value: string;
  link?: DetailLinkKind;
  /** The cite key row carries its own copy action. */
  citeKey?: boolean;
}

const joined = (values: readonly string[]): string => values.join(", ");
const text = (value: string | null | undefined): string => value ?? "";

/** The inspector's metadata in the macOS order; empty values are skipped. */
export function detailFields(reference: ReferenceDto): DetailField[] {
  const rows: DetailField[] = [
    { id: "引用键", label: tr("引用键"), value: reference.citeKey, citeKey: true },
    { id: "类型", label: tr("类型"), value: reference.displayType },
    { id: "作者", label: tr("作者"), value: joined(reference.authors) },
    { id: "年份", label: tr("年份"), value: reference.year != null ? String(reference.year) : "" },
    { id: "月份", label: tr("月份"), value: text(reference.month) },
    { id: "期刊", label: tr("期刊"), value: text(reference.journal) },
    { id: "期刊全称", label: tr("期刊全称"), value: text(reference.fullJournal) },
    { id: "卷号", label: tr("卷号"), value: text(reference.volume) },
    { id: "编号", label: tr("编号"), value: text(reference.number) },
    { id: "页码", label: tr("页码"), value: reference.pagesText },
    { id: "总页数", label: tr("总页数"), value: text(reference.bookPages) },
    { id: "出版社", label: tr("出版社"), value: joined(reference.publishers) },
    { id: "版本", label: tr("版本"), value: text(reference.edition) },
    { id: "丛书", label: tr("丛书"), value: text(reference.series) },
    {
      id: "metadata.editor",
      label: tr("metadata.editor"),
      value: joined(reference.editors.map((e) => (e.role ? `${e.name} (${e.role})` : e.name))),
    },
    { id: "学校", label: tr("学校"), value: text(reference.school) },
    { id: "地址", label: tr("地址"), value: text(reference.address) },
    { id: "组织", label: tr("组织"), value: joined(reference.organizations) },
    { id: "机构", label: tr("机构"), value: text(reference.institution) },
    { id: "DOI", label: "DOI", value: text(reference.doi), link: "doi" },
    { id: "ISBN", label: "ISBN", value: text(reference.isbn) },
    { id: "MR 分类", label: tr("MR 分类"), value: text(reference.mrclass) },
    { id: "URL", label: "URL", value: text(reference.url), link: "url" },
    { id: "文件", label: tr("文件"), value: text(reference.file), link: "file" },
    { id: "Eprint", label: "Eprint", value: text(reference.eprint) },
    { id: "Archive Prefix", label: "Archive Prefix", value: text(reference.archivePrefix) },
    { id: "arXiv 分类", label: tr("arXiv 分类"), value: text(reference.arxivPrimaryClass) },
    { id: "发表方式", label: tr("发表方式"), value: text(reference.howPublished) },
  ];
  return rows.filter((row) => row.value !== "");
}

/** The rich paragraphs below the fields: 摘要, 书名, 期号, 备注. */
export function richSections(reference: ReferenceDto): { id: string; label: string; chunks: TextChunkDto[] }[] {
  return [
    { id: "摘要", label: tr("摘要"), chunks: reference.abstractChunks },
    { id: "书名", label: tr("书名"), chunks: reference.bookTitleChunks },
    { id: "期号", label: tr("期号"), chunks: reference.issueChunks },
    { id: "备注", label: tr("备注"), chunks: reference.noteChunks },
  ].filter((section) => section.chunks.length > 0);
}

/** The label of each link field's action. */
export function linkLabel(kind: DetailLinkKind): string {
  return kind === "doi" ? tr("打开 DOI") : kind === "url" ? tr("打开 URL") : tr("打开文件");
}

/** The type filter choices of the workbench (macOS order). */
export function typeOptions(): { value: string; title: string }[] {
  return [
    { value: "all", title: tr("全部类型") },
    { value: "Article", title: tr("期刊论文") },
    { value: "Book", title: tr("图书") },
    { value: "Thesis", title: tr("学位论文") },
    { value: "TechReport", title: tr("技术报告") },
    { value: "Misc", title: tr("其他") },
    { value: "Booklet", title: tr("小册子") },
    { value: "InBook", title: tr("书籍章节") },
    { value: "InCollection", title: tr("文集章节") },
    { value: "InProceedings", title: tr("会议论文") },
  ];
}

/** The default field choices of the workbench. */
export function fieldOptions(): { value: string; title: string }[] {
  return [
    { value: "all", title: tr("全部字段") },
    { value: "author", title: tr("作者") },
    { value: "title", title: tr("标题") },
    { value: "journal", title: tr("期刊") },
    { value: "year", title: tr("年份") },
  ];
}
