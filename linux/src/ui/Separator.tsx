import { View } from "nsh";
import { cx } from "./cx";

/** A one-point rule in the separator color. */
export function Separator(props: { vertical?: boolean; class?: string }) {
  return (
    <View
      class={cx(props.vertical ? "w-px self-stretch" : "h-px self-stretch", "bg-separator", props.class)}
      accessibilityElementsHidden
    />
  );
}
