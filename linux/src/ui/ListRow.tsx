import type { JSX } from "nsh";
import { View } from "nsh";
import { helper } from "../theme/metrics";
import { transition } from "../theme/motion";
import { cx } from "./cx";

/**
 * - `workbench`: a result in the main window (content height, 8pt padding).
 * - `library`: a sidebar library (5pt padding).
 * - `helperReference` / `helperLibrary`: the helper's fixed 80pt / 56pt rows,
 *   which the helper panel sizes itself from.
 * - `tray`: a tray result (7pt padding).
 */
export type ListRowVariant = "workbench" | "library" | "helperReference" | "helperLibrary" | "tray";

const variantClass: Record<ListRowVariant, string> = {
  workbench: "px-2.5 py-2",
  library: "px-2.5 py-1.25",
  helperReference: "px-2.5 py-2",
  helperLibrary: "px-2.5 py-2 justify-center",
  tray: "px-2.5 py-1.75",
};

export interface ListRowProps {
  variant: ListRowVariant;
  /** Part of the selection (the library shown, the rows chosen in batch mode). */
  selected?: boolean;
  /** The row the detail follows while several are chosen. */
  focused?: boolean;
  onPress?: () => void;
  accessibilityLabel?: string;
  /** A check box or icon before the content. */
  leading?: JSX.Element;
  /** The content column. */
  children?: JSX.Element;
  /** Right-aligned content (a cite key, tags, a row menu). */
  trailing?: JSX.Element;
}

/**
 * A row of a list: one inset, rounded selection shape shared by every list,
 * so a chosen row reads the same in the helper, the tray and the main window.
 *
 * It keeps no state of its own: every look comes from props the list derives
 * from its data by id, so a recycled cell shows its new item correctly.
 */
export function ListRow(props: ListRowProps) {
  const fixedHeight = (): number | undefined =>
    props.variant === "helperReference" ? helper.referenceRow : props.variant === "helperLibrary" ? helper.libraryRow : undefined;
  return (
    <View class="px-1.5">
      <View
        class={cx(
          "flex-row items-start gap-2.5 rounded-(--radius-row) border border-transparent hover:bg-hover",
          variantClass[props.variant],
          transition.colors(),
        )}
        classList={{ "bg-selection": !!props.selected, "border-primary": !!props.focused }}
        style={{ height: fixedHeight() }}
        accessibilityLabel={props.accessibilityLabel}
        accessibilityState={{ selected: !!props.selected }}
        onPress={props.onPress}
      >
        {props.leading}
        <View class="flex-1 min-w-0 gap-1.25">{props.children}</View>
        {props.trailing}
      </View>
    </View>
  );
}
