import type { JSX } from "nsh";
import { Scroll, View } from "nsh";
import { Button as BaseButton } from "nsh/base";
import type { ReferenceDto, TagDto } from "bibcitex/native";
import { tr } from "../i18n";
import { authorLine, detailFields, fieldRanges, linkLabel, metaLine, referenceKey, richSections } from "../store/format";
import { formulaAt } from "../store/math";
import type { Workbench } from "../store/workbench";
import {
  Checkbox,
  CiteKeyChip,
  ContextMenu,
  CopyButton,
  Icon,
  IconButton,
  ListRow,
  MathText,
  Separator,
  TagCapsule,
  UIText,
  cx,
  type CopySurface,
  type IconName,
} from "../ui";

export const formula13 = formulaAt(13);
export const formula12 = formulaAt(12);
export const formula15 = formulaAt(15);

/** A title, or “暂无标题” when the entry has none. */
export function TitleText(props: {
  reference: ReferenceDto;
  variant?: "body" | "callout" | "title3";
  weight?: "regular" | "medium" | "semibold";
  lines?: number;
  class?: string;
}) {
  const formula = () => (props.variant === "title3" ? formula15 : props.variant === "callout" ? formula12 : formula13);
  return (
    <If
      when={props.reference.titleChunks.length > 0}
      else={
        <UIText variant={props.variant ?? "body"} tone="secondary" class={props.class}>
          {tr("暂无标题")}
        </UIText>
      }
    >
      <MathText
        chunks={props.reference.titleChunks}
        highlights={fieldRanges(props.reference.matchRanges, "title")}
        formula={formula()}
        variant={props.variant ?? "body"}
        weight={props.weight}
        lines={props.lines}
        class={props.class}
      />
    </If>
  );
}

/** Authors with their search matches tinted. */
export function AuthorText(props: { reference: ReferenceDto; variant: "callout" | "caption" | "small" }) {
  const line = () => authorLine(props.reference);
  return (
    <If when={props.reference.authors.length > 0}>
      <MathText chunks={line().chunks} highlights={line().ranges} formula={formula12} variant={props.variant} tone="secondary" lines={1} />
    </If>
  );
}

/** One bounded line of tags: two chips and “+N” for the rest. */
export function TagChips(props: { tags: readonly TagDto[]; onPress?: () => void }) {
  const shown = () => props.tags.slice(0, 2);
  const rest = () => props.tags.length - 2;
  return (
    <If when={props.tags.length > 0}>
      <View class="flex-row items-center gap-1 max-w-40">
        <Each of={shown()} key={(t) => t.id}>
          {(tag) => <TagCapsule name={tag().name} color={tag().color} onPress={props.onPress} />}
        </Each>
        <If when={rest() > 0}>
          <UIText variant="caption" tone="secondary" lines={1}>
            {`+${rest()}`}
          </UIText>
        </If>
      </View>
    </If>
  );
}

/** A result row of a workbench list (main window or tray). It reads its
 * item through props in bindings, so a recycled cell shows its new item. */
export function ResultRow(props: {
  wb: Workbench;
  reference: ReferenceDto;
  variant: "workbench" | "tray";
  onTags: (reference: ReferenceDto) => void;
}) {
  const key = () => referenceKey(props.reference);
  const tray = () => props.variant === "tray";
  const press = () => {
    if (props.wb.batchActive()) props.wb.toggleBatchItem(key());
    else props.wb.select(key());
  };
  return (
    <ContextMenu
      entries={[
        { label: tr("标签"), icon: "tag", onSelect: () => props.onTags(props.reference) },
        { label: tr("引用"), icon: "copy", onSelect: () => void props.wb.cite([props.reference], "list") },
        { label: tr("复制 BibTeX"), icon: "clipboard", onSelect: () => void props.wb.copyBibtex(props.reference, "list") },
      ]}
      trigger={
        <ListRow
          variant={props.variant}
          selected={props.wb.batchIds().has(key())}
          focused={props.wb.batchActive() && props.wb.selected() === key()}
          onPress={press}
          accessibilityLabel={props.reference.citeKey}
          leading={
            <If when={props.wb.batchActive()}>
              <View class="pt-0.5">
                <Checkbox
                  checked={props.wb.batchIds().has(key())}
                  accessibilityLabel={props.reference.citeKey}
                  onChange={() => props.wb.toggleBatchItem(key())}
                />
              </View>
            </If>
          }
          trailing={
            <View class="items-end gap-1.5">
              <CiteKeyChip
                citeKey={props.reference.citeKey}
                copied={props.wb.copiedOn("list") === props.wb.citeToken([props.reference])}
                onPress={() => void props.wb.cite([props.reference], "list")}
                maxWidthClass="max-w-40"
              />
              <TagChips tags={props.reference.tags} onPress={() => props.onTags(props.reference)} />
            </View>
          }
        >
          <TitleText reference={props.reference} lines={2} />
          <AuthorText reference={props.reference} variant={tray() ? "caption" : "callout"} />
          <UIText variant={tray() ? "small" : "caption"} tone="secondary" lines={1}>
            {metaLine(props.reference)}
          </UIText>
        </ListRow>
      }
    />
  );
}

