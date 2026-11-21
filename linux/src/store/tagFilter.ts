import type { QueryDocumentDto, QueryNodeDto } from "bibcitex/native";

/** One tag of a filter, optionally excluded (NOT). */
export interface TagFilterTerm {
  tagId: string;
  negated: boolean;
}

/** The independent tag filter of a workbench: its terms joined by AND or
 * OR. Rust evaluates it as a query tree; the tree is built from stable tag
 * IDs, so renames keep it. */
export interface TagFilter {
  combine: "and" | "or";
  terms: readonly TagFilterTerm[];
}

export const emptyTagFilter: TagFilter = { combine: "and", terms: [] };

export function isTagFilterEmpty(filter: TagFilter): boolean {
  return filter.terms.length === 0;
}

/** The filter as the query tree `SearchSession.search` takes. */
export function tagFilterDocument(filter: TagFilter): QueryDocumentDto | null {
  if (filter.terms.length === 0) return null;
  const nodes: QueryNodeDto[] = [];
  const add = (node: Omit<QueryNodeDto, "start" | "end">): number => {
    nodes.push({ ...node, start: 0, end: 0 });
    return nodes.length - 1;
  };
  const leaves = filter.terms.map((term) => {
    const word = add({ kind: "word", textValue: "tag", tagId: term.tagId, children: [] });
    const scope = add({ kind: "scope", textValue: "tag", children: [word] });
    return term.negated ? add({ kind: "not", textValue: "", children: [scope] }) : scope;
  });
  const root = leaves.length === 1 ? leaves[0] : add({ kind: filter.combine, textValue: "", children: leaves });
  return { text: "", nodes, root };
}

/** The filter as text, for the editor's preview: `"A" AND NOT "B"`. */
export function tagFilterExpression(filter: TagFilter, names: Readonly<Record<string, string>>): string {
  return filter.terms
    .map((term) => {
      const name = names[term.tagId];
      const quoted = name === undefined ? "?" : `"${name.replace(/\\/g, "\\\\").replace(/"/g, '\\"')}"`;
      return term.negated ? `NOT ${quoted}` : quoted;
    })
    .join(filter.combine === "or" ? " OR " : " AND ");
}
