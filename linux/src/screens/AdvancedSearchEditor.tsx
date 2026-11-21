import { batch, effect, Scroll, signal, untracked, View } from "nsh";
import { Button as BaseButton, Input } from "nsh/base";
import {
  Query,
  Tags,
  type QueryAnalysisDto,
  type QueryDocumentDto,
  type QueryEditActionDto,
  type QueryNodeDto,
  type SearchCatalogDto,
  type TagDto,
} from "bibcitex/native";
import { tr } from "../i18n";
import { dialogs } from "../theme";
import { tagsChanged } from "../store/desktop";
import { errorText } from "../store/format";
import { Button, DialogFrame, SegmentedControl, Select, Separator, Switch, UIText, cx } from "../ui";

const inputClass =
  "h-7 px-2.5 rounded-(--radius-control) border border-input bg-background text-(length:--fs-body) text-foreground placeholder:text-muted-foreground focus:border-primary";

interface Row {
  index: number;
  depth: number;
}

/** The tree's nodes in reading order with their depth (bounded). */
function visibleNodes(doc: QueryDocumentDto): Row[] {
  if (doc.root == null) return [];
  const pending: Row[] = [{ index: doc.root, depth: 0 }];
  const out: Row[] = [];
  while (pending.length > 0 && out.length < 4096) {
    const row = pending.pop()!;
    const node = doc.nodes[row.index];
    if (!node) continue;
    out.push(row);
    for (let i = node.children.length - 1; i >= 0; i--) pending.push({ index: node.children[i], depth: row.depth + 1 });
  }
  return out;
}

/** The diagnostics of an analysis, as the workbench shows them. */
function DiagnosticList(props: { analysis: QueryAnalysisDto | null }) {
  return (
    <View class="gap-0.75" selectable>
      <Each of={props.analysis?.diagnostics ?? []}>
        {(d) => (
          <UIText variant="caption" tone={d().warning ? "secondary" : "danger"}>
            {`${tr(d().code)} · ${d().start}–${d().end}`}
          </UIText>
        )}
      </Each>
    </View>
  );
}

/**
 * 搜索条件 (560×640): the query as a tree of conditions. Conditions are built
 * by the shared service from a field, a value and a match mode, appended
 * with AND / OR, negated, replaced or removed; tag conditions keep the tag's
 * ID. Applying hands the canonical text and the bound tree to the workbench.
 */