/** A value that opens its target (DOI, URL, attachment) when pressed. */
function DetailLink(props: { text: string; label: string; onPress: () => void }) {
  return (
    <BaseButton
      class="self-start rounded-(--radius-keycap) focus-visible:outline-2 focus-visible:outline-ring cursor-pointer"
      accessibilityLabel={props.label}
      onPress={props.onPress}
    >
      <UIText tone="link">{props.text}</UIText>
    </BaseButton>
  );
}

/** A caption label with an optional trailing action. */
function FieldLabel(props: { label: string; children?: JSX.Element }) {
  return (
    <View class="flex-row items-center justify-between min-h-6">
      <UIText variant="caption" tone="secondary">
        {props.label}
      </UIText>
      {props.children}
    </View>
  );
}

/** The detail's tag chips and the control that assigns more. */
export function DetailTags(props: { reference: ReferenceDto; onAdd: () => void }) {
  return (
    <View class="flex-row flex-wrap items-center gap-1.5">
      <Each of={props.reference.tags} key={(t) => t.id}>
        {(tag) => <TagCapsule name={tag().name} color={tag().color} size="detail" />}
      </Each>
      <IconButton icon="add" label={tr("添加")} size="sm" variant="capsule" iconSize={12} onPress={props.onAdd} />
    </View>
  );
}

/**
 * The fields of a reference, in the macOS order, each selectable; the DOI,
 * URL and attachment values are links. `full` is the main inspector; the
 * tray's detail shows its shorter list.
 */
export function ReferenceDetail(props: {
  wb: Workbench;
  reference: ReferenceDto;
  full: boolean;
  onTags: (reference: ReferenceDto) => void;
}) {
  const copySurface: CopySurface = "detail";
  const fields = () => {
    const all = detailFields(props.reference);
    if (props.full) return all;
    const keep = ["引用键", "作者", "类型", "年份", "DOI", "URL"];
    const venue = props.reference.venueText;
    const rows = all.filter((f) => keep.includes(f.id));
    if (venue) rows.splice(4, 0, { id: "期刊", label: tr("期刊"), value: venue });
    return rows;
  };
  const sections = () => {
    const all = richSections(props.reference);
    return props.full ? all : all.filter((s) => s.id === "摘要");
  };
  return (
    <Scroll class="flex-1" contentClass={props.full ? "p-4 gap-4" : "p-3.5 gap-3"}>
      <DetailTags reference={props.reference} onAdd={() => props.onTags(props.reference)} />
      <Each of={fields()} key={(f) => f.id}>
        {(f) => (
          <View class="gap-1">
            <FieldLabel label={f().label}>
              <If when={f().citeKey}>
                <CopyButton
                  icon="copy"
                  label={tr("引用")}
                  copied={props.wb.copiedOn(copySurface) === props.wb.citeToken([props.reference])}
                  onPress={() => void props.wb.cite([props.reference], copySurface)}
                />
              </If>
            </FieldLabel>
            <If
              when={f().link}
              else={
                <UIText variant={props.full ? "body" : "callout"} selectable>
                  {f().value}
                </UIText>
              }
            >
              {(kind) => <DetailLink text={f().value} label={linkLabel(kind())} onPress={() => void props.wb.open(props.reference, kind())} />}
            </If>
          </View>
        )}
      </Each>
      <Each of={sections()} key={(s) => s.id}>
        {(s) => (
          <View class="gap-1">
            <FieldLabel label={s().label} />
            <View selectable>
              <MathText chunks={s().chunks} formula={props.full ? formula13 : formula12} variant={props.full ? "body" : "callout"} />
            </View>
          </View>
        )}
      </Each>
      <View class="gap-1">
        <FieldLabel label="BibTeX">
          <CopyButton
            icon="clipboard"
            label={tr("复制 BibTeX")}
            copied={props.wb.copiedOn(copySurface) === props.wb.bibtexToken(props.reference)}
            onPress={() => void props.wb.copyBibtex(props.reference, copySurface)}
          />
        </FieldLabel>
        <UIText variant="monoCaption" selectable>
          {props.reference.source}
        </UIText>
      </View>
    </Scroll>
  );
}

