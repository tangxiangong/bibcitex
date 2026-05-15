import type { Chunk, EntryType, Reference } from "../../types";
import { getChunkValue } from "../../types";
import type { SvgIconName } from "../ui/SvgIcon";

export type ReferenceTypeKey =
  | "article"
  | "book"
  | "booklet"
  | "inBook"
  | "inCollection"
  | "inProceedings"
  | "manual"
  | "misc"
  | "proceedings"
  | "techReport"
  | "thesis"
  | "unpublished"
  | "unknown";

export interface TypeStyle {
  label: string;
  badgeClass: string;
  borderClass: string;
  icon: SvgIconName;
}

export interface MetadataStyle {
  icon: SvgIconName;
  chipClass: string;
  textClass: string;
}

export const TYPE_STYLES: Record<ReferenceTypeKey, TypeStyle> = {
  article: {
    label: "期刊论文",
    badgeClass: "badge bg-info/15 text-base-content border-info/30",
    borderClass: "border-info",
    icon: "fileText",
  },
  book: {
    label: "图书",
    badgeClass: "badge bg-primary/15 text-base-content border-primary/30",
    borderClass: "border-primary",
    icon: "book",
  },
  booklet: {
    label: "小册子",
    badgeClass: "badge bg-secondary/15 text-base-content border-secondary/30",
    borderClass: "border-secondary",
    icon: "book",
  },
  inBook: {
    label: "书籍章节",
    badgeClass: "badge bg-accent/15 text-base-content border-accent/30",
    borderClass: "border-accent",
    icon: "book",
  },
  inCollection: {
    label: "文集章节",
    badgeClass: "badge bg-success/15 text-base-content border-success/30",
    borderClass: "border-success",
    icon: "library",
  },
  inProceedings: {
    label: "会议论文",
    badgeClass: "badge bg-warning/15 text-base-content border-warning/30",
    borderClass: "border-warning",
    icon: "fileText",
  },
  manual: {
    label: "手册",
    badgeClass: "badge bg-neutral/15 text-base-content border-neutral/30",
    borderClass: "border-neutral",
    icon: "fileText",
  },
  misc: {
    label: "其他",
    badgeClass: "badge bg-base-200 text-base-content border-base-300",
    borderClass: "border-base-300",
    icon: "tag",
  },
  proceedings: {
    label: "会议论文集",
    badgeClass: "badge bg-warning/15 text-base-content border-warning/30",
    borderClass: "border-warning",
    icon: "library",
  },
  techReport: {
    label: "技术报告",
    badgeClass: "badge bg-error/15 text-base-content border-error/30",
    borderClass: "border-error",
    icon: "fileText",
  },
  thesis: {
    label: "学位论文",
    badgeClass: "badge bg-secondary/15 text-base-content border-secondary/30",
    borderClass: "border-secondary",
    icon: "book",
  },
  unpublished: {
    label: "未发表",
    badgeClass: "badge bg-neutral/15 text-base-content border-neutral/30",
    borderClass: "border-neutral",
    icon: "fileText",
  },
  unknown: {
    label: "未知类型",
    badgeClass: "badge bg-base-200 text-base-content border-base-300",
    borderClass: "border-base-300",
    icon: "tag",
  },
};

export const METADATA_STYLES = {
  title: {
    icon: "fileText",
    chipClass: "bg-primary/10 text-base-content border border-primary/30",
    textClass: "text-base-content",
  },
  author: {
    icon: "user",
    chipClass: "bg-secondary/10 text-base-content border border-secondary/30",
    textClass: "text-base-content",
  },
  year: {
    icon: "calendar",
    chipClass: "bg-accent/10 text-base-content border border-accent/30",
    textClass: "text-base-content",
  },
  venue: {
    icon: "book",
    chipClass: "bg-info/10 text-base-content border border-info/30",
    textClass: "text-base-content",
  },
  link: {
    icon: "externalLink",
    chipClass: "bg-success/10 text-base-content border border-success/30",
    textClass: "text-base-content",
  },
  citeKey: {
    icon: "tag",
    chipClass: "bg-warning/10 text-base-content border border-warning/30 font-mono",
    textClass: "text-base-content",
  },
  note: {
    icon: "fileText",
    chipClass: "bg-neutral/10 text-base-content border border-neutral/30",
    textClass: "text-base-content",
  },
} as const satisfies Record<string, MetadataStyle>;

export function getReferenceTypeKey(type: EntryType): ReferenceTypeKey {
  const rawType = typeof type === "string" ? type : type.Unknown;

  switch (rawType) {
    case "Article":
      return "article";
    case "Book":
      return "book";
    case "Booklet":
      return "booklet";
    case "InBook":
      return "inBook";
    case "InCollection":
      return "inCollection";
    case "InProceedings":
      return "inProceedings";
    case "Manual":
      return "manual";
    case "MastersThesis":
    case "PhdThesis":
    case "Thesis":
      return "thesis";
    case "Proceedings":
      return "proceedings";
    case "TechReport":
      return "techReport";
    case "Unpublished":
      return "unpublished";
    case "Misc":
      return "misc";
    default:
      return "unknown";
  }
}

export function getChunkText(chunks?: Chunk[]): string {
  return chunks?.map(getChunkValue).join("").trim() ?? "";
}

export function getReferenceTitleText(reference: Reference): string {
  return getChunkText(reference.title) || "暂无标题";
}

export function getReferenceVenue(reference: Reference): string {
  return (
    reference.full_journal ||
    reference.journal ||
    getChunkText(reference.book_title) ||
    reference.publisher?.join(", ") ||
    reference.school ||
    reference.institution ||
    reference.organization?.join(", ") ||
    reference.how_published ||
    ""
  );
}
