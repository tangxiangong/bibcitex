import type { JSX } from "nsh";
import { token, View } from "nsh";
import {
  ContextMenu as BaseContextMenu,
  Dialog,
  Menu as BaseMenu,
  MenuItem,
  MenuSeparator,
  Popover,
} from "nsh/base";
import { popupMotion, transition } from "../theme/motion";
import { cx } from "./cx";
import { Icon, type IconName } from "./Icon";
import { Separator } from "./Separator";
import { popupClass } from "./Surface";
import { UIText } from "./Typography";

export interface DialogFrameProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  /** The sheet's width in points (`dialogs` in theme/metrics). */
  width: number;
  height?: number;
  title?: string;
  /** Saving in progress: Escape and backdrop presses do not close it. */
  busy?: boolean;
  /** A leading mark beside the title (the library editor's folder tile). */
  mark?: JSX.Element;
  subtitle?: string;
  /** Right-aligned buttons, the default commit last. */
  footer?: JSX.Element;
  children?: JSX.Element;
}

/** A centered sheet over the window: title, content, a separated footer. */
export function DialogFrame(props: DialogFrameProps) {
  return (
    <Dialog
      open={props.open}
      onOpenChange={({ open }) => props.onOpenChange(open)}
      closeOnEscape={!props.busy}
      closeOnBackdropPress={!props.busy}
      animation="fade"
      accessibilityLabel={props.title}
      class="m-auto bg-background border border-panel-border rounded-(--radius-popover) p-7"
      style={{ width: props.width, height: props.height, boxShadow: [{ x: 0, y: 12, blur: 32, color: token("shadow") }] }}
      backdropClass="bg-black/20"
    >
      <If when={props.title}>
        <View class="flex-row items-start gap-3.5 pb-6">
          {props.mark}
          <View class="flex-1 gap-1.5 pt-0.75">
            <UIText variant="title2" weight="semibold">
              {props.title}
            </UIText>
            <If when={props.subtitle}>
              <UIText variant="callout" tone="secondary">
                {props.subtitle}
              </UIText>
            </If>
          </View>
        </View>
      </If>
      <View class="flex-1 gap-5">{props.children}</View>
      <If when={props.footer}>
        <Separator class="mt-6 mb-4" />
        <View class="flex-row justify-end gap-3">{props.footer}</View>
      </If>
    </Dialog>
  );
}

export interface PopoverFrameProps {
  trigger: JSX.Element;
  open: boolean;
  onOpenChange: (open: boolean) => void;
  placement?: "top" | "bottom" | "left" | "right";
  /** The popup's width in points. */
  width?: number;
  maxHeight?: number;
  children?: JSX.Element;
}

/** A popover in the popup family's surface that grows out of its trigger. */
export function PopoverFrame(props: PopoverFrameProps) {
  return (
    <Popover
      open={props.open}
      onOpenChange={({ open }) => props.onOpenChange(open)}
      placement={props.placement ?? "bottom"}
      offset={6}
      margin={8}
      trigger={props.trigger}
      popupClass={cx(popupClass, "p-3.5", transition.popup())}
      popupStyle={{ width: props.width, maxHeight: props.maxHeight }}
      popupStartingClass={popupMotion.starting()}
      popupEndingClass={popupMotion.ending()}
    >
      {props.children}
    </Popover>
  );
}

/** One entry of a menu: an action or a separator. */
export type MenuEntry =
  | { kind: "separator" }
  | { kind?: "action"; label: string; icon?: IconName; destructive?: boolean; disabled?: boolean; onSelect: () => void };

const itemClass =
  "flex-row items-center gap-2 h-6 px-2 rounded-(--radius-keycap) focus:bg-primary hover:bg-primary disabled:opacity-40 group";

function Items(props: { entries: readonly MenuEntry[] }) {
  return (
    <Each of={props.entries}>
      {(entry) => {
        const e = entry();
        if (e.kind === "separator") return <MenuSeparator class="h-px my-1 mx-1.5 bg-separator" />;
        return (
          <MenuItem class={itemClass} disabled={e.disabled} onSelect={e.onSelect} accessibilityLabel={e.label}>
            <If when={e.icon}>{(name) => <Icon name={name()} size={14} tone={e.destructive ? "danger" : "primary"} />}</If>
            <UIText tone={e.destructive ? "danger" : "primary"} class="group-focus:text-primary-foreground group-hover:text-primary-foreground" lines={1}>
              {e.label}
            </UIText>
          </MenuItem>
        );
      }}
    </Each>
  );
}

/** A menu opened by its trigger (a row's “more” button). */
export function ActionMenu(props: {
  trigger: JSX.Element;
  entries: readonly MenuEntry[];
  open?: boolean;
  onOpenChange?: (open: boolean) => void;
}) {
  return (
    <BaseMenu
      trigger={props.trigger}
      open={props.open}
      onOpenChange={({ open }) => props.onOpenChange?.(open)}
      popupClass={cx(popupClass, "min-w-44", transition.popup())}
      popupStartingClass={popupMotion.starting()}
      popupEndingClass={popupMotion.ending()}
    >
      <Items entries={props.entries} />
    </BaseMenu>
  );
}

/** A right-click (Shift+F10, Menu key) menu about its trigger. */
export function ContextMenu(props: { trigger: JSX.Element; entries: readonly MenuEntry[]; disabled?: boolean }) {
  return (
    <BaseContextMenu
      trigger={props.trigger}
      disabled={props.disabled}
      popupClass={cx(popupClass, "min-w-44", transition.popup())}
      popupStartingClass={popupMotion.starting()}
      popupEndingClass={popupMotion.ending()}
    >
      <Items entries={props.entries} />
    </BaseContextMenu>
  );
}