/** The floating batch actions: 全选 / 清空 / 批量 / 引用 | 结束. */
export function BatchActions(props: { wb: Workbench; onBatchTags: () => void }) {
  const selectedRefs = () => props.wb.selectedReferences();
  const didCopy = () =>
    selectedRefs().length > 0 && props.wb.copiedOn("batch") === props.wb.citeToken(selectedRefs());
  return (
    <View class="flex-row items-center gap-1.5 p-2.5">
      <BatchAction label={tr("全选")} icon="check" onPress={props.wb.selectAll} />
      <BatchAction label={tr("清空")} icon="trash" disabled={selectedRefs().length === 0} onPress={props.wb.clearBatch} />
      <BatchAction label={tr("批量")} icon="tag" disabled={selectedRefs().length === 0} onPress={props.onBatchTags} />
      <BatchAction
        label={tr("引用")}
        icon={didCopy() ? "check" : "copy"}
        disabled={selectedRefs().length === 0}
        onPress={() => void props.wb.cite(selectedRefs(), "batch")}
      />
      <Separator vertical class="h-5 mx-1" />
      <BatchAction label={tr("结束")} icon="x" onPress={() => props.wb.setBatchActive(false)} />
    </View>
  );
}

function BatchAction(props: { label: string; icon: IconName; disabled?: boolean; onPress: () => void }) {
  return (
    <BaseButton
      class={cx(
        "flex-row items-center gap-1.5 px-1.5 py-1.25 rounded-(--radius-keycap)",
        "hover:bg-hover active:bg-selection disabled:opacity-40",
        "focus-visible:outline-2 focus-visible:outline-ring",
      )}
      disabled={props.disabled}
      accessibilityLabel={props.label}
      onPress={props.onPress}
    >
      <Icon name={props.icon} size={14} tone="primary" />
      <UIText variant="callout" lines={1}>
        {props.label}
      </UIText>
    </BaseButton>
  );
}

/** The 多选 / “已选 N” toggle of a toolbar. */
export function BatchToggle(props: { wb: Workbench; /** Round, inside a glass toolbar group. */ round?: boolean }) {
  const title = () => (props.wb.batchActive() ? `${tr("已选")} ${props.wb.batchIds().size}` : tr("多选"));
  return (
    <BaseButton
      class={cx(
        "flex-row items-center justify-center gap-1.5 h-7 px-1.5",
        props.round ? "min-w-7 rounded-full" : "rounded-(--radius-control)",
        "hover:bg-hover active:bg-selection selected:bg-selection disabled:opacity-40",
        "focus-visible:outline-2 focus-visible:outline-ring",
      )}
      selected={props.wb.batchActive()}
      disabled={!props.wb.batchActive() && props.wb.references().length === 0}
      accessibilityLabel={title()}
      onPress={() => props.wb.setBatchActive(!props.wb.batchActive())}
    >
      <Icon name="checklist" size={16} tone="secondary" />
      <If when={props.wb.batchActive()}>
        <UIText variant="callout" lines={1}>
          {title()}
        </UIText>
      </If>
    </BaseButton>
  );
}

/** Query diagnostics under a search field: code and UTF-16 span. */
export function Diagnostics(props: { wb: { analysis: () => import("bibcitex/native").QueryAnalysisDto | null }; limit?: number }) {
  const items = () => (props.wb.analysis()?.diagnostics ?? []).slice(0, props.limit ?? 32);
  return (
    <If when={items().length > 0}>
      <View class="gap-0.75" selectable>
        <Each of={items()}>
          {(d) => (
            <UIText variant="caption" tone={d().warning ? "secondary" : "danger"}>
              {`${tr(d().code)} · ${d().start}–${d().end}`}
            </UIText>
          )}
        </Each>
      </View>
    </If>
  );
}
