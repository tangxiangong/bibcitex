import { updateState } from "../store/updater";
import type { InputRef, ListRef } from "nsh";
import { cleanup, effect, Image, keymap, List, signal, untracked, View, windowSize } from "nsh";
import { Button as BaseButton, Input, Tooltip } from "nsh/base";
import { currentWindow } from "nsh/desktop";
import { Catalog, Files, type LibraryDto, type ReferenceDto, type TagTargetDto } from "bibcitex/native";
import { tr, trReferences } from "../i18n";
import { transition, workbench as metrics } from "../theme";
import logo from "../../.nsh/resources/favicon.png";
import { libraryList, prefs, savePreference, showError } from "../store/desktop";
import { errorText, fieldOptions, referenceKey, typeOptions } from "../store/format";
import { showHelper, setSettingsOpen } from "../store/surfaces";
import type { Workbench as WorkbenchModel } from "../store/workbench";
import {
  ActionMenu,
  Button,
  ContextMenu,
  EmptyState,
  Icon,
  IconButton,
  ListRow,
  PopoverFrame,
  SearchField,
  Select,
  Separator,
  Spinner,
  SplitPanes,
  Surface,
  Switch,
  UIText,
  cx,
  liquidGlass,
  paneLayout,
  tooltipClass,
  type IconName,
  type MenuEntry,
} from "../ui";
import { AdvancedSearchEditor } from "./AdvancedSearchEditor";
import { LibraryEditor } from "./LibraryEditor";
import { BatchActions, BatchToggle, Diagnostics, ReferenceDetail, ResultRow, TitleText } from "./shared";
import { BatchTagEditor, TagManager, type TagTarget } from "./TagDialogs";
import { TagFilterButton } from "./TagFilterButton";

const renameClass =
  "h-6 px-2 rounded-(--radius-control) border border-input bg-background text-(length:--fs-body) text-foreground focus:border-primary";

/** The title bar's height: the 48pt logo and 36pt buttons centred in 52pt,
 * or the system's own when it asks for more. */
const toolbarHeight = 52;

/** macOS 26 Liquid Glass on a toolbar control. Glass alone: no opaque fill,
 * border or shadow over it, and never one glass inside another. */
const glass = liquidGlass ? ({ glassEffect: "regular", backdropFilter: "blur(20px)" } as const) : undefined;

const paneToggleClass = (selected: boolean) =>
  liquidGlass
    ? cx("w-9 h-9 items-center justify-center rounded-full hover:bg-hover/60", selected ? "bg-selection/60" : "")
    : cx("w-8 h-8 items-center justify-center rounded-(--radius-control) hover:bg-hover", selected ? "bg-selection" : "");

/** The sidebar and inspector switches at the bar's ends: round glass
 * buttons on macOS 26, quiet toolbar buttons elsewhere. */
function PaneToggle(props: { icon: IconName; label: string; onPress: () => void }) {
  return (
    <BaseButton
      class={cx(paneToggleClass(false), "focus-visible:outline-2 focus-visible:outline-ring", transition.colors())}
      style={glass}
      accessibilityLabel={props.label}
      onPress={props.onPress}
      tooltip={
        <Tooltip class={tooltipClass}>
          <UIText variant="small">{props.label}</UIText>
        </Tooltip>
      }
    >
      <Icon name={props.icon} size={18} tone="secondary" />
    </BaseButton>
  );
}

/** The same switch as a popover's trigger (the popover toggles it). */
function TriggerGlyph(props: { icon: IconName; label: string; selected?: boolean }) {
  return (
    <View
      class={cx(paneToggleClass(!!props.selected), transition.colors())}
      style={glass}
      accessibilityLabel={props.label}
      role="button"
      desktop={{ windowDrag: false }}
    >
      <Icon name={props.icon} size={18} tone="secondary" />
    </View>
  );
}

/** A button of the trailing group: round inside the glass capsule. */
function ToolbarAction(props: { icon: IconName; label: string; onPress: () => void }) {
  return liquidGlass ? (
    <IconButton icon={props.icon} label={props.label} size="md" iconSize={16} class="rounded-full hover:bg-hover/60" onPress={props.onPress} />
  ) : (
    <IconButton icon={props.icon} label={props.label} size="toolbar" onPress={props.onPress} />
  );
}

