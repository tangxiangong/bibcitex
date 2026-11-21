import type { InputRef, ListRef } from "nsh";
import { effect, List, signal, untracked, View } from "nsh";
import { Button as BaseButton, Input } from "nsh/base";
import type { LibraryDto, ReferenceDto } from "bibcitex/native";
import { tr } from "../i18n";
import { helper as metrics, transition } from "../theme";
import { metaLine, referenceKey } from "../store/format";
import type { Helper as HelperModel } from "../store/helper";
import { Icon, IconButton, Kbd, Separator, Spinner, Surface, TagCapsule, UIText, cx, type NamedKey } from "../ui";
import { AuthorText, TitleText } from "./shared";

const heroInputClass = "text-(length:--fs-hero) text-foreground placeholder:text-muted-foreground";

/** Width of the free-text field: it grows with its text (190 empty, 90
 * after capsules, at most 430), as in the macOS helper. */
function inputWidth(text: string, capsules: number, candidate: boolean): number {
  if (candidate) return text === "" ? 0 : Math.min(text.length * 11 + 12, 430);
  return Math.max(capsules === 0 ? 190 : 90, Math.min(text.length * 11 + 24, 430));
}

/** The candidates' filter (or a `$` field's value), beside the trigger symbol. */
function CandidateField(props: { helper: HelperModel }) {
  const composing = signal(false);
  const state = () => props.helper.candidate();
  return (
    <View class="flex-row items-center gap-1.5">
      <UIText variant="title3" weight="medium" tone="secondary">
        {state()?.kind ?? ""}
      </UIText>
      <View
        class="h-7.5 px-2 justify-center rounded-(--radius-control) bg-control-surface border border-primary"
        captureKeys={composing() ? [] : ["up", "down", "escape", "tab"]}
        onKeyDown={(e) => {
          if (composing()) return;
          if (e.key === "ArrowUp") props.helper.move(-1);
          else if (e.key === "ArrowDown") props.helper.move(1);
          else if (e.key === "Escape") props.helper.cancelCandidates();
        }}
      >
        <Input
          value={state()?.filter ?? ""}
          placeholder={state()?.field !== null ? tr("字段值（可选）") : tr("筛选候选")}
          accessibilityLabel={tr("候选")}
          autoFocus
          autoCapitalize="none"
          autoCorrect={false}
          submitBehavior="submit"
          class="w-35 text-(length:--fs-title3) text-foreground placeholder:text-muted-foreground"
          onChange={({ value }) => props.helper.setCandidateFilter(value)}
          onCompositionChange={({ composing: active }) => composing.set(active)}
          onSubmit={() => !composing() && props.helper.confirmCandidate()}
        />
      </View>
    </View>
  );
}

