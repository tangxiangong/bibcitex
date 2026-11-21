import type { JSX } from "nsh";
import { View } from "nsh";
import { cx } from "./cx";
import { UIText } from "./Typography";

/** A pane with nothing to show: an optional mark, one line, optional action. */
export function EmptyState(props: { title: string; mark?: JSX.Element; action?: JSX.Element; class?: string }) {
  return (
    <View class={cx("flex-1 items-center justify-center gap-4 p-4", props.class)}>
      {props.mark}
      <UIText tone="secondary">{props.title}</UIText>
      {props.action}
    </View>
  );
}
