import { updateState, checkUpdates, downloadAndPrepare, restartAfterUpdate, updateBusy, updatesAvailable } from "../store/updater";
import type { JSX } from "nsh";
import { Image, Scroll, signal, View } from "nsh";
import { Shortcut, type CitationFormat } from "bibcitex/native";
import { tr } from "../i18n";
import logo from "../../.nsh/resources/favicon.png";
import {
  desktop,
  desktopCapabilities,
  desktopPlatform,
  portalShortcutDescription,
  prefs,
  savePreference,
  showError,
} from "../store/desktop";
import { errorText } from "../store/format";
import { changeShortcut, pauseShortcut } from "../store/surfaces";
import { Button, Select, Separator, ShortcutRecorder, Spinner, Switch, UIText, isMac } from "../ui";

function Section(props: { title: string; children?: JSX.Element }) {
  return (
    <View class="gap-1.5">
      <UIText variant="callout" weight="semibold" tone="secondary" class="px-1">
        {props.title}
      </UIText>
      <View class="rounded-(--radius-card) border border-separator bg-control-surface/40 px-3">{props.children}</View>
    </View>
  );
}

function Row(props: { label: string; children?: JSX.Element }) {
  return (
    <View class="flex-row items-center gap-3 min-h-10 py-1.5">
      <UIText class="flex-1">{props.label}</UIText>
      {props.children}
    </View>
  );
}

function Note(props: { text: string; tone?: "secondary" | "danger" }) {
  return (
    <UIText variant="callout" tone={props.tone ?? "secondary"} class="pb-2">
      {props.text}
    </UIText>
  );
}

/** The citation family a format belongs to: key, LaTeX or Typst. */
function familyOf(formats: readonly CitationFormat[], id: string): string {
  return formats.find((f) => f.id === id)?.group ?? "raw";
}

/**
 * Settings (480×540, grouped): tray icon and helper shortcut, appearance and
 * language, citation format, updates. Every control writes the shared
 * preferences at once; a failed save restores the confirmed value.
 */