/** The 64pt search row: glyph, condition capsules, the field, the library switch. */
function Header(props: { helper: HelperModel }) {
  const h = props.helper;
  const composing = signal(false);
  let input: InputRef | undefined;

  effect(() => {
    h.focusRequest();
    if (untracked(h.candidate) === null) input?.focus();
  });

  const placeholder = () =>
    h.capsules().length > 0 || h.candidate() ? "" : h.selectingLibrary() ? tr("搜索或选择文献库") : tr("搜索文献、作者、标题");

  return (
    <View
      class="flex-row items-center gap-3.5 px-4.5"
      style={{ height: metrics.header }}
      captureKeys={composing() || h.candidate() ? [] : ["up", "down", "tab", "escape"]}
      onKeyDown={(e) => {
        if (composing() || h.candidate()) return;
        switch (e.key) {
          case "ArrowUp":
            h.move(-1);
            break;
          case "ArrowDown":
            h.move(1);
            break;
          case "Tab":
            if (!e.repeat) h.tab();
            break;
          case "Escape":
            h.escape();
            break;
          case "Backspace":
            if (h.input() === "") h.deletePreviousCapsule();
            break;
        }
      }}
    >
      <Icon name="search" size={18} tone="secondary" />
      <View class="flex-1 min-w-0 flex-row items-center gap-1.75 overflow-hidden">
        <Each of={h.capsules()} key={(c) => c.id}>
          {(capsule) => (
            <View class="flex-row items-center gap-1.25 px-2.25 py-1.5 rounded-full bg-control-surface">
              <UIText variant="callout" weight="medium" lines={1}>
                {capsule().text}
              </UIText>
              <BaseButton class="rounded-full" accessibilityLabel={tr("移除")} onPress={() => h.removeCapsule(capsule().id)}>
                <Icon name="x" size={9} tone="secondary" />
              </BaseButton>
            </View>
          )}
        </Each>
        <View style={{ width: inputWidth(h.input(), h.capsules().length, h.candidate() !== null) }} class="overflow-hidden">
          <Input
            ref={(r: InputRef) => (input = r)}
            value={h.input()}
            placeholder={placeholder()}
            accessibilityLabel={tr("搜索")}
            autoFocus
            readonly={h.candidate() !== null}
            autoCapitalize="none"
            autoCorrect={false}
            returnKeyType="search"
            submitBehavior="submit"
            class={heroInputClass}
            onChange={({ value }) => h.setInput(value)}
            onCompositionChange={({ composing: active }) => {
              composing.set(active);
              h.setComposing(active);
            }}
            onSubmit={() => !composing() && h.submit()}
          />
        </View>
        <If when={h.candidate()}>
          <CandidateField helper={h} />
        </If>
      </View>
      <If when={h.loading()}>
        <Spinner accessibilityLabel={tr("正在搜索")} />
      </If>
      <If when={h.selectingLibrary() ? null : h.library()}>
        {(lib) => (
          <BaseButton
            class={cx("flex-row items-center gap-1.25 px-2.5 py-1.5 rounded-full bg-control-surface hover:bg-hover max-w-40", transition.colors())}
            accessibilityLabel={tr("切换文献库")}
            onPress={h.startSelectingLibrary}
          >
            <Icon name="library" size={14} tone="secondary" />
            <UIText variant="small" weight="medium" lines={1}>
              {lib().name}
            </UIText>
            <Icon name="chevronDown" size={9} tone="secondary" />
          </BaseButton>
        )}
      </If>
    </View>
  );
}

function HelperReferenceRow(props: { helper: HelperModel; reference: ReferenceDto; index: number }) {
  const selected = () => props.helper.isPicked(props.reference, props.index);
  return (
    <View class="px-0">
      <View
        role="button"
        accessibilityLabel={props.reference.citeKey}
        accessibilityHint={tr("粘贴引用键")}
        accessibilityState={{ selected: selected() }}
        class={cx("flex-row items-start gap-2.5 px-2.5 py-2 rounded-(--radius-row) hover:bg-hover", transition.colors())}
        classList={{ "bg-selection": selected() }}
        style={{ height: metrics.referenceRow }}
        onPress={(e) => {
          if (e.modifiers.platform || e.modifiers.control) props.helper.pick(props.index, "toggle");
          else if (e.modifiers.shift) props.helper.pick(props.index, "extend");
          else props.helper.activate(props.index);
        }}
      >
        <View class="pt-0.5">
          <Icon name="fileText" size={15} tone="secondary" />
        </View>
        <View class="flex-1 min-w-0 gap-0.75">
          <TitleText reference={props.reference} weight="medium" lines={2} />
          <AuthorText reference={props.reference} variant="small" />
          <UIText variant="small" tone="secondary" lines={1}>
            {metaLine(props.reference)}
          </UIText>
        </View>
        <View class="items-end gap-1.5">
          <UIText variant="monoSmall" tone="accent" lines={1}>
            {props.reference.citeKey}
          </UIText>
          <View class="flex-row gap-1">
            <Each of={props.reference.tags.slice(0, 2)} key={(t) => t.id}>
              {(tag) => <TagCapsule name={tag().name} color={tag().color} />}
            </Each>
          </View>
        </View>
      </View>
    </View>
  );
}

