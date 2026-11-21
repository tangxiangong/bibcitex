import { createMotion, View } from "nsh";
import { cx } from "./cx";

/** An indeterminate activity ring. Under reduced motion it rests. */
export function Spinner(props: { size?: "sm" | "md"; accessibilityLabel?: string; class?: string }) {
  const motion = createMotion();
  void motion.play("spin");
  return (
    <View
      role="progressbar"
      accessibilityLabel={props.accessibilityLabel}
      class={cx(
        props.size === "md" ? "w-4 h-4 border-2" : "w-3 h-3 border-[1.5px]",
        "rounded-full border-muted-foreground/25 border-t-muted-foreground",
        props.class,
      )}
      style={motion.style}
    />
  );
}
