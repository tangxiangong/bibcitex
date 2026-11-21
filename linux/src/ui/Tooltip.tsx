import type { JSX } from "nsh";
import { Tooltip as BaseTooltip } from "nsh/base";
import { UIText } from "./Typography";

/** The tooltip bubble: the popup family's surface, compact. */
export const tooltipClass = "bg-popover border border-panel-border rounded-(--radius-keycap) px-2 py-1";

/**
 * A tooltip for a view's `tooltip` prop:
 * `<View tooltip={<Tooltip text={tr("刷新")} />}>`. Icon-only controls get
 * theirs from `IconButton`.
 */
export function Tooltip(props: { text: string; children?: JSX.Element }) {
  return (
    <BaseTooltip class={tooltipClass}>
      <UIText variant="small">{props.text}</UIText>
      {props.children}
    </BaseTooltip>
  );
}
