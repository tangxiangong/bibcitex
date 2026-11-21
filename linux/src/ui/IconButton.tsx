import { Button as BaseButton, Tooltip } from "nsh/base";
import { transition } from "../theme/motion";
import { cx } from "./cx";
import { Icon, type IconName, type IconTone } from "./Icon";
import { tooltipClass } from "./Tooltip";
import { UIText } from "./Typography";

/** `sm` 24pt (row actions), `md` 28pt (pane headers), `toolbar` 32pt. */
export type IconButtonSize = "sm" | "md" | "toolbar";
/** `ghost`: a hover wash; `capsule`: a quiet filled capsule (helper chrome). */
export type IconButtonVariant = "ghost" | "capsule";

const box: Record<IconButtonSize, string> = { sm: "w-6 h-6", md: "w-7 h-7", toolbar: "w-8 h-8" };
const glyph: Record<IconButtonSize, number> = { sm: 14, md: 16, toolbar: 18 };

export interface IconButtonProps {
  icon: IconName;
  /** Names the action for screen readers and its tooltip (an existing UI string). */
  label: string;
  size?: IconButtonSize;
  variant?: IconButtonVariant;
  tone?: IconTone;
  /** Overrides the glyph size of `size` where the macOS client differs (folderAdd 18 in a 28pt box). */
  iconSize?: number;
  selected?: boolean;
  disabled?: boolean;
  /** Layout classes only. */
  class?: string;
  onPress?: () => void;
}

/** An icon-only action: always named, always with a tooltip on desktops. */
export function IconButton(props: IconButtonProps) {
  return (
    <BaseButton
      class={cx(
        "items-center justify-center rounded-(--radius-control)",
        "hover:bg-hover active:bg-selection selected:bg-selection disabled:opacity-40",
        "focus-visible:outline-2 focus-visible:outline-ring focus-visible:outline-offset-1",
        props.variant === "capsule" ? "bg-control-surface rounded-full" : "",
        box[props.size ?? "md"],
        transition.colors(),
        props.class,
      )}
      accessibilityLabel={props.label}
      selected={props.selected}
      disabled={props.disabled}
      onPress={props.onPress}
      tooltip={
        <Tooltip class={tooltipClass}>
          <UIText variant="small">{props.label}</UIText>
        </Tooltip>
      }
    >
      <Icon name={props.icon} size={props.iconSize ?? glyph[props.size ?? "md"]} tone={props.tone ?? "secondary"} />
    </BaseButton>
  );
}
