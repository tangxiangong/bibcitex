import { keymap, View } from "nsh";
import { cx } from "./cx";
import { isMac } from "./platform";
import { UIText } from "./Typography";

/** Keys the helper shows as hints, spelled per platform. */
export type NamedKey = "enter" | "tab" | "escape" | "up" | "down" | "backspace";

const macKeys: Record<NamedKey, string> = { enter: "↵", tab: "⇥", escape: "esc", up: "↑", down: "↓", backspace: "⌫" };
const pcKeys: Record<NamedKey, string> = { enter: "Enter", tab: "Tab", escape: "Esc", up: "↑", down: "↓", backspace: "Backspace" };

/** A keystroke as the platform writes it: `⇧⌘K` on macOS, `Ctrl+Shift+K` elsewhere. */
export function shortcutLabel(keystroke: string): string {
  try {
    return keymap.label(keystroke);
  } catch {
    return keystroke;
  }
}

export interface KbdProps {
  /** A named key (`enter`) or a keystroke (`mod+shift+k`). */
  keyName?: NamedKey;
  shortcut?: string;
  class?: string;
}

/** A key cap: 18pt high, rounded 6, in the control surface color. */
export function Kbd(props: KbdProps) {
  const label = (): string =>
    props.shortcut ? shortcutLabel(props.shortcut) : props.keyName ? (isMac ? macKeys : pcKeys)[props.keyName] : "";
  return (
    <View
      class={cx("min-w-4.5 h-4.5 px-1 items-center justify-center rounded-(--radius-keycap) bg-control-surface", props.class)}
      accessibilityElementsHidden
    >
      <UIText variant="small" weight="medium" tone="secondary" lines={1}>
        {label()}
      </UIText>
    </View>
  );
}
