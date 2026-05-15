import { splitProps } from "solid-js";
import type { JSX } from "solid-js";
import SvgIcon, { type SvgIconName } from "./SvgIcon";

export type IconButtonVariant = "ghost" | "primary" | "error";
export type IconButtonSize = "xs" | "sm" | "md" | "lg";

export interface IconButtonProps
  extends Omit<
    JSX.ButtonHTMLAttributes<HTMLButtonElement>,
    "aria-label" | "title"
  > {
  icon: SvgIconName;
  label: string;
  tooltip?: string;
  variant?: IconButtonVariant;
  size?: IconButtonSize;
  iconClass?: string;
}

const VARIANT_CLASSES: Record<IconButtonVariant, string> = {
  ghost: "btn-ghost",
  primary: "btn-primary",
  error: "btn-error",
};

const SIZE_CLASSES: Record<IconButtonSize, string> = {
  xs: "btn-xs",
  sm: "btn-sm",
  md: "btn-md",
  lg: "btn-lg",
};

export function IconButton(props: IconButtonProps) {
  const [local, buttonProps] = splitProps(props, [
    "icon",
    "label",
    "tooltip",
    "variant",
    "size",
    "iconClass",
    "class",
    "type",
    "disabled",
  ]);
  const tooltipText = () => local.tooltip ?? local.label;
  const variantClass = () => VARIANT_CLASSES[local.variant ?? "ghost"];
  const sizeClass = () => SIZE_CLASSES[local.size ?? "sm"];
  const classes = () =>
    [
      "btn",
      "btn-square",
      variantClass(),
      sizeClass(),
      tooltipText() ? "tooltip" : "",
      local.class,
    ]
      .filter(Boolean)
      .join(" ");

  return (
    <button
      {...buttonProps}
      type={local.type ?? "button"}
      class={classes()}
      aria-label={local.label}
      title={tooltipText()}
      data-tip={tooltipText()}
      disabled={local.disabled}
    >
      <SvgIcon name={local.icon} class={local.iconClass} aria-hidden />
    </button>
  );
}

export default IconButton;
