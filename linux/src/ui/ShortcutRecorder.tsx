import type { KeyEvent } from "nsh";
import { signal, View } from "nsh";
import { tr } from "../i18n";
import { transition } from "../theme/motion";
import { cx } from "./cx";
import { shortcutLabel } from "./Kbd";
import { isMac } from "./platform";
import { UIText } from "./Typography";

const modifierCodes = /^(Shift|Control|Alt|Meta|OS|Super|Hyper)(Left|Right)?$/;

/** The keystroke of a key press (`mod+shift+k` form), or `null` for a lone modifier. */
export function keystrokeOf(e: KeyEvent): string | null {
  if (modifierCodes.test(e.code)) return null;
  let key: string | null = null;
  if (/^Key[A-Z]$/.test(e.code)) key = e.code.slice(3).toLowerCase();
  else if (/^Digit[0-9]$/.test(e.code)) key = e.code.slice(5);
  else if (/^F([1-9]|1[0-2])$/.test(e.code)) key = e.code.toLowerCase();
  else if (e.code === "Space") key = "space";
  if (!key) return "";
  const parts: string[] = [];
  if (e.modifiers.platform) parts.push(isMac ? "cmd" : "super");
  if (e.modifiers.control) parts.push("ctrl");
  if (e.modifiers.alt) parts.push("alt");
  if (e.modifiers.shift) parts.push("shift");
  parts.push(key);
  return parts.join("+");
}

export interface ShortcutRecorderProps {
  /** The registered shortcut (`mod+shift+k` form). */
  value: string;
  /** A keystroke the user pressed; the owner validates and registers it. */
  onRecord: (keystroke: string) => void;
  /** A key that cannot be a shortcut (no modifier, an unsupported key). */
  onInvalid: () => void;
  /** Recording started or ended: the owner pauses the global shortcut meanwhile. */
  onRecordingChange?: (recording: boolean) => void;
  class?: string;
}

/**
 * The 150 × 26pt shortcut recorder of Settings: a press starts recording,
 * the next valid keystroke is the new shortcut, Escape (or leaving the field)
 * cancels, Tab cancels and moves on.
 */
export function ShortcutRecorder(props: ShortcutRecorderProps) {
  // One recorder per Settings window, never in a list cell.
  const recording = signal(false);
  const setRecording = (value: boolean) => {
    if (recording() === value) return;
    recording.set(value);
    props.onRecordingChange?.(value);
  };

  return (
    <View
      focusable
      role="button"
      accessibilityLabel={tr("助手快捷键")}
      accessibilityHint={recording() ? tr("按 Esc 取消录制") : tr("录制快捷键")}
      captureKeys={recording() ? "all" : "none"}
      onPress={() => setRecording(!recording())}
      onBlur={() => setRecording(false)}
      onKeyDown={(e) => {
        if (!recording()) return;
        if (e.key === "Escape" || e.key === "Tab") {
          setRecording(false);
          return;
        }
        if (e.repeat) return;
        const keystroke = keystrokeOf(e);
        if (keystroke === null) return;
        if (keystroke === "" || !keystroke.includes("+")) {
          props.onInvalid();
          return;
        }
        setRecording(false);
        props.onRecord(keystroke);
      }}
      class={cx(
        "w-37.5 h-6.5 items-center justify-center rounded-(--radius-control) border border-input bg-background hover:bg-hover",
        "focus-visible:outline-2 focus-visible:outline-ring focus-visible:outline-offset-1",
        transition.colors(),
        props.class,
      )}
      classList={{ "border-primary": recording() }}
    >
      <UIText variant={recording() ? "body" : "mono"} tone={recording() ? "secondary" : "primary"} lines={1}>
        {recording() ? tr("请按快捷键…") : shortcutLabel(props.value)}
      </UIText>
    </View>
  );
}
