import type { JSX } from "nsh";
import { View } from "nsh";
import { cx } from "./cx";
import { UIText, type TextTone } from "./Typography";

export type BadgeVariant = "neutral" | "accent" | "success" | "danger";

const look: Record<BadgeVariant, { bg: string; tone: TextTone }> = {
  neutral: { bg: "bg-control-surface", tone: "secondary" },
  accent: { bg: "bg-primary/15", tone: "accent" },
  success: { bg: "bg-success/15", tone: "success" },
  danger: { bg: "bg-destructive/15", tone: "danger" },
};

/** A small status label (“不可用”, “已选 3”). Ordinary metadata stays neutral. */
export function Badge(props: { label: string; variant?: BadgeVariant; class?: string; children?: JSX.Element }) {
  const v = (): BadgeVariant => props.variant ?? "neutral";
  return (
    <View class={cx("flex-row items-center gap-1 rounded-full px-1.5 py-0.5", look[v()].bg, props.class)}>
      {props.children}
      <UIText variant="caption" tone={look[v()].tone} lines={1}>
        {props.label}
      </UIText>
    </View>
  );
}
