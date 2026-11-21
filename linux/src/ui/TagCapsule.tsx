import { View } from "nsh";
import { Button as BaseButton } from "nsh/base";
import { transition } from "../theme/motion";
import { cx } from "./cx";
import { UIText } from "./Typography";

/** `#rgb` / `#rrggbb` with an alpha; other colors pass through unchanged. */
export function withAlpha(color: string, alpha: number): string {
  const hex = color.trim();
  const full = /^#([0-9a-f]{3})$/i.test(hex)
    ? `#${hex[1]}${hex[1]}${hex[2]}${hex[2]}${hex[3]}${hex[3]}`
    : /^#[0-9a-f]{6}$/i.test(hex)
      ? hex
      : null;
  if (!full) return color;
  const a = Math.round(Math.min(1, Math.max(0, alpha)) * 255)
    .toString(16)
    .padStart(2, "0");
  return `${full}${a}`;
}

export interface TagCapsuleProps {
  name: string;
  /** The tag's own color: user data, drawn as it is. */
  color: string;
  /** `list`: inside a result row; `detail`: the inspector's 24pt chips. */
  size?: "list" | "detail";
  /** Present when pressing the tag does something (edit, filter). */
  onPress?: () => void;
  accessibilityLabel?: string;
}

/** A tag: a 6pt dot and its name on the tag's color at 14%. */
export function TagCapsule(props: TagCapsuleProps) {
  const content = (
    <View
      class={cx(
        "flex-row items-center rounded-full",
        props.size === "detail" ? "h-6 px-2 gap-1.25" : "px-1.5 py-0.5 gap-1",
      )}
      style={{ backgroundColor: withAlpha(props.color, 0.14) }}
    >
      <View class="w-1.5 h-1.5 rounded-full" style={{ backgroundColor: props.color }} />
      <UIText variant={props.size === "detail" ? "callout" : "caption"} lines={1}>
        {props.name}
      </UIText>
    </View>
  );
  return props.onPress ? (
    <BaseButton
      class={cx("rounded-full active:opacity-70", transition.opacity())}
      accessibilityLabel={props.accessibilityLabel ?? props.name}
      onPress={props.onPress}
    >
      {content}
    </BaseButton>
  ) : (
    content
  );
}