export function AdvancedSearchEditor(props: {
  open: boolean;
  text: string;
  document: QueryDocumentDto | null;
  onApply: (text: string, document: QueryDocumentDto | null, tagRevision: string | null) => void;
  onClose: () => void;
}) {
  const analysis = signal<QueryAnalysisDto | null>(null);
  const catalog = signal<SearchCatalogDto | null>(null);
  const tags = signal<readonly TagDto[]>([]);
  const tagRevision = signal<string | null>(null);
  const selected = signal<number | null>(null);
  const field = signal("");
  const value = signal("");
  const yearFrom = signal("");
  const yearTo = signal("");
  const exclude = signal(false);
  const tagId = signal<string | null>(null);
  const combineOr = signal(false);
  const matchStyle = signal<"all" | "any" | "phrase">("all");
  const error = signal<string | null>(null);
  const busy = signal(false);
  let generation = 0;

  const descriptor = () => catalog()?.fieldDescriptors.find((f) => f.name === field());
  const matchMode = () => (field() === "" ? "text" : (descriptor()?.matchMode ?? ""));
  const supportsPhrase = () => field() === "" || descriptor()?.supportsPhrase === true;
  const conditionValue = () => (matchMode() === "year" ? `${yearFrom()}..${yearTo()}` : value());
  const ready = () => (matchMode() === "tag" ? tagId() !== null : conditionValue().replace("..", "") !== "");

  const load = async (fromCurrent: boolean) => {
    const version = ++generation;
    try {
      catalog.set(Query.catalog());
    } catch {
      catalog.set(null);
    }
    try {
      const current = untracked(analysis);
      let resolved: QueryAnalysisDto;
      if (fromCurrent && current?.status === "valid" && current.document) resolved = await Query.refresh(current.document);
      else {
        const syntax = await Query.analyze(untracked(() => props.text));
        const doc = untracked(() => props.document);
        resolved = syntax.status === "valid" && doc ? await Query.refresh(doc) : syntax;
      }
      if (version !== generation) return;
      analysis.set(resolved);
      const snapshot = await Tags.catalog();
      if (version !== generation) return;
      batch(() => {
        tags.set(snapshot.tags);
        tagRevision.set(snapshot.revision);
      });
    } catch (e) {
      if (version === generation) error.set(errorText(e));
    }
  };

  effect(() => {
    if (!props.open) return;
    untracked(() => {
      batch(() => {
        analysis.set(null);
        selected.set(null);
        field.set("");
        value.set("");
        yearFrom.set("");
        yearTo.set("");
        exclude.set(false);
        tagId.set(null);
        combineOr.set(false);
        matchStyle.set("all");
        error.set(null);
      });
      void load(false);
    });
  });

  let seenTags = untracked(tagsChanged);
  effect(() => {
    const revision = tagsChanged();
    if (revision === seenTags) return;
    seenTags = revision;
    if (untracked(() => props.open)) void load(true);
  });

  const label = (node: QueryNodeDto): string => {
    switch (node.kind) {
      case "and":
        return "AND";
      case "or":
        return "OR";
      case "not":
        return "NOT";
      case "scope": {
        const d = catalog()?.fieldDescriptors.find((f) => f.name === node.textValue);
        return `${d ? tr(d.labelKey) : node.textValue}:`;
      }
      case "phrase":
        return `"${node.textValue}"`;
      default:
        return node.textValue;
    }
  };

  const edit = async (action: QueryEditActionDto, operand: QueryDocumentDto | null = null) => {
    const current = analysis();
    if (current?.status !== "valid" || !current.document) return;
    generation++;
    try {
      const result = await Query.edit({ document: current.document, action, target: selected(), operand });
      batch(() => {
        analysis.set(result);
        selected.set(null);
      });
    } catch (e) {
      error.set(errorText(e));
    }
  };

  const addCondition = async (replace: boolean) => {
    if (busy()) return;
    generation++;
    busy.set(true);
    try {
      const tag = tags().find((t) => t.id === tagId());
      const any = matchStyle() === "any";
      const result = await Query.condition({
        fieldName: field(),
        textValue: matchMode() === "tag" ? (tag?.name ?? "") : conditionValue(),
        phrase: supportsPhrase() && matchStyle() === "phrase",
        exclude: exclude() && !any,
        tagId: matchMode() === "tag" ? (tag?.id ?? null) : null,
      });
      if (result.status !== "valid" || !result.document) {
        error.set(result.diagnostics.map((d) => tr(d.code)).join(", "));
        return;
      }
      let operand = result.document;
      if (any && operand.root != null) {
        // The service drew the token boundaries; only the group operator changes.
        const nodes = operand.nodes.map((n) => ({ ...n }));
        let root = operand.root;
        while (nodes[root] && (nodes[root].kind === "scope" || nodes[root].kind === "not") && nodes[root].children.length > 0) {
          root = nodes[root].children[0];
        }
        if (nodes[root]?.kind === "and") nodes[root].kind = "or";
        operand = { ...operand, nodes };
        if (exclude()) {
          const negated = await Query.edit({ document: operand, action: "negate", target: null, operand: null });
          if (negated.status !== "valid" || !negated.document) {
            error.set(negated.diagnostics.map((d) => tr(d.code)).join(", "));
            return;
          }
          operand = negated.document;
        }
      }
      const current = analysis();
      const next = current?.document
        ? await Query.edit({
            document: current.document,
            action: replace ? "replace" : combineOr() ? "appendOr" : "appendAnd",
            target: selected(),
            operand,
          })
        : await Query.render(operand);
      batch(() => {
        analysis.set(next);
        value.set("");
        error.set(null);
      });
    } catch (e) {
      error.set(errorText(e));
    } finally {
      busy.set(false);
    }
  };

  const apply = () => {
    const current = analysis();
    if (current?.status !== "valid") return;
    props.onApply(current.text, current.document ?? null, current.tagRevision ?? tagRevision());
    props.onClose();
  };

  return (
    <DialogFrame
      open={props.open}
      onOpenChange={(open) => !open && props.onClose()}
      width={dialogs.advancedSearch.width}
      height={dialogs.advancedSearch.height}
      busy={busy()}
    >
      <Scroll class="flex-1 -m-2.5" contentClass="gap-3">
        <UIText variant="headline">{tr("搜索条件")}</UIText>
        <If when={analysis()?.document}>
          {(doc) => (
            <View class="gap-2">
              <Scroll class="max-h-55 rounded-(--radius-card) border border-separator" contentClass="p-1 gap-1.25">
                <Each of={visibleNodes(doc())} key={(r) => String(r.index)}>
                  {(row) => (
                    <BaseButton
                      class={cx("flex-row px-1.5 py-1.5 rounded-(--radius-keycap) hover:bg-hover", "focus-visible:outline-2 focus-visible:outline-ring")}
                      selected={selected() === row().index}
                      accessibilityLabel={label(doc().nodes[row().index])}
                      onPress={() => selected.set(row().index)}
                    >
                      <View classList={{ "bg-primary/15": selected() === row().index }} class="flex-1 flex-row rounded-(--radius-keycap)">
                        <View style={{ width: Math.min(row().depth, 12) * 12 }} />
                        <UIText lines={1}>{label(doc().nodes[row().index])}</UIText>
                      </View>
                    </BaseButton>
                  )}
                </Each>
              </Scroll>
              <View class="flex-row gap-2">
                <Button label={tr("整组取反")} size="sm" disabled={selected() === null} onPress={() => void edit("negate")} />
                <Button label={tr("移除")} size="sm" disabled={selected() === null} onPress={() => void edit("remove")} />
                <Button label={tr("全部条件")} size="sm" onPress={() => selected.set(null)} />
              </View>
            </View>
          )}
        </If>
        <Separator />
        <Select
          accessibilityLabel={tr("字段")}
          size="md"
          value={field()}
          options={[
            { value: "", title: tr("全部字段") },
            ...(catalog()?.fieldDescriptors ?? []).map((f) => ({ value: f.name, title: tr(f.labelKey) })),
          ]}
          onChange={(next) =>
            batch(() => {
              field.set(next);
              matchStyle.set("all");
              value.set("");
            })
          }
        />
        <If
          when={matchMode() === "tag"}
          else={
            <If
              when={matchMode() === "type"}
              else={
                <Input
                  value={value()}
                  placeholder={tr("条件值")}
                  accessibilityLabel={tr("条件值")}
                  class={inputClass}
                  onChange={({ value: next }) => value.set(next)}
                  onSubmit={() => ready() && void addCondition(false)}
                />
              }
            >
              <Select
                accessibilityLabel={tr("类型")}
                placeholder={tr("选择类型")}
                size="md"
                value={value() || null}
                options={(catalog()?.typeDescriptors ?? []).map((t) => ({ value: t.name, title: tr(t.labelKey) }))}
                onChange={(next) => value.set(next)}
              />
            </If>
          }
        >
          <Select
            accessibilityLabel={tr("标签")}
            placeholder={tr("选择标签")}
            size="md"
            value={tagId()}
            options={tags().map((t) => ({ value: t.id, title: t.name }))}
            onChange={(next) => tagId.set(next)}
          />
        </If>
        <If when={matchMode() === "year"}>
          <View class="flex-row items-center gap-2">
            <Input
              value={yearFrom()}
              placeholder={tr("起始年份")}
              accessibilityLabel={tr("起始年份")}
              class={cx(inputClass, "flex-1")}
              onChange={({ value: next }) => yearFrom.set(next)}
            />
            <UIText tone="secondary">…</UIText>
            <Input
              value={yearTo()}
              placeholder={tr("结束年份")}
              accessibilityLabel={tr("结束年份")}
              class={cx(inputClass, "flex-1")}
              onChange={({ value: next }) => yearTo.set(next)}
            />
          </View>
        </If>
        <Select
          accessibilityLabel={tr("匹配方式")}
          size="md"
          disabled={!supportsPhrase()}
          value={matchStyle()}
          options={[
            { value: "all", title: tr("包含全部关键词") },
            { value: "any", title: tr("包含任意关键词") },
            { value: "phrase", title: tr("包含完整短语") },
          ]}
          onChange={(next) => matchStyle.set(next as "all" | "any" | "phrase")}
        />
        <View class="flex-row items-center gap-3">
          <Switch label={tr("排除")} checked={exclude()} onChange={exclude.set} />
          <UIText variant="callout" tone="secondary">
            {tr("组合")}
          </UIText>
          <SegmentedControl
            class="w-32.5"
            value={combineOr() ? "or" : "and"}
            options={[
              { value: "and", label: "AND" },
              { value: "or", label: "OR" },
            ]}
            onChange={(next) => combineOr.set(next === "or")}
          />
        </View>
        <View class="flex-row gap-2">
          <Button label={tr("添加条件")} disabled={busy() || !ready()} onPress={() => void addCondition(false)} />
          <Button label={tr("替换条件")} disabled={busy() || selected() === null || !ready()} onPress={() => void addCondition(true)} />
        </View>
        <If when={analysis()}>
          {(a) => (
            <View class="gap-1">
              <UIText variant="monoCaption" selectable>
                {a().text}
              </UIText>
              <DiagnosticList analysis={a()} />
            </View>
          )}
        </If>
        <If when={error()}>
          {(message) => (
            <UIText variant="caption" tone="danger" selectable>
              {message()}
            </UIText>
          )}
        </If>
        <View class="flex-row justify-end gap-2 pt-1">
          <Button label={tr("取消")} onPress={props.onClose} />
          <Button label={tr("应用")} variant="primary" disabled={analysis()?.status !== "valid" || busy()} onPress={apply} />
        </View>
      </Scroll>
    </DialogFrame>
  );
}
