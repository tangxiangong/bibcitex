import { batch, effect, signal, untracked, View } from "nsh";
import { Input } from "nsh/base";
import { Catalog, Files, type LibraryDto } from "bibcitex/native";
import { tr } from "../i18n";
import { dialogs } from "../theme";
import { libraryList } from "../store/desktop";
import { errorText } from "../store/format";
import { Button, DialogFrame, Icon, Spinner, Surface, UIText } from "../ui";

const fieldClass =
  "h-8 px-2.5 rounded-(--radius-control) border border-input bg-background text-(length:--fs-body) text-foreground placeholder:text-muted-foreground focus:border-primary";

/**
 * 新增文献库 / 编辑文献库: name, the .bib file (picked natively, so its
 * absolute path is registered), and a description. Saving keeps the sheet
 * open and blocks dismissal until the registry answers.
 */
export function LibraryEditor(props: {
  open: boolean;
  /** The library being edited, or `null` to add one. */
  library: LibraryDto | null;
  onClose: () => void;
  /** The saved library, so the workbench can select it. */
  onSaved: (library: LibraryDto) => void;
}) {
  const name = signal("");
  const path = signal("");
  const description = signal("");
  const error = signal<string | null>(null);
  const saving = signal(false);

  // Fill the fields each time the sheet opens for a library (or none).
  effect(() => {
    const open = props.open;
    const library = props.library;
    if (!open) return;
    untracked(() => {
      batch(() => {
        name.set(library?.name ?? "");
        path.set(library?.path ?? "");
        description.set(library?.description ?? "");
        error.set(null);
      });
    });
  });

  const pick = async () => {
    try {
      const picked = await Files.pickBibliography();
      if (!picked) return;
      path.set(picked);
      if (name().trim() === "") {
        const file = picked.split(/[\\/]/).pop() ?? "";
        name.set(file.replace(/\.[^.]+$/, ""));
      }
    } catch (e) {
      error.set(errorText(e));
    }
  };

  const save = async () => {
    const trimmed = name().trim();
    if (trimmed === "") {
      error.set(tr("文献库名称不能为空"));
      return;
    }
    if (libraryList().some((l) => l.name === trimmed && l.name !== props.library?.name)) {
      error.set(tr("该名称已存在，请换一个"));
      return;
    }
    saving.set(true);
    try {
      const saved = props.library
        ? await Catalog.update(props.library.name, trimmed, path() === props.library.path ? null : path(), description())
        : await Catalog.add(trimmed, path(), description());
      saving.set(false);
      props.onSaved(saved);
      props.onClose();
    } catch (e) {
      saving.set(false);
      error.set(errorText(e));
    }
  };

  const close = () => {
    if (saving()) return;
    props.onClose();
  };

  return (
    <DialogFrame
      open={props.open}
      onOpenChange={(open) => {
        if (!open) close();
      }}
      width={dialogs.libraryEditor}
      busy={saving()}
      title={props.library ? tr("编辑文献库") : tr("新增文献库")}
      subtitle={props.library ? undefined : tr("添加一个 .bib 文件到你的工作空间")}
      mark={
        <View class="w-12 h-12 items-center justify-center rounded-(--radius-tile) bg-primary/10">
          <Icon name="folderAdd" size={24} tone="accent" />
        </View>
      }
      footer={
        <View class="flex-1 flex-row items-center gap-3">
          <If when={saving()}>
            <Spinner />
          </If>
          <View class="flex-1" />
          <Button label={tr("取消")} size="lg" disabled={saving()} onPress={close} />
          <Button
            label={tr("保存")}
            variant="primary"
            size="lg"
            disabled={name().trim() === "" || path() === "" || saving()}
            onPress={() => void save()}
          />
        </View>
      }
    >
      <View class="gap-2">
        <UIText variant="callout" weight="medium">
          {tr("文献库名称")}
        </UIText>
        <Input
          value={name()}
          placeholder={tr("为文献库起一个名字")}
          accessibilityLabel={tr("文献库名称")}
          readonly={saving()}
          autoFocus
          class={fieldClass}
          onChange={({ value }) => name.set(value)}
          onSubmit={() => void save()}
        />
      </View>
      <View class="gap-2">
        <UIText variant="callout" weight="medium">
          {tr("文件路径")}
        </UIText>
        <Surface variant="card" class="flex-row items-center gap-3 p-4">
          <Icon name="fileText" size={24} tone="secondary" />
          <UIText variant="callout" tone={path() === "" ? "secondary" : "primary"} lines={2} class="flex-1">
            {path() === "" ? tr("尚未选择文件") : path()}
          </UIText>
          <Button
            label={props.library ? tr("重新定位来源") : tr("选择文件")}
            size="lg"
            disabled={saving()}
            onPress={() => void pick()}
          />
        </Surface>
      </View>
      <View class="gap-2">
        <UIText variant="callout" weight="medium">
          {tr("描述")}
        </UIText>
        <Input
          value={description()}
          placeholder={tr("简单描述一下这个文献库...")}
          accessibilityLabel={tr("描述")}
          multiline
          readonly={saving()}
          class={`${fieldClass} h-auto min-h-16 py-1.5`}
          onChange={({ value }) => description.set(value)}
        />
      </View>
      <If when={error()}>
        {(message) => (
          <UIText variant="callout" tone="danger" selectable>
            {message()}
          </UIText>
        )}
      </If>
    </DialogFrame>
  );
}