function HelperLibraryRow(props: { helper: HelperModel; library: LibraryDto; index: number }) {
  const selected = () => props.helper.libraryIndex() === props.index;
  const current = () => props.helper.library()?.uid === props.library.uid;
  return (
    <View
      role="button"
      accessibilityLabel={props.library.name}
      accessibilityState={{ selected: selected() }}
      class={cx("flex-row items-center gap-2.5 px-2.5 py-2 rounded-(--radius-row) hover:bg-hover", transition.colors())}
      classList={{ "bg-selection": selected() }}
      style={{ height: metrics.libraryRow }}
      onPress={() => void props.helper.chooseLibrary(props.library)}
    >
      <Icon name="library" size={15} tone="secondary" />
      <View class="flex-1 min-w-0 gap-0.75">
        <UIText weight="semibold" tone={props.library.available ? "primary" : "secondary"} lines={1}>
          {props.library.name}
        </UIText>
        <UIText variant="small" tone="secondary" lines={1}>
          {props.library.path}
        </UIText>
      </View>
      <If
        when={props.library.available}
        else={
          <UIText variant="small" tone="secondary">
            {tr("不可用")}
          </UIText>
        }
      >
        <If when={current()}>
          <Icon name="check" size={13} tone="accent" />
        </If>
      </If>
    </View>
  );
}

function Placeholder(props: { text?: string; icon?: "search" | "library"; loading?: boolean }) {
  return (
    <View class="items-center justify-center gap-2" style={{ height: metrics.emptyRow }}>
      <If
        when={props.loading}
        else={
          <>
            <If when={props.icon}>{(icon) => <Icon name={icon()} size={24} tone="secondary" />}</If>
            <UIText variant="callout" tone="secondary">
              {props.text ?? ""}
            </UIText>
          </>
        }
      >
        <Spinner size="md" />
      </If>
    </View>
  );
}

function BarButton(props: { label: string; keyName: NamedKey; disabled?: boolean; onPress: () => void }) {
  return (
    <BaseButton
      class={cx("flex-row items-center gap-1.5 px-2.5 rounded-full hover:bg-hover disabled:opacity-50", transition.colors())}
      style={{ height: metrics.barButton }}
      disabled={props.disabled}
      accessibilityLabel={props.label}
      onPress={props.onPress}
    >
      <UIText variant="callout" weight="medium" tone="secondary" lines={1} class="max-w-60">
        {props.label}
      </UIText>
      <Kbd keyName={props.keyName} />
    </BaseButton>
  );
}

/** The floating capsule over the list: paste into the target, switch library. */
function BottomBar(props: { helper: HelperModel }) {
  const h = props.helper;
  const pasteTitle = () => {
    const name = h.target()?.targetAppName;
    return name ? tr("粘贴到 {0}", name) : tr("粘贴到当前应用");
  };
  return (
    <View class="absolute bottom-0 left-0 right-0 flex-row items-center justify-end px-3" style={{ height: metrics.bottomBar, pointerEvents: "box-none" }}>
      <Surface variant="capsule" class="flex-row items-center gap-0.5 p-1">
        <If when={h.pickedCount() > 1}>
          <UIText variant="small" class="px-2">{`${tr("已选")} ${h.pickedCount()}`}</UIText>
        </If>
        <BarButton label={pasteTitle()} keyName="enter" disabled={h.pickedCount() === 0} onPress={h.activateSelection} />
        <BarButton label={tr("切换文献库")} keyName="tab" onPress={h.startSelectingLibrary} />
      </Surface>
    </View>
  );
}

/** The error line: a failed paste keeps 引用 to copy the citation by hand. */
function ErrorBar(props: { helper: HelperModel }) {
  return (
    <If when={props.helper.error()}>
      {(error) => (
        <View class="flex-row items-center gap-1.75 px-3 py-1.75" style={{ minHeight: metrics.errorBar }}>
          <Icon name="alert" size={14} tone="danger" />
          <UIText variant="small" tone="danger" lines={2} class="flex-1">
            {error().message}
          </UIText>
          <If when={error().ticket !== null || props.helper.pickedCount() > 0}>
            <BaseButton class="px-1 rounded-(--radius-keycap) hover:bg-hover" accessibilityLabel={tr("引用")} onPress={props.helper.copyFallback}>
              <UIText variant="small" tone="accent">
                {tr("引用")}
              </UIText>
            </BaseButton>
          </If>
          <IconButton icon="x" label={tr("关闭")} size="sm" iconSize={10} onPress={props.helper.dismissError} />
        </View>
      )}
    </If>
  );
}

