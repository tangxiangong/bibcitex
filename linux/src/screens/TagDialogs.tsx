import { batch, effect, Scroll, signal, untracked, View } from "nsh";
import { AlertDialog, Button as BaseButton, Input } from "nsh/base";
import {
  Files,
  SearchSession,
  Tags,
  type BatchTagSnapshotDto,
  type ReferenceDto,
  type TagActionDto,
  type TagDto,
  type TagSnapshotDto,
  type TagTargetDto,
} from "bibcitex/native";
import { tr } from "../i18n";
import { dialogs } from "../theme";
import { libraryList } from "../store/desktop";
import { errorText } from "../store/format";
import { ActionMenu, Button, Checkbox, DialogFrame, Icon, PopoverFrame, Select, Spinner, UIText, cx, withAlpha } from "../ui";

/** The record a tag dialog assigns to, captured when it opened. */
export interface TagTarget {
  libraryUid: string;
  recordUid: string;
  citeKey: string;
}

/** The preset tag colors of the macOS client. Tag colors are user data
 * (shown as they are), so these are values, not theme roles. */
const presetColors = ["#64748B", "#EF4444", "#F59E0B", "#22C55E", "#14B8A6", "#3B82F6", "#8B5CF6", "#EC4899"] as const;

const inputClass =
  "h-7 px-2.5 rounded-(--radius-control) border border-input bg-background text-(length:--fs-body) text-foreground placeholder:text-muted-foreground focus:border-primary";

const sortByName = (a: TagDto, b: TagDto) => a.name.localeCompare(b.name);

function ColorDot(props: { color: string; size: "sm" | "md" }) {
  return (
    <View
      class={cx("rounded-full border border-foreground/25", props.size === "md" ? "w-3.5 h-3.5" : "w-3 h-3")}
      style={{ backgroundColor: props.color }}
    />
  );
}

/** The color popover of a tag: the preset swatches and a custom value. */
function ColorEditor(props: { tag: TagDto; onSave: (color: string) => void }) {
  const open = signal(false);
  const custom = signal(props.tag.color);
  const valid = () => /^#[0-9a-f]{6}$/i.test(custom().trim());
  const save = (color: string) => {
    open.set(false);
    props.onSave(color.toUpperCase());
  };
  return (
    <PopoverFrame
      open={open()}
      onOpenChange={open.set}
      width={240}
      trigger={
        <View class="w-6 h-6 items-center justify-center rounded-(--radius-keycap) hover:bg-hover" accessibilityLabel={`${tr("标签颜色")} ${props.tag.name}`}>
          <ColorDot color={props.tag.color} size="md" />
        </View>
      }
    >
      <View class="gap-2.5">
        <View class="flex-row gap-1.5">
          <Each of={presetColors}>
            {(color) => (
              <BaseButton class="w-5 h-5 rounded-full items-center justify-center" accessibilityLabel={color()} onPress={() => save(color())}>
                <View class="w-5 h-5 rounded-full border border-foreground/25" style={{ backgroundColor: color() }} />
              </BaseButton>
            )}
          </Each>
        </View>
        <UIText variant="callout" tone="secondary">
          {tr("自定义颜色")}
        </UIText>
        <View class="flex-row items-center gap-2">
          <View class="w-5 h-5 rounded-full border border-foreground/25" style={{ backgroundColor: valid() ? custom() : withAlpha(props.tag.color, 0.3) }} />
          <Input
            value={custom()}
            accessibilityLabel={tr("自定义颜色")}
            autoCapitalize="none"
            autoCorrect={false}
            class={cx(inputClass, "flex-1")}
            onChange={({ value }) => custom.set(value)}
            onSubmit={() => valid() && save(custom().trim())}
          />
          <Button label={tr("保存")} variant="primary" size="sm" disabled={!valid()} onPress={() => save(custom().trim())} />
        </View>
      </View>
    </PopoverFrame>
  );
}

/**
 * 标签管理 (catalog maintenance, 460×440) or 标签 for one record (assignment,
 * 360×340): search or create, assign and unassign, rename, recolor and, in
 * the catalog, merge, delete and repair broken associations.
 */