/** The sidebar: the libraries, pinned first as the registry orders them. */
function Sidebar(props: { wb: WorkbenchModel; onAdd: () => void; onEdit: (library: LibraryDto) => void }) {
  const renaming = signal<string | null>(null);
  const draftName = signal("");
  const renameError = signal<string | null>(null);
  const savingName = signal(false);

  const startRename = (library: LibraryDto) => {
    draftName.set(library.name);
    renameError.set(null);
    renaming.set(library.uid);
  };

  const saveName = async (library: LibraryDto) => {
    if (renaming() !== library.uid || savingName()) return;
    const name = draftName().trim();
    if (name === "") {
      renameError.set(tr("文献库名称不能为空"));
      return;
    }
    if (name === library.name) {
      renaming.set(null);
      return;
    }
    savingName.set(true);
    try {
      await Catalog.update(library.name, name, null, null);
      renaming.set(null);
      if (props.wb.libraryUid() === library.uid) savePreference({ library: name });
    } catch (e) {
      renameError.set(errorText(e));
    } finally {
      savingName.set(false);
    }
  };

  const entries = (library: LibraryDto): MenuEntry[] => [
    { label: tr("编辑"), icon: "settings", onSelect: () => props.onEdit(library) },
    { label: tr("重命名"), icon: "rename", onSelect: () => startRename(library) },
    { label: library.pinned ? tr("取消置顶") : tr("置顶"), icon: "pin", onSelect: () => void props.wb.setPinned(library, !library.pinned) },
    { kind: "separator" },
    { label: tr("打开文件"), icon: "folderOpen", onSelect: () => void Files.openLibrary(library.uid).catch(showError) },
    { label: tr("移除"), icon: "x", destructive: true, onSelect: () => void props.wb.removeLibrary(library) },
  ];

  return (
    <View class="flex-1">
      <View class="flex-row items-center px-4 py-3">
        <UIText variant="headline" class="flex-1">
          {tr("文献库")}
        </UIText>
        <IconButton icon="folderAdd" label={tr("新增文献库")} iconSize={18} onPress={props.onAdd} />
      </View>
      <If
        when={libraryList().length > 0}
        else={<EmptyState title={tr("暂无文献库")} />}
      >
        <List each={libraryList()} keyExtractor={(l) => l.uid} estimatedItemSize={40} class="flex-1" contentClass="gap-0.5 pb-2">
          {(library) => (
            <ContextMenu
              entries={entries(library())}
              trigger={
                <View class="group">
                  <ListRow
                    variant="library"
                    selected={props.wb.libraryUid() === library().uid}
                    onPress={() => renaming() !== library().uid && props.wb.selectLibrary(library().uid)}
                    accessibilityLabel={library().name}
                    leading={
                      <View class="pt-0.5">
                        <Icon name="library" size={16} tone="secondary" />
                      </View>
                    }
                    trailing={
                      <View class="flex-row items-center gap-1.5 self-center">
                        <If when={!library().available}>
                          <UIText variant="caption" tone="secondary">
                            {tr("不可用")}
                          </UIText>
                        </If>
                        <If when={library().pinned}>
                          <Icon name="pin" size={13} tone="secondary" accessibilityLabel={tr("已置顶")} />
                        </If>
                        <View class={cx("opacity-0 group-hover:opacity-100 focus-within:opacity-100")}>
                          <ActionMenu
                            entries={entries(library())}
                            trigger={
                              <View class="w-6 h-6 items-center justify-center rounded-(--radius-keycap) hover:bg-hover" accessibilityLabel={tr("{0}的操作", library().name)}>
                                <Icon name="more" size={16} tone="secondary" />
                              </View>
                            }
                          />
                        </View>
                      </View>
                    }
                  >
                    <If
                      when={renaming() === library().uid}
                      else={
                        <UIText tone={library().available ? "primary" : "secondary"} lines={1}>
                          {library().name}
                        </UIText>
                      }
                    >
                      <Input
                        value={draftName()}
                        accessibilityLabel={tr("文献库名称")}
                        autoFocus
                        readonly={savingName()}
                        class={renameClass}
                        onChange={({ value }) => draftName.set(value)}
                        onSubmit={() => void saveName(library())}
                        onBlur={() => void saveName(library())}
                        onKeyDown={(e) => {
                          if (e.key === "Escape" && !savingName()) renaming.set(null);
                        }}
                        captureKeys={["escape"]}
                      />
                      <If when={renameError()}>
                        {(message) => (
                          <UIText variant="caption" tone="danger">
                            {message()}
                          </UIText>
                        )}
                      </If>
                    </If>
                    <If when={library().description}>
                      {(description) => (
                        <UIText variant="caption" tone="secondary" lines={1}>
                          {description()}
                        </UIText>
                      )}
                    </If>
                  </ListRow>
                </View>
              }
            />
          )}
        </List>
      </If>
    </View>
  );
}

