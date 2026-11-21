import type { JSX } from "nsh";
import { Button as BaseButton } from "nsh/base";
import { transition } from "../theme/motion";
import { cx } from "./cx";
import { Icon, type IconName, type IconTone } from "./Icon";
import { UIText, type TextTone } from "./Typography";

export type ButtonVariant = "primary" | "secondary" | "ghost" | "outline" | "danger";
export type ButtonSize = "sm" | "md" | "lg";

const variantClass: Record<ButtonVariant, string> = {
  primary: "bg-primary hover:bg-primary/90 active:bg-primary/80",
  secondary: "bg-control-surface hover:bg-hover active:bg-selection",
  ghost: "hover:bg-hover active:bg-selection selected:bg-selection",
  outline: "border border-input bg-background hover:bg-hover active:bg-selection",
  danger: "bg-destructive hover:bg-destructive/90 active:bg-destructive/80",
};

const sizeClass: Record<ButtonSize, string> = {
  sm: "h-6 px-2 gap-1",
  md: "h-7 px-3 gap-1.5",
  lg: "h-8 px-4 gap-2",
};

const labelTone: Record<ButtonVariant, TextTone> = {
  primary: "inherit",
  secondary: "primary",
  ghost: "primary",
  outline: "primary",
  danger: "inherit",
};

const iconTone: Record<ButtonVariant, IconTone> = {
  primary: "onPrimary",
  secondary: "primary",
  ghost: "primary",
  outline: "primary",
  danger: "onPrimary",
};

export interface ButtonProps {
  variant?: ButtonVariant;
  size?: ButtonSize;
  /** The button's text (an existing UI string). */
  label?: string;
  icon?: IconName;
  disabled?: boolean;
  selected?: boolean;
  /** Layout classes only. */
  class?: string;
  accessibilityLabel?: string;
  onPress?: () => void;
  /** Content after the label (a key cap). */
  children?: JSX.Element;
}

/** The app's text button. The default commit of a form is `primary`. */
export function Button(props: ButtonProps) {
  const variant = (): ButtonVariant => props.variant ?? "secondary";
  return (
    <BaseButton
      class={cx(
        "flex-row items-center justify-center rounded-(--radius-control)",
        "focus-visible:outline-2 focus-visible:outline-ring focus-visible:outline-offset-1",
        "disabled:opacity-50",
        sizeClass[props.size ?? "md"],
        variantClass[variant()],
        transition.colors(),
        props.class,
      )}
      disabled={props.disabled}
      selected={props.selected}
      accessibilityLabel={props.accessibilityLabel ?? props.label}
      onPress={props.onPress}
    >
      <If when={props.icon}>
        {(name) => <Icon name={name()} size={props.size === "lg" ? 16 : 14} tone={iconTone[variant()]} />}
      </If>
      <If when={props.label}>
        <UIText
          variant={props.size === "sm" ? "callout" : "body"}
          weight={variant() === "primary" || variant() === "danger" ? "medium" : "regular"}
          tone={labelTone[variant()]}
          class={variant() === "primary" || variant() === "danger" ? "text-primary-foreground" : undefined}
          lines={1}
        >
          {props.label}
        </UIText>
      </If>
      {props.children}
    </BaseButton>
  );
}