/**
 * The compact helper: idle it is the 64pt search row alone; a query, a
 * search or library selection opens the list below it, up to 460pt. Enter
 * or a press pastes into the app captured before it showed.
 */
export function HelperPanel(props: { helper: HelperModel }) {
  const h = props.helper;
  let list: ListRef | undefined;
  effect(() => {
    const at = h.index();
    if (at !== null && list) list.scrollToIndex({ index: at, animated: false, viewPosition: 0.5 });
  });

  return (
    <Surface variant="panel">
      <View style={{ height: h.height() }}>
        <Header helper={h} />
        <ErrorBar helper={h} />
        <If when={(h.analysis()?.diagnostics.length ?? 0) > 0}>
          <View class="px-4.5 pb-1 flex-row gap-2">
            <Each of={(h.analysis()?.diagnostics ?? []).slice(0, 3)}>
              {(d) => (
                <UIText variant="caption" tone={d().warning ? "secondary" : "danger"} lines={1}>
                  {`${tr(d().code)} · ${d().start}–${d().end}`}
                </UIText>
              )}
            </Each>
          </View>
        </If>
        <If when={h.candidate() && h.candidate()!.field === null}>
          <Separator />
          <View class="flex-1 p-1.5">
            <If
              when={h.candidates().length > 0}
              else={
                <UIText tone="secondary" class="px-1.75 py-1.5">
                  {tr("无候选")}
                </UIText>
              }
            >
              <List each={h.candidates()} keyExtractor={(c) => c.id} estimatedItemSize={28} class="flex-1">
                {(c, i) => (
                  <BaseButton
                    class="h-7 px-1.75 justify-center rounded-(--radius-keycap) hover:bg-hover"
                    selected={h.candidate()?.index === i()}
                    accessibilityLabel={c().name}
                    onPress={() => h.confirmCandidate(i())}
                  >
                    <View class="absolute inset-0 rounded-(--radius-keycap)" classList={{ "bg-highlight": h.candidate()?.index === i() }} />
                    <UIText lines={1}>{c().name}</UIText>
                  </BaseButton>
                )}
              </List>
            </If>
          </View>
        </If>
        <If when={h.showsList() && h.candidate() === null}>
          <Separator />
          <View class="flex-1">
            <If
              when={h.selectingLibrary()}
              else={
                <If
                  when={h.results().length > 0}
                  else={<Placeholder loading={h.loading()} text={tr("未找到匹配记录")} icon="search" />}
                >
                  <List
                    ref={(r: ListRef) => (list = r)}
                    each={h.results()}
                    keyExtractor={referenceKey}
                    estimatedItemSize={metrics.referenceRow + metrics.rowGap}
                    class="flex-1"
                    contentClass="p-2 gap-0.5"
                    contentStyle={{ paddingBottom: h.showsBottomBar() ? metrics.bottomBar : 8 }}
                  >
                    {(item, index) => <HelperReferenceRow helper={h} reference={item()} index={index()} />}
                  </List>
                  <If when={h.showsBottomBar()}>
                    <BottomBar helper={h} />
                  </If>
                </If>
              }
            >
              <If
                when={h.libraries().length > 0}
                else={<Placeholder text={tr("未找到文献库，请先到主窗口添加文献库")} icon="library" />}
              >
                <List each={h.libraries()} keyExtractor={(l) => l.uid} estimatedItemSize={metrics.libraryRow + metrics.rowGap} class="flex-1" contentClass="p-2 gap-0.5">
                  {(item, index) => <HelperLibraryRow helper={h} library={item()} index={index()} />}
                </List>
              </If>
            </If>
          </View>
        </If>
      </View>
    </Surface>
  );
}