/** The middle pane: library header, search, filters, results. */
function References(props: {
  wb: WorkbenchModel;
  onTags: (reference: ReferenceDto) => void;
  onBatchTags: () => void;
  onAdvancedEditor: () => void;
  registerSearch: (ref: InputRef) => void;
}) {
  let list: ListRef | undefined;
  // Keep the keyboard's row in view.
  effect(() => {
    const key = props.wb.selected();
    if (!key || !list) return;
    const index = untracked(props.wb.references).findIndex((r) => referenceKey(r) === key);
    if (index >= 0) list.scrollToIndex({ index, animated: false, viewPosition: 0.5 });
  });

  return (
    <View class="flex-1">
      <If when={libraryList().length > 0}>
        <View class="p-4 gap-3">
          <View class="flex-row items-center gap-2">
            <UIText variant="title3" weight="semibold" lines={1} class="flex-1">
              {props.wb.library()?.name ?? tr("文献工作台")}
            </UIText>
            <UIText variant="caption" tone="secondary">
              {trReferences(props.wb.references().length)}
            </UIText>
            <If when={props.wb.loading()}>
              <Spinner />
            </If>
          </View>
          <SearchField
            value={props.wb.query()}
            onChange={props.wb.setQuery}
            placeholder={tr("搜索文献")}
            onMove={props.wb.move}
            onCompositionChange={props.wb.setComposing}
            inputRef={props.registerSearch}
          />
          <View class="flex-row flex-wrap items-center gap-2">
            <Switch label={tr("高级搜索")} checked={props.wb.advanced()} onChange={props.wb.setAdvanced} />
            <If when={props.wb.advanced()}>
              <Button label={tr("搜索条件")} size="sm" onPress={props.onAdvancedEditor} />
            </If>
            <Select
              accessibilityLabel={tr("类型筛选")}
              value={props.wb.type()}
              options={typeOptions()}
              onChange={props.wb.setType}
            />
            <Select
              accessibilityLabel={tr("字段筛选")}
              value={props.wb.field()}
              options={fieldOptions()}
              onChange={props.wb.setField}
            />
            <TagFilterButton filter={props.wb.tagFilter()} onApply={props.wb.setTagFilter} />
          </View>
          <Diagnostics wb={props.wb} />
        </View>
        <Separator />
      </If>
      <If
        when={props.wb.references().length > 0 || props.wb.loading()}
        else={
          <EmptyState
            title={tr("暂无可显示的文献")}
            mark={
              <If when={libraryList().length === 0}>
                <Image src={logo} accessibilityLabel="BibCiTeX" style={{ width: metrics.emptyLogo, height: metrics.emptyLogo }} />
              </If>
            }
          />
        }
      >
        <View class="flex-1">
          <List
            ref={(r: ListRef) => (list = r)}
            each={props.wb.references()}
            keyExtractor={referenceKey}
            estimatedItemSize={76}
            class="flex-1"
            contentClass="py-1.5 gap-0.5"
          >
            {(item) => <ResultRow wb={props.wb} reference={item()} variant="workbench" onTags={props.onTags} />}
          </List>
          <If when={props.wb.batchActive()}>
            <View class="absolute bottom-3 left-0 right-0 items-center" style={{ pointerEvents: "box-none" }}>
              <Surface variant="capsule">
                <BatchActions wb={props.wb} onBatchTags={props.onBatchTags} />
              </Surface>
            </View>
          </If>
        </View>
      </If>
    </View>
  );
}

