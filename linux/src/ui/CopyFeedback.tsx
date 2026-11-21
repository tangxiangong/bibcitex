import { createMotion, View } from "nsh";
import { Button as BaseButton, Tooltip } from "nsh/base";
import { tr } from "../i18n";
import { duration, transition } from "../theme/motion";
import { cx } from "./cx";
import { Icon, type IconName } from "./Icon";
import { tooltipClass } from "./Tooltip";
import { UIText } from "./Typography";

/**
 * Where a copy happened. Each surface keeps its own confirmation, so copying
 * from the list never lights up the inspector and the reverse.
 */
export type CopySurface = "list" | "detail" | "batch" | "helper" | "tray";

function Copied(props: { iconSize: number; showLabel: boolean }) {
  const motion = createMotion({ enter: "zoomIn", duration: duration.fast });
  return (
    <View class="flex-row items-center gap-1" style={motion.style}>
      <Icon name="check" size={props.iconSize} tone="success" />
      <If when={props.showLabel}>
        <UIText variant="caption" tone="success" lines={1}>
          {tr("已复制")}
        </UIText>
      </If>
    </View>
  );
}

export interface CopyButtonProps {
  /** Whether this surface's last copy was this value (the store decides). */
  copied: boolean;
  /** The idle glyph: `copy` for a citation, `clipboard` for BibTeX. */
  icon: IconName;
  /** The action's name (引用 / 复制 BibTeX). */
  label: string;
  onPress: () => void;
  /** Show “已复制” beside the check (default true). */
  showLabel?: boolean;
  class?: string;
}

/** A borderless copy action that answers on itself: its glyph turns into a
 * check and “已复制” for a moment, with no alert to dismiss. */
export function CopyButton(props: CopyButtonProps) {
  return (
    <BaseButton
      class={cx("flex-row items-center rounded-(--radius-keycap) px-1 h-6 hover:bg-hover", transition.colors(), props.class)}
      accessibilityLabel={props.label}
      onPress={props.onPress}
      tooltip={
        <Tooltip class={tooltipClass}>
          <UIText variant="small">{props.label}</UIText>
        </Tooltip>
      }
    >
      <If when={props.copied} else={<Icon name={props.icon} size={14} tone="secondary" />}>
        <Copied iconSize={12} showLabel={props.showLabel ?? true} />
      </If>
    </BaseButton>
  );
}

export interface CiteKeyChipProps {
  citeKey: string;
  copied: boolean;
  onPress: () => void;
  /** Widest it gets before the key truncates (the tray caps it at 160). */
  maxWidthClass?: string;
}

/** The cite key on a result row, pinned top right: pressing copies the
 * citation and the chip itself confirms it. */
export function CiteKeyChip(props: CiteKeyChipProps) {
  return (
    <BaseButton
      class={cx("flex-row items-center gap-1 rounded-full px-1.75 py-0.75 bg-control-surface active:opacity-70", transition.opacity(), props.maxWidthClass)}
      accessibilityLabel={tr("引用 {0}", props.citeKey)}
      onPress={props.onPress}
      tooltip={
        <Tooltip class={tooltipClass}>
          <UIText variant="small">{tr("引用")}</UIText>
        </Tooltip>
      }
    >
      <If
        when={props.copied}
        else={
          <View class="flex-row items-center gap-1">
            <Icon name="copy" size={10} tone="accent" />
            <UIText variant="monoCaption" tone="accent" lines={1}>
              {props.citeKey}
            </UIText>
          </View>
        }
      >
        <Copied iconSize={10} showLabel />
      </If>
    </BaseButton>
  );
}
