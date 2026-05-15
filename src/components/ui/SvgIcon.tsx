import { splitProps } from "solid-js";
import type { JSX } from "solid-js";
import {
  SVG_ICON_PATHS,
  type StaticSvgIconName,
} from "../../constants/icons.ts";

export const ICONS = SVG_ICON_PATHS;

export type SvgIconName = StaticSvgIconName;

export interface SvgIconProps
  extends Omit<JSX.HTMLAttributes<HTMLSpanElement>, "children"> {
  name: SvgIconName;
  size?: number | string;
  title?: string;
}

export function SvgIcon(props: SvgIconProps) {
  const [local, iconProps] = splitProps(props, [
    "name",
    "title",
    "class",
    "size",
    "aria-label",
    "aria-hidden",
    "role",
    "style",
  ]);
  const iconUrl = () => ICONS[local.name];
  const size = () =>
    typeof local.size === "number" ? `${local.size}px` : local.size ?? "18px";
  const ariaHidden = () =>
    local["aria-hidden"] ?? (local.title || local["aria-label"] ? undefined : true);

  return (
    <span
      {...iconProps}
      class={`inline-block shrink-0 ${local.class ?? ""}`.trim()}
      aria-label={local["aria-label"] ?? local.title}
      aria-hidden={ariaHidden()}
      role={local.role ?? (local.title || local["aria-label"] ? "img" : undefined)}
      title={local.title}
      style={{
        width: size(),
        height: size(),
        "background-color": "currentColor",
        "mask-image": `url("${iconUrl()}")`,
        "mask-position": "center",
        "mask-repeat": "no-repeat",
        "mask-size": "contain",
        "-webkit-mask-image": `url("${iconUrl()}")`,
        "-webkit-mask-position": "center",
        "-webkit-mask-repeat": "no-repeat",
        "-webkit-mask-size": "contain",
        ...(typeof local.style === "object" ? local.style : {}),
      }}
    />
  );
}

export default SvgIcon;