/** The inspector: the selected reference's title, fields and actions. */
function Inspector(props: { wb: WorkbenchModel; onTags: (reference: ReferenceDto) => void }) {
  return (
    <View class="flex-1">
      <View class="p-4">
        <If when={props.wb.reference()} else={<UIText variant="title3">{tr("文献详情")}</UIText>}>
          {(reference) => <TitleText reference={reference()} variant="title3" />}
        </If>
      </View>
      <Separator />
      <If
        when={props.wb.reference()}
        else={<EmptyState title={tr("选择一条文献查看字段和操作")} />}
      >
        {(reference) => <ReferenceDetail wb={props.wb} reference={reference()} full onTags={props.onTags} />}
      </If>
    </View>
  );
}

/**
 * The main window: a toolbar over three resizable panes. Below 900pt the
 * sidebar, below 1280pt the inspector, move behind toolbar popovers.
 */
export function Workbench(props: { wb: WorkbenchModel }) {
  const wb = props.wb;
  // The window's hidden title bar: its height and the room the system's
  // window controls take (traffic lights left, Windows / GTK buttons right).
  const win = currentWindow();
  const sidebarWidth = signal<number>(metrics.sidebar.ideal);
  const inspectorWidth = signal<number>(metrics.inspector.ideal);
  const compactSidebar = signal(false);
  const editorOpen = signal(false);
  const editing = signal<LibraryDto | null>(null);
  const advancedEditor = signal(false);
  const tagManager = signal(false);
  const tagTarget = signal<TagTarget | null>(null);
  const batchTargets = signal<readonly TagTargetDto[] | null>(null);
  let search: InputRef | undefined;

  const sidebarShown = () => prefs()?.sidebar ?? true;
  const inspectorShown = () => prefs()?.inspector ?? false;
  const layout = () => paneLayout(sidebarShown(), inspectorShown());
  const updateBadge = () => {
    const phase = updateState()?.phase;
    return phase === "available" || phase === "ready";
  };

  // Close the compact sidebar once the window is wide enough to dock it.
  effect(() => {
    if (windowSize().width >= metrics.compactSidebarBelow) untracked(() => compactSidebar.set(false));
  });

  // Escape ends batch selection once overlays and editing have had it.
  effect(() => {
    if (!wb.batchActive()) return;
    const stop = keymap.bind("escape", () => wb.setBatchActive(false));
    cleanup(stop);
  });
  keymap.bind("mod+f", () => search?.focus());

  const openTags = (reference: ReferenceDto) =>
    tagTarget.set({ libraryUid: reference.libraryUid, recordUid: reference.recordUid, citeKey: reference.citeKey });

  const openEditor = (library: LibraryDto | null) => {
    editing.set(library);
    editorOpen.set(true);
  };

  const sidebar = () => <Sidebar wb={wb} onAdd={() => openEditor(null)} onEdit={(l) => openEditor(l)} />;
  const inspector = () => <Inspector wb={wb} onTags={openTags} />;

  return (
    <View class="flex-1 bg-background">
      <View
        class="flex-row items-center border-b border-separator bg-background"
        style={{
          height: Math.max(toolbarHeight, win.titleBar().height),
          paddingLeft: win.titleBar().left > 0 ? win.titleBar().left + 8 : 12,
          paddingRight: win.titleBar().right > 0 ? win.titleBar().right + 12 : 12,
        }}
        desktop={{ windowDrag: true }}
      >
        <View class="flex-row items-center gap-2.5" desktop={{ windowDrag: true }}>
          <If
            when={layout().sidebarCompact}
            else={
              <PaneToggle
                icon={sidebarShown() ? "panelLeftClose" : "panelLeftOpen"}
                label={tr("文献库")}
                onPress={() => savePreference({ sidebar: !sidebarShown() })}
              />
            }
          >
            <View desktop={{ windowDrag: false }}>
              <PopoverFrame
                open={compactSidebar()}
                onOpenChange={compactSidebar.set}
                width={metrics.sidebar.popoverWidth}
                maxHeight={Math.min(metrics.popoverMaxHeight, windowSize().height)}
                trigger={<TriggerGlyph icon={compactSidebar() ? "panelLeftClose" : "panelLeftOpen"} label={tr("文献库")} selected={compactSidebar()} />}
              >
                <View class="-m-3.5" style={{ height: Math.min(metrics.popoverMaxHeight, windowSize().height) - 16 }}>
                  {sidebar()}
                </View>
              </PopoverFrame>
            </View>
          </If>
          <Image src={logo} accessibilityLabel="BibCiTeX" style={{ width: metrics.logo, height: metrics.logo }} />
        </View>
        <View class="flex-1 self-stretch" desktop={{ windowDrag: true }} />
        <View
          class={liquidGlass ? "h-9 px-1 rounded-full flex-row items-center gap-0.5" : "flex-row items-center gap-1"}
          style={glass}
          desktop={{ windowDrag: false }}
        >
          <BatchToggle wb={wb} round={liquidGlass} />
          <ToolbarAction icon="tag" label={tr("标签管理")} onPress={() => tagManager.set(true)} />
          <ToolbarAction icon="search" label={tr("快捷助手")} onPress={() => void showHelper()} />
          <ToolbarAction icon="refresh" label={tr("刷新")} onPress={wb.refresh} />
          <View>
            <ToolbarAction icon="settings" label={tr("设置")} onPress={() => setSettingsOpen(true)} />
            <If when={updateBadge()}>
              <View class="absolute top-0.5 right-0.5 w-2 h-2 rounded-full bg-primary" style={{ pointerEvents: "none" }} />
            </If>
          </View>
        </View>
        <View class="w-2.5" />
        <If
          when={layout().inspectorCompact}
          else={
            <PaneToggle
              icon={inspectorShown() ? "panelRightClose" : "panelRightOpen"}
              label={tr("文献详情")}
              onPress={() => savePreference({ inspector: !inspectorShown() })}
            />
          }
        >
          <View desktop={{ windowDrag: false }}>
            <PopoverFrame
              open={inspectorShown()}
              onOpenChange={(open) => savePreference({ inspector: open })}
              width={metrics.inspector.popoverWidth}
              maxHeight={Math.min(metrics.popoverMaxHeight, windowSize().height)}
              trigger={<TriggerGlyph icon={inspectorShown() ? "panelRightClose" : "panelRightOpen"} label={tr("文献详情")} selected={inspectorShown()} />}
            >
              <View class="-m-3.5" style={{ height: Math.min(metrics.popoverMaxHeight, windowSize().height) - 16 }}>
                {inspector()}
              </View>
            </PopoverFrame>
          </View>
        </If>
      </View>
      <SplitPanes
        sidebarShown={sidebarShown()}
        inspectorShown={inspectorShown()}
        sidebarWidth={sidebarWidth()}
        inspectorWidth={inspectorWidth()}
        onSidebarResize={sidebarWidth.set}
        onInspectorResize={inspectorWidth.set}
        sidebar={sidebar()}
        content={
          <References
            wb={wb}
            onTags={openTags}
            onBatchTags={() => batchTargets.set(wb.selectedReferences().map((r) => ({ libraryUid: r.libraryUid, recordUid: r.recordUid })))}
            onAdvancedEditor={() => advancedEditor.set(true)}
            registerSearch={(r) => (search = r)}
          />
        }
        inspector={inspector()}
      />
      <LibraryEditor
        open={editorOpen()}
        library={editing()}
        onClose={() => editorOpen.set(false)}
        onSaved={(saved) => {
          if (!editing()) wb.selectLibrary(saved.uid);
          else if (wb.libraryUid() === saved.uid) savePreference({ library: saved.name });
        }}
      />
      <AdvancedSearchEditor
        open={advancedEditor()}
        text={wb.query()}
        document={wb.advancedDraft().document}
        onApply={wb.applyAdvancedQuery}
        onClose={() => advancedEditor.set(false)}
      />
      <TagManager open={tagManager()} target={null} onClose={() => tagManager.set(false)} />
      <TagManager open={tagTarget() !== null} target={tagTarget()} onClose={() => tagTarget.set(null)} />
      <BatchTagEditor open={batchTargets() !== null} targets={batchTargets() ?? []} onClose={() => batchTargets.set(null)} />
    </View>
  );
}