export function TagManager(props: { open: boolean; target: TagTarget | null; onClose: () => void }) {
  const snapshot = signal<TagSnapshotDto | null>(null);
  const filter = signal("");
  const editingId = signal<string | null>(null);
  const editName = signal("");
  const mergeSource = signal<TagDto | null>(null);
  const mergeTarget = signal<string | null>(null);
  const error = signal<string | null>(null);
  const busy = signal(false);
  const recovery = signal(false);
  let loadVersion = 0;

  const assignment = () => props.target !== null;

  const reload = async () => {
    const version = ++loadVersion;
    const target = untracked(() => props.target);
    try {
      const next = target ? await Tags.snapshot({ libraryUid: target.libraryUid, recordUid: target.recordUid }) : await Tags.catalog();
      if (version === loadVersion) snapshot.set(next);
    } catch (e) {
      if (version === loadVersion) error.set(errorText(e));
    }
  };

  effect(() => {
    const open = props.open;
    void props.target;
    if (!open) return;
    untracked(() => {
      batch(() => {
        snapshot.set(null);
        filter.set("");
        editingId.set(null);
        mergeSource.set(null);
        error.set(null);
      });
      void reload();
    });
  });

  const assigned = (id: string) => snapshot()?.assignedIds.includes(id) ?? false;
  const filtered = () => {
    const q = filter().trim().toLowerCase();
    const tags = (snapshot()?.tags ?? []).filter((t) => q === "" || t.name.toLowerCase().includes(q));
    return [...tags].sort((a, b) => (assigned(a.id) !== assigned(b.id) ? (assigned(a.id) ? -1 : 1) : sortByName(a, b)));
  };
  const canCreate = () => {
    const value = filter().trim();
    return value !== "" && !(snapshot()?.tags ?? []).some((t) => t.name.toLowerCase() === value.toLowerCase());
  };

  const mutate = async (action: TagActionDto, fields: { id?: string; name?: string; color?: string | null; targetId?: string } = {}) => {
    const state = snapshot();
    if (busy() || !state) return;
    busy.set(true);
    try {
      const next = await Tags.mutate({
        libraryUid: state.libraryUid,
        recordUid: state.recordUid,
        color: fields.color ?? null,
        action,
        id: fields.id ?? "",
        name: fields.name ?? "",
        targetId: fields.targetId ?? "",
        path: "",
        citeKey: "",
        expectedRevision: state.revision,
      });
      batch(() => {
        snapshot.set(next);
        error.set(null);
        if (action === "create") filter.set("");
        if (action === "rename") editingId.set(null);
        if (action === "merge") mergeSource.set(null);
      });
    } catch (e) {
      error.set(errorText(e));
      await reload();
    } finally {
      busy.set(false);
    }
  };

  const confirmDelete = async (tag: TagDto) => {
    const { action } = await AlertDialog.show({
      title: tr("删除标签"),
      description: tag.name,
      actions: [
        { id: "cancel", label: tr("取消"), role: "cancel" },
        { id: "delete", label: tr("删除"), role: "destructive" },
      ],
      closeOnEscape: true,
    });
    if (action === "delete") void mutate("delete", { id: tag.id });
  };

  const toggle = (tag: TagDto) => void mutate(assigned(tag.id) ? "unassign" : "assign", { id: tag.id });

  return (
    <DialogFrame
      open={props.open}
      onOpenChange={(open) => !open && props.onClose()}
      width={assignment() ? dialogs.tagAssignment.width : dialogs.tagManager.width}
      height={assignment() ? dialogs.tagAssignment.height : dialogs.tagManager.height}
      busy={busy()}
    >
      <View class="flex-1 gap-2.5 -m-3.5">
        <View class="flex-row items-center gap-2">
          <UIText variant="headline">{assignment() ? tr("标签") : tr("标签管理")}</UIText>
          <If when={props.target}>
            {(t) => (
              <UIText variant="caption" tone="secondary" lines={1} class="flex-1">
                {t().citeKey}
              </UIText>
            )}
          </If>
          <View class="flex-1" />
          <If when={busy()}>
            <Spinner />
          </If>
          <Button label={tr("关闭")} size="sm" onPress={props.onClose} />
        </View>
        <Input
          value={filter()}
          placeholder={tr("搜索或创建标签")}
          accessibilityLabel={tr("搜索或创建标签")}
          autoFocus
          class={inputClass}
          onChange={({ value }) => filter.set(value)}
          onSubmit={() => canCreate() && void mutate("create", { name: filter().trim() })}
        />
        <Scroll class="flex-1">
          <Each of={filtered()} key={(t) => t.id}>
            {(tag) => (
              <View class="flex-row items-center gap-2 py-1">
                <If when={assignment()}>
                  <Checkbox checked={assigned(tag().id)} accessibilityLabel={tag().name} onChange={() => toggle(tag())} />
                </If>
                <ColorEditor tag={tag()} onSave={(color) => void mutate("recolor", { id: tag().id, color })} />
                <If
                  when={editingId() === tag().id}
                  else={
                    <BaseButton class="flex-1 min-w-0 rounded-(--radius-keycap) px-1" accessibilityLabel={tag().name} onPress={() => assignment() && toggle(tag())}>
                      <UIText lines={1}>{tag().name}</UIText>
                    </BaseButton>
                  }
                >
                  <Input
                    value={editName()}
                    accessibilityLabel={tr("标签名称")}
                    autoFocus
                    class={cx(inputClass, "flex-1")}
                    onChange={({ value }) => editName.set(value)}
                    onSubmit={() => void mutate("rename", { id: tag().id, name: editName().trim() })}
                  />
                  <Button label={tr("保存")} size="sm" onPress={() => void mutate("rename", { id: tag().id, name: editName().trim() })} />
                </If>
                <ActionMenu
                  trigger={
                    <View class="w-7 h-6 items-center justify-center rounded-(--radius-keycap) hover:bg-hover" accessibilityLabel={`${tr("标签管理")} ${tag().name}`}>
                      <Icon name="more" size={14} tone="secondary" />
                    </View>
                  }
                  entries={[
                    {
                      label: tr("重命名"),
                      icon: "rename",
                      onSelect: () => {
                        editName.set(tag().name);
                        editingId.set(tag().id);
                      },
                    },
                    ...(assignment()
                      ? []
                      : [
                          {
                            label: tr("合并到"),
                            onSelect: () => {
                              mergeSource.set(tag());
                              mergeTarget.set(null);
                            },
                          },
                          { kind: "separator" as const },
                          { label: tr("删除"), icon: "trash" as const, destructive: true, onSelect: () => void confirmDelete(tag()) },
                        ]),
                  ]}
                />
              </View>
            )}
          </Each>
          <If when={canCreate()}>
            <BaseButton
              class="flex-row items-center gap-1.5 py-1 px-1 rounded-(--radius-keycap) hover:bg-hover"
              accessibilityLabel={assignment() ? tr("创建并添加") : tr("创建")}
              onPress={() => void mutate("create", { name: filter().trim() })}
            >
              <Icon name="add" size={12} tone="accent" />
              <UIText tone="accent" lines={1}>
                {`${assignment() ? tr("创建并添加") : tr("创建")} “${filter().trim()}”`}
              </UIText>
            </BaseButton>
          </If>
        </Scroll>
        <If when={mergeSource()}>
          {(source) => (
            <View class="flex-row items-center gap-2">
              <UIText variant="callout" lines={1}>
                {`${tr("合并到")} · ${source().name} →`}
              </UIText>
              <Select
                class="flex-1"
                accessibilityLabel={tr("目标标签")}
                placeholder={tr("目标标签")}
                value={mergeTarget()}
                options={(snapshot()?.tags ?? []).filter((t) => t.id !== source().id).map((t) => ({ value: t.id, title: t.name }))}
                onChange={(value) => mergeTarget.set(value)}
              />
              <Button label={tr("取消")} size="sm" onPress={() => mergeSource.set(null)} />
              <Button
                label={tr("确认")}
                variant="primary"
                size="sm"
                disabled={!mergeTarget()}
                onPress={() => void mutate("merge", { id: source().id, targetId: mergeTarget() ?? "" })}
              />
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
        <If when={!assignment() && (snapshot()?.pending.length ?? 0) + (snapshot()?.sources.filter((s) => !s.available).length ?? 0) > 0}>
          <Button class="self-start" label={tr("失效标签关联")} size="sm" onPress={() => recovery.set(true)} />
        </If>
      </View>
      <TagRecovery
        open={recovery()}
        onClose={() => {
          recovery.set(false);
          void reload();
        }}
      />
    </DialogFrame>
  );
}

/** 批量标签 (460×340): what the selection shares, then add or remove for all. */
export function BatchTagEditor(props: { open: boolean; targets: readonly TagTargetDto[]; onClose: () => void }) {
  const snapshot = signal<BatchTagSnapshotDto | null>(null);
  const filter = signal("");
  const error = signal<string | null>(null);
  const busy = signal(false);

  const reload = async () => {
    try {
      snapshot.set(await Tags.batchSnapshot([...untracked(() => props.targets)]));
    } catch (e) {
      batch(() => {
        snapshot.set(null);
        error.set(errorText(e));
      });
    }
  };

  effect(() => {
    if (!props.open) return;
    void props.targets;
    untracked(() => {
      batch(() => {
        snapshot.set(null);
        filter.set("");
        error.set(null);
      });
      void reload();
    });
  });

  const state = (id: string) =>
    snapshot()?.assignedIds.includes(id) ? "all" : snapshot()?.partialIds.includes(id) ? "some" : "none";

  const mutate = async (action: TagActionDto, id = "", name = "") => {
    const current = snapshot();
    if (busy() || !current) return;
    busy.set(true);
    try {
      const next = await Tags.mutateBatch({ targets: [...props.targets], action, id, name, expectedRevision: current.revision });
      batch(() => {
        snapshot.set(next);
        error.set(null);
        if (action === "create") filter.set("");
      });
    } catch (e) {
      error.set(errorText(e));
      await reload();
    } finally {
      busy.set(false);
    }
  };

  const filtered = () => {
    const q = filter().trim().toLowerCase();
    return (snapshot()?.tags ?? []).filter((t) => q === "" || t.name.toLowerCase().includes(q));
  };

  return (
    <DialogFrame
      open={props.open}
      onOpenChange={(open) => !open && props.onClose()}
      width={dialogs.batchTags.width}
      height={dialogs.batchTags.height}
      busy={busy()}
    >
      <View class="flex-1 gap-2.5 -m-3.5">
        <View class="flex-row items-center gap-2">
          <UIText variant="headline">{tr("批量标签")}</UIText>
          <UIText tone="secondary">{`${tr("已选")} ${snapshot()?.count ?? props.targets.length}`}</UIText>
          <View class="flex-1" />
          <If when={busy()}>
            <Spinner />
          </If>
          <Button label={tr("关闭")} size="sm" onPress={props.onClose} />
        </View>
        <Input
          value={filter()}
          placeholder={tr("搜索或创建标签")}
          accessibilityLabel={tr("搜索或创建标签")}
          autoFocus
          class={inputClass}
          onChange={({ value }) => filter.set(value)}
        />
        <Scroll class="flex-1" contentClass="gap-2">
          <Each of={filtered()} key={(t) => t.id}>
            {(tag) => (
              <View class="flex-row items-center gap-2">
                <ColorDot color={tag().color} size="sm" />
                <UIText lines={1} class="flex-1">
                  {tag().name}
                </UIText>
                <UIText variant="caption" tone="secondary">
                  {state(tag().id) === "all" ? tr("全部已添加") : state(tag().id) === "some" ? tr("部分已添加") : tr("未添加")}
                </UIText>
                <Button label={tr("添加")} size="sm" disabled={state(tag().id) === "all"} onPress={() => void mutate("assign", tag().id)} />
                <Button label={tr("移除")} size="sm" disabled={state(tag().id) === "none"} onPress={() => void mutate("unassign", tag().id)} />
              </View>
            )}
          </Each>
          <If when={filter().trim() !== ""}>
            <Button class="self-start" label={tr("创建并添加")} size="sm" onPress={() => void mutate("create", "", filter().trim())} />
          </If>
        </Scroll>
        <If when={error()}>
          {(message) => (
            <UIText variant="caption" tone="danger" selectable>
              {message()}
            </UIText>
          )}
        </If>
      </View>
    </DialogFrame>
  );
}

/**
 * 失效标签关联 (420 wide): reconnect an absent record's tags to a record of a
 * registered library, or relocate a missing bibliography source. Nothing
 * is guessed: the user picks every identity.
 */
function TagRecovery(props: { open: boolean; onClose: () => void }) {
  const snapshot = signal<TagSnapshotDto | null>(null);
  const pendingId = signal<string | null>(null);
  const libraryUid = signal<string | null>(null);
  const recordUid = signal<string | null>(null);
  const records = signal<readonly ReferenceDto[]>([]);
  const sourceId = signal<string | null>(null);
  const error = signal<string | null>(null);
  const busy = signal(false);
  const finder = new SearchSession("main");
  let recordLoad = 0;

  const reload = async () => {
    try {
      snapshot.set(await Tags.catalog());
    } catch (e) {
      error.set(errorText(e));
    }
  };

  effect(() => {
    if (!props.open) return;
    untracked(() => {
      batch(() => {
        pendingId.set(null);
        libraryUid.set(null);
        recordUid.set(null);
        records.set([]);
        sourceId.set(null);
        error.set(null);
      });
      void reload();
    });
  });

  const chooseLibrary = async (uid: string) => {
    const version = ++recordLoad;
    batch(() => {
      libraryUid.set(uid);
      recordUid.set(null);
      records.set([]);
    });
    const lib = libraryList().find((l) => l.uid === uid);
    if (!lib) return;
    try {
      const reply = await finder.search({
        path: lib.path,
        mode: "ordinary",
        inputKind: "text",
        text: "",
        document: null,
        expectedTagRevision: null,
        defaultField: "all",
        typeFilter: "all",
        queryVersion: String(version),
      });
      if (version === recordLoad) records.set(reply.references);
    } catch (e) {
      if (version === recordLoad) error.set(errorText(e));
    }
  };

  const run = async (action: "relink" | "relocate", fields: { libraryUid?: string; recordUid?: string; id: string; path?: string }) => {
    if (busy()) return;
    busy.set(true);
    try {
      await reload();
      const revision = snapshot()?.revision;
      if (!revision) return;
      snapshot.set(
        await Tags.mutate({
          libraryUid: fields.libraryUid ?? "",
          recordUid: fields.recordUid ?? "",
          color: null,
          action,
          id: fields.id,
          name: "",
          targetId: "",
          path: fields.path ?? "",
          citeKey: "",
          expectedRevision: revision,
        }),
      );
      batch(() => {
        pendingId.set(null);
        sourceId.set(null);
        error.set(null);
      });
    } catch (e) {
      error.set(errorText(e));
    } finally {
      busy.set(false);
    }
  };

  const relocate = async () => {
    const id = sourceId();
    if (!id) return;
    try {
      const path = await Files.pickBibliography();
      if (path) await run("relocate", { id, path });
    } catch (e) {
      error.set(errorText(e));
    }
  };

  const missingSources = () => (snapshot()?.sources ?? []).filter((s) => !s.available);

  return (
    <DialogFrame open={props.open} onOpenChange={(open) => !open && props.onClose()} width={dialogs.invalidTagLinks} busy={busy()}>
      <View class="gap-3 -m-2.5">
        <UIText variant="headline">{tr("失效标签关联")}</UIText>
        <Select
          accessibilityLabel={tr("旧记录")}
          placeholder={tr("选择记录")}
          value={pendingId()}
          options={(snapshot()?.pending ?? []).map((p) => ({ value: p.recordId, title: p.citeKey }))}
          onChange={(value) => pendingId.set(value)}
        />
        <Select
          accessibilityLabel={tr("目标文献库")}
          placeholder={tr("选择文献库")}
          value={libraryUid()}
          options={libraryList()
            .filter((l) => l.available)
            .map((l) => ({ value: l.uid, title: l.name }))}
          onChange={(value) => void chooseLibrary(value)}
        />
        <Select
          accessibilityLabel={tr("目标记录")}
          placeholder={tr("选择记录")}
          value={recordUid()}
          options={records().map((r) => ({ value: r.recordUid, title: r.citeKey }))}
          onChange={(value) => recordUid.set(value)}
        />
        <Button
          class="self-start"
          label={tr("重新关联")}
          disabled={!pendingId() || !recordUid() || busy()}
          onPress={() => void run("relink", { libraryUid: libraryUid() ?? "", recordUid: recordUid() ?? "", id: pendingId() ?? "" })}
        />
        <If when={missingSources().length > 0}>
          <View class="gap-2">
            <UIText variant="callout" weight="medium">
              {tr("重新定位原文件")}
            </UIText>
            <View class="flex-row items-center gap-2">
              <Select
                class="flex-1"
                accessibilityLabel={tr("原文献来源")}
                placeholder={tr("选择来源")}
                value={sourceId()}
                options={missingSources().map((s) => ({ value: s.id, title: s.path }))}
                onChange={(value) => sourceId.set(value)}
              />
              <Button label={tr("选择文件")} disabled={!sourceId() || busy()} onPress={() => void relocate()} />
            </View>
          </View>
        </If>
        <If when={error()}>
          {(message) => (
            <UIText variant="caption" tone="danger" selectable>
              {message()}
            </UIText>
          )}
        </If>
        <View class="flex-row justify-end">
          <Button label={tr("关闭")} onPress={props.onClose} />
        </View>
      </View>
    </DialogFrame>
  );
}
