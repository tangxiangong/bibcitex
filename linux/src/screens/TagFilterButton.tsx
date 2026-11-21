import { batch, effect, signal, untracked, View } from "nsh";
import { Button as BaseButton, Input } from "nsh/base";
import { Tags, type TagDto } from "bibcitex/native";
import { tr } from "../i18n";
import { dialogs } from "../theme";
import { tagsChanged } from "../store/desktop";
import { errorText } from "../store/format";
import { emptyTagFilter, isTagFilterEmpty, tagFilterExpression, type TagFilter } from "../store/tagFilter";
import { Button, Checkbox, Icon, IconButton, PopoverFrame, SegmentedControl, Separator, Spinner, TagCapsule, UIText, cx } from "../ui";

const inputClass =
  "h-6 px-2 rounded-(--radius-control) border border-input bg-background text-(length:--fs-callout) text-foreground placeholder:text-muted-foreground focus:border-primary";

/**
 * 标签筛选: a popover that edits a workbench's tag filter as a draft (tags
 * joined by AND or OR, each optionally excluded) and applies it on 应用.
 */
export function TagFilterButton(props: { filter: TagFilter; onApply: (filter: TagFilter) => void }) {
  const open = signal(false);
  const draft = signal<TagFilter>(emptyTagFilter);
  const tags = signal<readonly TagDto[]>([]);
  const search = signal("");
  const loading = signal(false);
  const error = signal<string | null>(null);
  let loadVersion = 0;

  const load = async () => {
    const version = ++loadVersion;
    batch(() => {
      loading.set(true);
      error.set(null);
    });
    try {
      const snapshot = await Tags.catalog();
      if (version !== loadVersion) return;
      tags.set(snapshot.tags);
    } catch (e) {
      if (version === loadVersion) error.set(errorText(e));
    } finally {
      if (version === loadVersion) loading.set(false);
    }
  };

  effect(() => {
    if (!open()) return;
    untracked(() => {
      batch(() => {
        draft.set(props.filter);
        search.set("");
      });
      void load();
    });
  });

  let seenTags = untracked(tagsChanged);
  effect(() => {
    const revision = tagsChanged();
    if (revision === seenTags) return;
    seenTags = revision;
    if (untracked(open)) void load();
  });

  const names = () => Object.fromEntries(tags().map((t) => [t.id, t.name]));
  const tagById = (id: string) => tags().find((t) => t.id === id);
  const missing = () => !loading() && error() === null && draft().terms.some((t) => !tagById(t.tagId));
  const valid = () => isTagFilterEmpty(draft()) || !missing();
  const matches = () => {
    const q = search().trim().toLowerCase();
    return tags().filter((t) => q === "" || t.name.toLowerCase().includes(q));
  };

  const add = (tag: TagDto) =>
    draft.update((d) => (d.terms.some((t) => t.tagId === tag.id) ? d : { ...d, terms: [...d.terms, { tagId: tag.id, negated: false }] }));
  const remove = (id: string) => draft.update((d) => ({ ...d, terms: d.terms.filter((t) => t.tagId !== id) }));
  const negate = (id: string, negated: boolean) =>
    draft.update((d) => ({ ...d, terms: d.terms.map((t) => (t.tagId === id ? { ...t, negated } : t)) }));

  const active = () => !isTagFilterEmpty(props.filter);

  return (
    <PopoverFrame
      open={open()}
      onOpenChange={open.set}
      width={dialogs.tagFilter}
      maxHeight={480}
      trigger={
        <View
          class={cx(
            "flex-row items-center gap-1.5 h-6 px-2 rounded-(--radius-control) border",
            active() ? "bg-primary border-primary" : "bg-background border-input hover:bg-hover",
          )}
          accessibilityLabel={tr("标签筛选")}
          accessibilityValue={{ text: active() ? tr("已启用") : tr("未启用") }}
        >
          <Icon name="tag" size={12} tone={active() ? "onPrimary" : "secondary"} />
          <UIText variant="callout" tone="inherit" class={active() ? "text-primary-foreground" : "text-foreground"} lines={1}>
            {tr("标签筛选")}
          </UIText>
        </View>
      }
    >
      <View class="gap-2.5">
        <View class="flex-row items-center justify-between">
          <UIText variant="headline">{tr("标签筛选")}</UIText>
          <SegmentedControl
            class="w-25"
            value={draft().combine}
            options={[
              { value: "and", label: "AND" },
              { value: "or", label: "OR" },
            ]}
            onChange={(value) => draft.update((d) => ({ ...d, combine: value === "or" ? "or" : "and" }))}
          />
        </View>
        <View class="gap-1.5">
          <Each of={draft().terms} key={(t) => t.tagId}>
            {(term) => (
              <View class="flex-row items-center gap-2">
                <Checkbox
                  checked={term().negated}
                  accessibilityLabel={`${tr("排除")} ${tagById(term().tagId)?.name ?? ""}`}
                  onChange={(checked) => negate(term().tagId, checked)}
                />
                <UIText variant="caption" tone="secondary">
                  NOT
                </UIText>
                <If
                  when={tagById(term().tagId)}
                  else={
                    <UIText variant="callout" tone="secondary" class="flex-1">
                      {loading() ? "…" : tr("已删除标签")}
                    </UIText>
                  }
                >
                  {(tag) => (
                    <View class="flex-1 flex-row">
                      <TagCapsule name={tag().name} color={tag().color} />
                    </View>
                  )}
                </If>
                <IconButton icon="x" label={tr("移除")} size="sm" iconSize={10} onPress={() => remove(term().tagId)} />
              </View>
            )}
          </Each>
        </View>
        <If when={!isTagFilterEmpty(draft())}>
          <UIText variant="monoCaption" tone="secondary" lines={3} selectable>
            {tagFilterExpression(draft(), names())}
          </UIText>
        </If>
        <If when={!valid()}>
          <UIText variant="caption" tone="danger">
            {tr("筛选标签已删除，请替换或移除")}
          </UIText>
        </If>
        <Separator />
        <UIText variant="caption" weight="medium">
          {tr("添加到当前分组")}
        </UIText>
        <Input
          value={search()}
          placeholder={tr("搜索标签")}
          accessibilityLabel={tr("搜索标签")}
          class={inputClass}
          onChange={({ value }) => search.set(value)}
        />
        <If when={loading()}>
          <Spinner />
        </If>
        <If when={error()}>
          {(message) => (
            <View class="flex-row items-center gap-2">
              <UIText variant="caption" tone="danger" class="flex-1">
                {message()}
              </UIText>
              <Button label={tr("重试")} size="sm" onPress={() => void load()} />
            </View>
          )}
        </If>
        <View class="flex-row flex-wrap gap-1.5">
          <Each of={matches()} key={(t) => t.id}>
            {(tag) => <TagCapsule name={tag().name} color={tag().color} size="detail" onPress={() => add(tag())} />}
          </Each>
        </View>
        <If when={!loading() && error() === null && matches().length === 0}>
          <UIText variant="caption" tone="secondary">
            {tr("无匹配标签")}
          </UIText>
        </If>
        <Separator />
        <View class="flex-row items-center gap-2">
          <BaseButton
            class="h-6 px-2 justify-center rounded-(--radius-control) hover:bg-hover"
            accessibilityLabel={tr("清空")}
            onPress={() => draft.set(emptyTagFilter)}
          >
            <UIText variant="callout">{tr("清空")}</UIText>
          </BaseButton>
          <View class="flex-1" />
          <Button label={tr("取消")} size="sm" onPress={() => open.set(false)} />
          <Button
            label={tr("应用")}
            variant="primary"
            size="sm"
            disabled={!valid() || (!isTagFilterEmpty(draft()) && (loading() || error() !== null))}
            onPress={() => {
              props.onApply(draft());
              open.set(false);
            }}
          />
        </View>
      </View>
    </PopoverFrame>
  );
}