export function SettingsPanel() {
  const formats = (() => {
    try {
      return desktop.citationFormats();
    } catch {
      return [] as CitationFormat[];
    }
  })();
  const shortcutError = signal<string | null>(null);
  const checking = signal(false);
  const accessibility = signal(isMac ? desktop.accessibilityStatus() : true);

  const p = () => prefs();
  const family = () => familyOf(formats, p()?.citationFormat ?? "raw");
  const formatErrorKey = () =>
    desktopPlatform()?.os === "macos"
      ? "error.helperShortcutFormatMac"
      : desktopPlatform()?.os === "windows"
        ? "error.helperShortcutFormatWindows"
        : "error.helperShortcutFormatLinux";

  const setFamily = (next: string) => {
    if (next === family()) return;
    const first = formats.find((f) => f.group === next);
    if (first) savePreference({ citationFormat: first.id });
  };

  const record = (keystroke: string) => {
    shortcutError.set(null);
    changeShortcut(keystroke).catch((e) => shortcutError.set(errorText(e)));
  };

  const update = () => updateState();
  const check = async () => {
    checking.set(true);
    try {
      await checkUpdates();
    } catch (e) {
      showError(e);
    } finally {
      checking.set(false);
    }
  };

  const updateMessage = (): string | null => {
    const u = update();
    if (!u) return null;
    if (!u.canCheck) return u.unavailableReasonKey ? tr(u.unavailableReasonKey) : tr("自动更新不可用");
    switch (u.phase) {
      case "available":
        return tr("BibCiTeX {0} 现已推出，当前版本为 {1}。", u.version ?? "", u.currentVersion);
      case "installing":
        return update()?.total ? `${tr("正在下载…")} ${Math.min(100, Math.floor((update()!.downloaded / update()!.total!) * 100))}%` : tr("正在下载…");
      case "ready":
        return tr("BibCiTeX {0} 已下载，将在退出时安装。", u.version ?? "");
      case "error":
        return u.message ? tr(u.message) : tr("暂时无法获取更新信息，请稍后重试。");
      case "idle":
        return u.checked ? tr("当前已是最新版本") : null;
      default:
        return null;
    }
  };

  const shortcutValue = () => {
    const value = p()?.helperShortcut ?? "";
    try {
      return Shortcut.parse(value).value.toLowerCase();
    } catch {
      return value.toLowerCase();
    }
  };

  return (
    <Scroll class="flex-1 bg-background" contentClass="p-5 gap-5">
      <Section title={tr("通用")}>
        <Row label={isMac ? tr("显示菜单栏图标") : tr("显示托盘图标")}>
          <Switch
            checked={p()?.trayIconEnabled ?? true}
            disabled={!desktopCapabilities()?.tray}
            accessibilityLabel={isMac ? tr("显示菜单栏图标") : tr("显示托盘图标")}
            onChange={(checkedValue) => savePreference({ trayIconEnabled: checkedValue })}
          />
        </Row>
        <Separator />
        <Row label={tr("助手快捷键")}>
          <View class="flex-row items-center gap-2">
            <If
              when={desktopPlatform()?.portalShortcut && portalShortcutDescription()}
              else={
                <ShortcutRecorder
                  value={shortcutValue()}
                  onRecord={record}
                  onInvalid={() => shortcutError.set(tr(formatErrorKey()))}
                  onRecordingChange={pauseShortcut}
                />
              }
            >
              {(description) => (
                <UIText variant="mono" lines={1}>
                  {String(description())}
                </UIText>
              )}
            </If>
            <Button
              label={tr("恢复默认")}
              size="sm"
              onPress={() => {
                try {
                  record(Shortcut.defaultValue());
                } catch (e) {
                  shortcutError.set(errorText(e));
                }
              }}
            />
          </View>
        </Row>
        <If when={shortcutError()}>{(message) => <Note text={message()} tone="danger" />}</If>
        <If when={isMac && !accessibility()}>
          <Separator />
          <View class="flex-row items-center gap-2 py-2">
            <UIText variant="callout" tone="secondary" class="flex-1">
              {tr("缺少辅助功能权限，无法控制其他应用")}
            </UIText>
            <Button label={tr("重试")} size="sm" onPress={() => accessibility.set(desktop.accessibilityStatus())} />
          </View>
        </If>
      </Section>
      <Section title={tr("外观")}>
        <Row label={tr("主题")}>
          <Select
            class="w-40"
            accessibilityLabel={tr("主题")}
            value={p()?.appearance ?? "system"}
            options={[
              { value: "system", title: tr("跟随系统") },
              { value: "light", title: tr("浅色") },
              { value: "dark", title: tr("深色") },
            ]}
            onChange={(value) => savePreference({ appearance: value })}
          />
        </Row>
        <Separator />
        <Row label={tr("语言")}>
          <Select
            class="w-40"
            accessibilityLabel={tr("语言")}
            value={p()?.language ?? "system"}
            options={[
              { value: "system", title: tr("跟随系统") },
              { value: "zh-Hans", title: "简体中文" },
              { value: "en", title: "English" },
            ]}
            onChange={(value) => savePreference({ language: value })}
          />
        </Row>
      </Section>
      <Section title={tr("引用")}>
        <Row label={tr("引用格式")}>
          <Select
            class="w-40"
            accessibilityLabel={tr("引用格式")}
            value={family()}
            options={[
              { value: "raw", title: "key" },
              { value: "latex", title: "LaTeX" },
              { value: "typst", title: "Typst" },
            ]}
            onChange={setFamily}
          />
        </Row>
        <If when={family() !== "raw"}>
          <Separator />
          <Row label={tr("引用指令")}>
            <Select
              class="w-40"
              accessibilityLabel={tr("引用指令")}
              value={p()?.citationFormat ?? null}
              options={formats.filter((f) => f.group === family()).map((f) => ({ value: f.id, title: tr(f.label) }))}
              onChange={(value) => savePreference({ citationFormat: value })}
            />
          </Row>
        </If>
        <Separator />
        <Row label={tr("多选输出方式")}>
          <Select
            class="w-40"
            accessibilityLabel={tr("多选输出方式")}
            disabled={family() === "raw"}
            value={p()?.citationMode ?? "merged"}
            options={[
              { value: "merged", title: tr("合并") },
              { value: "individual", title: tr("逐项") },
            ]}
            onChange={(value) => savePreference({ citationMode: value })}
          />
        </Row>
      </Section>
      <Section title={tr("更新")}>
        <Row label={tr("当前版本")}>
          <UIText tone="secondary" selectable>
            {update()?.currentVersion ?? "—"}
          </UIText>
        </Row>
        <Separator />
        <Row label={tr("更新通道")}>
          <Select
            class="w-40"
            accessibilityLabel={tr("更新通道")}
            disabled={updateBusy() || update()?.phase === "ready"}
            value={p()?.updateChannel ?? "stable"}
            options={[
              { value: "stable", title: tr("正式版") },
              { value: "beta", title: "Beta" },
              { value: "alpha", title: "Alpha" },
            ]}
            onChange={(value) => {
              savePreference({ updateChannel: value });
            }}
          />
        </Row>
        <Separator />
        <Row label={tr("自动下载并安装更新")}>
          <Switch
            checked={p()?.automaticUpdates ?? false}
            disabled={!update()?.canCheck}
            accessibilityLabel={tr("自动下载并安装更新")}
            onChange={(value) => savePreference({ automaticUpdates: value })}
          />
        </Row>
        <Separator />
        <View class="flex-row items-center gap-2 py-2">
          <If
            when={update()?.phase === "ready"}
            else={
              <If
                when={update()?.phase === "available" && update()?.id}
                else={
                  <Button
                    label={tr("检查更新")}
                    disabled={!updatesAvailable() || checking() || update()?.phase === "checking" || update()?.phase === "installing"}
                    onPress={() => void check()}
                  />
                }
              >
                {() => <Button label={tr("下载并安装")} disabled={!updatesAvailable()} variant="primary" onPress={() => downloadAndPrepare().catch(showError)} />}
              </If>
            }
          >
            <Button label={tr("重启以完成更新")} variant="primary" onPress={() => restartAfterUpdate().catch(showError)} />
          </If>
          <If when={checking() || update()?.phase === "checking" || update()?.phase === "installing"}>
            <Spinner />
          </If>
        </View>
        <If when={updateMessage()}>{(message) => <Note text={message()} />}</If>
      </Section>
      <AboutContent />
    </Scroll>
  );
}

/** The app's mark, name and version (also the About window's content). */
export function AboutContent() {
  return (
    <View class="items-center gap-2 py-4">
      <Image src={logo} accessibilityLabel="BibCiTeX" style={{ width: 64, height: 64 }} />
      <UIText variant="title3" weight="semibold">
        BibCiTeX
      </UIText>
      <UIText variant="callout" tone="secondary" selectable>
        {`${tr("当前版本")} ${updateState()?.currentVersion ?? "—"}`}
      </UIText>
    </View>
  );
}

/** The About window. */
export function AboutPanel() {
  return (
    <View class="flex-1 items-center justify-center bg-background p-6">
      <AboutContent />
    </View>
  );
}
