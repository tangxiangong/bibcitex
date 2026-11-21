import type { InputRef, JSX } from "nsh";
import { signal, View } from "nsh";
import { Input } from "nsh/base";
import { tr } from "../i18n";
import { transition } from "../theme/motion";
import { cx } from "./cx";
import { Icon } from "./Icon";
import { IconButton } from "./IconButton";

/** `workbench`: the list pane's field; `tray`: the tray's framed field;
 * `hero`: the helper's 20pt field on glass, with no frame of its own. */
export type SearchFieldSize = "workbench" | "tray" | "hero";

/** Keys a search field hands to its surface while no IME composition runs. */
export type SearchKey = "Tab" | "Escape" | "BackspaceAtStart";

export interface SearchFieldProps {
  value: string;
  onChange: (value: string) => void;
  placeholder?: string;
  size?: SearchFieldSize;
  /** Return: committed text only, never during a composition. */
  onSubmit?: () => void;
  /** ↑ / ↓ move the surface's selection (-1 / +1). */
  onMove?: (delta: -1 | 1) => void;
  /** Tab, Escape, and Backspace in an empty field. */
  onKey?: (key: SearchKey) => void;
  /** Whether an input method is composing (pinyin, kana): searches pause. */
  onCompositionChange?: (composing: boolean) => void;
  /** Shows a clear button while there is text (default false: the macOS client has none). */
  clearable?: boolean;
  autoFocus?: boolean;
  inputRef?: (ref: InputRef) => void;
  accessibilityLabel?: string;
  /** Content after the field (a spinner, the library switch). */
  trailing?: JSX.Element;
  /** Layout classes only. */
  class?: string;
}

const frame: Record<SearchFieldSize, string> = {
  workbench: "h-8 px-2 gap-2 rounded-(--radius-control) bg-control-surface",
  tray: "h-9 px-2.5 gap-2 rounded-(--radius-field) bg-control-surface border border-panel-border focus-within:border-primary/60",
  hero: "h-11 gap-3.5",
};

const textClass: Record<SearchFieldSize, string> = {
  workbench: "text-(length:--fs-body)",
  tray: "text-(length:--fs-body)",
  hero: "text-(length:--fs-hero)",
};

/**
 * The search field of every surface. While an input method composes, the
 * keys the surface uses (↑ ↓ Return Tab Escape) stay with the input method,
 * and nothing is searched until the composition is committed.
 */
export function SearchField(props: SearchFieldProps) {
  // Per field instance, never per list cell: safe to keep here.
  const composing = signal(false);
  const capture = (): readonly string[] => {
    if (composing()) return [];
    const keys = ["up", "down"];
    if (props.onKey) keys.push("tab", "escape");
    return keys;
  };

  return (
    <View
      class={cx("flex-row items-center", frame[props.size ?? "workbench"], transition.colors(), props.class)}
      captureKeys={capture()}
      onKeyDown={(e) => {
        if (composing() || e.repeat && e.key === "Tab") return;
        switch (e.key) {
          case "ArrowUp":
            props.onMove?.(-1);
            break;
          case "ArrowDown":
            props.onMove?.(1);
            break;
          case "Tab":
            props.onKey?.("Tab");
            break;
          case "Escape":
            props.onKey?.("Escape");
            break;
          case "Backspace":
            if (props.value === "") props.onKey?.("BackspaceAtStart");
            break;
        }
      }}
    >
      <Icon name="search" size={props.size === "hero" ? 18 : 15} tone="secondary" />
      <Input
        ref={(r: InputRef) => props.inputRef?.(r)}
        value={props.value}
        placeholder={props.placeholder}
        autoFocus={props.autoFocus}
        autoCapitalize="none"
        autoCorrect={false}
        returnKeyType="search"
        submitBehavior="submit"
        accessibilityLabel={props.accessibilityLabel ?? props.placeholder}
        class={cx("flex-1 min-w-0 text-foreground placeholder:text-muted-foreground", textClass[props.size ?? "workbench"])}
        onChange={({ value }) => props.onChange(value)}
        onCompositionChange={({ composing: active }) => {
          composing.set(active);
          props.onCompositionChange?.(active);
        }}
        onSubmit={() => {
          if (!composing()) props.onSubmit?.();
        }}
      />
      <If when={props.clearable && props.value !== ""}>
        <IconButton icon="x" label={tr("清空")} size="sm" iconSize={10} onPress={() => props.onChange("")} />
      </If>
      {props.trailing}
    </View>
  );
}
