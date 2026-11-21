import type { ListRef } from "nsh";
import { effect, List, signal, untracked, View } from "nsh";
import type { ReferenceDto, TagTargetDto } from "bibcitex/native";
import { tr, trReferences } from "../i18n";
import { tray as metrics } from "../theme";
import { libraryList } from "../store/desktop";
import { fieldOptions, referenceKey, typeOptions } from "../store/format";
import { hideTray, showMain } from "../store/surfaces";
import type { Workbench } from "../store/workbench";
import { Button, IconButton, SearchField, Select, Separator, Spinner, Surface, Switch, UIText, Icon } from "../ui";
import { AdvancedSearchEditor } from "./AdvancedSearchEditor";
import { BatchActions, BatchToggle, Diagnostics, ReferenceDetail, ResultRow, TitleText } from "./shared";
import { BatchTagEditor, TagManager, type TagTarget } from "./TagDialogs";
import { TagFilterButton } from "./TagFilterButton";

/**
 * The tray workbench (680×580): its own search, library, filters and batch
 * selection over a list, with a 282pt detail sheet that slides over the
 * results' right edge.
 */
export function TrayPanel(props: { wb: Workbench }) {
  const wb = props.wb;
  const detailVisible = signal(false);
  const advancedEditor = signal(false);
  const tagManager = signal(false);
  const tagTarget = signal<TagTarget | null>(null);
  const batchTargets = signal<readonly TagTargetDto[] | null>(null);
  let list: ListRef | undefined;

  effect(() => {
    const key = wb.selected();
    if (!key || !list) return;
    const index = untracked(wb.references).findIndex((r) => referenceKey(r) === key);
    if (index >= 0) list.scrollToIndex({ index, animated: false, viewPosition: 0.5 });
  });

  const openTags = (reference: ReferenceDto) =>
    tagTarget.set({ libraryUid: reference.libraryUid, recordUid: reference.recordUid, citeKey: reference.citeKey });

  const libraryOptions = () =>
    libraryList().map((l) => ({
      value: l.uid,
      title: l.available ? l.name : `${l.name} · ${tr("不可用")}`,
      disabled: !l.available,
    }));

  return (
    <Surface variant="trayPanel" class="flex-1">
      <View class="p-4 gap-3">
        <View class="flex-row items-center gap-2.5">
          <UIText variant="headline" class="flex-1">
            BibCiTeX
          </UIText>
          <Surface variant="capsule" class="flex-row items-center gap-2 p-1">
            <BatchToggle wb={wb} />
            <IconButton icon="tag" label={tr("标签管理")} size="sm" iconSize={16} onPress={() => tagManager.set(true)} />
            <IconButton
              icon={detailVisible() ? "panelRightClose" : "panelRightOpen"}
              label={tr("文献详情")}
              size="sm"
              iconSize={16}
              selected={detailVisible()}
              onPress={() => detailVisible.set(!detailVisible())}
            />
            <IconButton icon="refresh" label={tr("刷新")} size="sm" iconSize={16} onPress={wb.refresh} />
            <IconButton
              icon="externalLink"
              label={tr("显示窗口")}
              size="sm"
              iconSize={16}
              onPress={() => {
                hideTray();
                showMain();
              }}
            />
            <IconButton icon="x" label={tr("关闭")} size="sm" iconSize={16} onPress={hideTray} />
          </Surface>
        </View>
        <SearchField
          size="tray"
          value={wb.query()}
          onChange={wb.setQuery}
          placeholder={tr("搜索文献")}
          onMove={wb.move}
          onCompositionChange={wb.setComposing}
          autoFocus
          trailing={
            <View class="flex-row items-center gap-2.5">
              <Separator vertical class="h-4" />
              <Select
                class="w-37.5"
                accessibilityLabel={tr("文献库")}
                placeholder={tr("暂无文献库")}
                disabled={libraryList().length === 0}
                value={wb.libraryUid()}
                options={libraryOptions()}
                leading={<Icon name="library" size={13} tone="secondary" />}
                onChange={wb.selectLibrary}
              />
            </View>
          }
        />
        <View class="flex-row flex-wrap items-center gap-2">
          <Switch label={tr("高级搜索")} checked={wb.advanced()} onChange={wb.setAdvanced} />
          <If when={wb.advanced()}>
            <Button label={tr("搜索条件")} size="sm" onPress={() => advancedEditor.set(true)} />
          </If>
          <Select accessibilityLabel={tr("类型筛选")} value={wb.type()} options={typeOptions()} onChange={wb.setType} />
          <Select accessibilityLabel={tr("字段筛选")} value={wb.field()} options={fieldOptions()} onChange={wb.setField} />
          <TagFilterButton filter={wb.tagFilter()} onApply={wb.setTagFilter} />
          <View class="flex-1" />
          <If when={wb.loading()}>
            <Spinner />
          </If>
          <UIText variant="caption" tone="secondary">
            {trReferences(wb.references().length)}
          </UIText>
        </View>
        <Diagnostics wb={wb} />
      </View>
      <Separator />
      <View class="flex-1">
        <If
          when={wb.references().length > 0 || wb.loading()}
          else={
            <View class="flex-1 items-center justify-center">
              <UIText variant="callout" tone="secondary">
                {tr("暂无可显示的文献")}
              </UIText>
            </View>
          }
        >
          <List
            ref={(r: ListRef) => (list = r)}
            each={wb.references()}
            keyExtractor={referenceKey}
            estimatedItemSize={70}
            class="flex-1"
            contentClass="py-1.5 gap-0.5"
          >
            {(item) => <ResultRow wb={wb} reference={item()} variant="tray" onTags={openTags} />}
          </List>
        </If>
        <If when={wb.batchActive()}>
          <View class="absolute bottom-3 left-0 right-0 items-center" style={{ pointerEvents: "box-none" }}>
            <Surface variant="capsule">
              <BatchActions
                wb={wb}
                onBatchTags={() => batchTargets.set(wb.selectedReferences().map((r) => ({ libraryUid: r.libraryUid, recordUid: r.recordUid })))}
              />
            </Surface>
          </View>
        </If>
        <If when={detailVisible()}>
          <View class="absolute top-3 right-3 bottom-3" style={{ width: metrics.detailWidth }}>
            <Surface variant="detail" class="flex-1">
              <View class="flex-row items-center gap-2 px-3.5 py-2">
                <View class="flex-1 min-w-0">
                  <If when={wb.reference()} else={<UIText variant="headline">{tr("文献详情")}</UIText>}>
                    {(reference) => <TitleText reference={reference()} weight="semibold" lines={2} />}
                  </If>
                </View>
                <IconButton icon="x" label={tr("关闭")} size="sm" onPress={() => detailVisible.set(false)} />
              </View>
              <Separator />
              <If
                when={wb.reference()}
                else={
                  <View class="flex-1 items-center justify-center p-4">
                    <UIText variant="callout" tone="secondary">
                      {tr("选择一条文献查看字段和操作")}
                    </UIText>
                  </View>
                }
              >
                {(reference) => <ReferenceDetail wb={wb} reference={reference()} full={false} onTags={openTags} />}
              </If>
            </Surface>
          </View>
        </If>
      </View>
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
    </Surface>
  );
}
