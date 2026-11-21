import type { JSX } from "nsh";
import { View } from "nsh";
import { Checkbox as BaseCheckbox, Select as BaseSelect, Switch as BaseSwitch, Toggle, ToggleGroup } from "nsh/base";
import { transition } from "../theme/motion";
import { cx } from "./cx";
import { Icon } from "./Icon";
import { popupClass } from "./Surface";
import { UIText } from "./Typography";

export interface SwitchProps {
  checked: boolean;
  onChange: (checked: boolean) => void;
  /** The setting it turns on (an existing UI string); shown after the track. */
  label?: string;
  accessibilityLabel?: string;
  disabled?: boolean;
  class?: string;
}

/** The small switch of the macOS client (高级搜索, 自动检查更新). */
export function Switch(props: SwitchProps) {
  return (
    <BaseSwitch
      checked={props.checked}
      disabled={props.disabled}
      accessibilityLabel={props.accessibilityLabel ?? props.label}
      onChange={({ checked }) => props.onChange(checked)}
      class={cx("flex-row items-center gap-2 disabled:opacity-50", props.class)}
      trackClass={cx(
        "w-8 h-4.5 rounded-full bg-input checked:bg-primary p-0.5",
        "focus-visible:outline-2 focus-visible:outline-ring focus-visible:outline-offset-1",
        transition.colors(),
      )}
      thumbClass={cx("w-3.5 h-3.5 rounded-full bg-background checked:translate-x-3.5", transition.press())}
    >
      <If when={props.label}>
        <UIText>{props.label}</UIText>
      </If>
    </BaseSwitch>
  );
}

export interface CheckboxProps {
  checked: boolean;
  indeterminate?: boolean;
  onChange: (checked: boolean) => void;
  accessibilityLabel: string;
  disabled?: boolean;
  class?: string;
}

/** The batch-selection check of a result row. */
export function Checkbox(props: CheckboxProps) {
  return (
    <BaseCheckbox
      checked={props.checked}
      indeterminate={props.indeterminate}
      disabled={props.disabled}
      accessibilityLabel={props.accessibilityLabel}
      onChange={({ checked }) => props.onChange(checked)}
      class={cx("disabled:opacity-50", props.class)}
      indicatorClass={cx(
        "w-3.5 h-3.5 rounded-sm border border-input bg-background items-center justify-center",
        "checked:bg-primary checked:border-primary indeterminate:bg-primary indeterminate:border-primary",
        "focus-visible:outline-2 focus-visible:outline-ring focus-visible:outline-offset-1",
        transition.colors(),
      )}
      indicator={
        <If when={props.checked || props.indeterminate}>
          <Icon name={props.indeterminate ? "x" : "check"} size={10} tone="onPrimary" />
        </If>
      }
    />
  );
}

export interface SegmentOption {
  value: string;
  label: string;
}

export interface SegmentedControlProps {
  options: readonly SegmentOption[];
  value: string;
  onChange: (value: string) => void;
  disabled?: boolean;
  class?: string;
}

/** Exactly one of a few choices (AND / OR, key / LaTeX / Typst). */
export function SegmentedControl(props: SegmentedControlProps) {
  return (
    <ToggleGroup
      disabled={props.disabled}
      class={cx("flex-row p-0.5 gap-0.5 rounded-(--radius-control) bg-control-surface disabled:opacity-50", props.class)}
    >
      <Each of={props.options} key={(o) => o.value}>
        {(option) => (
          <Toggle
            pressed={props.value === option().value}
            onChange={() => props.onChange(option().value)}
            class={cx(
              "flex-1 h-5.5 px-3 items-center justify-center rounded-(--radius-keycap)",
              "hover:bg-hover aria-pressed:bg-background",
              "focus-visible:outline-2 focus-visible:outline-ring",
              transition.colors(),
            )}
          >
            <UIText variant="callout" lines={1}>
              {option().label}
            </UIText>
          </Toggle>
        )}
      </Each>
    </ToggleGroup>
  );
}

export interface SelectOption {
  value: string;
  title: string;
  disabled?: boolean;
}

export interface SelectProps {
  options: readonly SelectOption[];
  value: string | null;
  onChange: (value: string) => void;
  accessibilityLabel: string;
  placeholder?: string;
  size?: "sm" | "md";
  disabled?: boolean;
  /** Layout classes only (a width). */
  class?: string;
  /** Content before the title in the trigger (an icon). */
  leading?: JSX.Element;
}

/** A pop-up button (类型筛选, 字段筛选, 主题, 语言, 更新通道). */
export function Select(props: SelectProps) {
  const title = (): string => props.options.find((o) => o.value === props.value)?.title ?? props.placeholder ?? "";
  return (
    <BaseSelect
      items={props.options.map((o) => ({ value: o.value, title: o.title, disabled: o.disabled }))}
      value={props.value}
      placeholder={props.placeholder}
      disabled={props.disabled}
      accessibilityLabel={props.accessibilityLabel}
      onChange={({ value }) => {
        if (value !== null) props.onChange(value);
      }}
      class={props.class}
      popupClass={popupClass}
      itemClass="h-6 px-2 justify-center rounded-(--radius-keycap) text-(length:--fs-body) text-foreground focus:bg-primary focus:text-primary-foreground selected:font-medium disabled:opacity-40"
      trigger={
        <View
          class={cx(
            "flex-row items-center gap-1.5 rounded-(--radius-control) border border-input bg-background",
            "hover:bg-hover aria-expanded:bg-selection",
            props.size === "md" ? "h-7 px-2.5" : "h-6 px-2",
            transition.colors(),
          )}
        >
          {props.leading}
          <UIText variant={props.size === "md" ? "body" : "callout"} lines={1} class="flex-1">
            {title()}
          </UIText>
          <Icon name="chevronDown" size={9} tone="secondary" />
        </View>
      }
    />
  );
}
