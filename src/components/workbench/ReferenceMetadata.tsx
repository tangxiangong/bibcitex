import { Show } from "solid-js";
import SvgIcon, { type SvgIconName } from "../ui/SvgIcon.tsx";
import { METADATA_STYLES } from "../reference/semantic.ts";

type MetadataKind = keyof typeof METADATA_STYLES;

interface MetadataChipProps {
  kind: MetadataKind;
  text?: string | number | null;
  title?: string;
  class?: string;
}

interface MetadataRowProps {
  kind: MetadataKind;
  label: string;
  value?: string | number | null;
  icon?: SvgIconName;
  children?: import("solid-js").JSX.Element;
}

export function MetadataChip(props: MetadataChipProps) {
  const style = () => METADATA_STYLES[props.kind];
  const text = () => props.text;

  return (
    <Show when={text() !== undefined && text() !== null && `${text()}` !== ""}>
      <span
        class={[
          "inline-flex min-w-0 max-w-full items-center gap-1.5 rounded px-2 py-0.5 text-xs leading-5",
          style().chipClass,
          props.class,
        ]
          .filter(Boolean)
          .join(" ")}
        title={props.title ?? `${text()}`}
      >
        <SvgIcon name={style().icon} class="h-3.5 w-3.5 shrink-0" aria-hidden />
        <span class="min-w-0 truncate">{text()}</span>
      </span>
    </Show>
  );
}

export function MetadataRow(props: MetadataRowProps) {
  const style = () => METADATA_STYLES[props.kind];
  const iconName = () => props.icon ?? style().icon;

  return (
    <Show
      when={
        props.children !== undefined ||
        (props.value !== undefined && props.value !== null && `${props.value}` !== "")
      }
    >
      <div class="grid grid-cols-[7rem_minmax(0,1fr)] gap-3 border-b border-base-300/70 px-4 py-3 last:border-b-0">
        <div class="flex min-w-0 items-center gap-2 text-xs font-medium uppercase tracking-wide text-base-content/55">
          <SvgIcon name={iconName()} class="h-3.5 w-3.5 shrink-0" aria-hidden />
          <span class="truncate">{props.label}</span>
        </div>
        <div class={["min-w-0 text-sm", style().textClass].join(" ")}>
          <Show when={props.children} fallback={<span>{props.value}</span>}>
            {props.children}
          </Show>
        </div>
      </div>
    </Show>
  );
}

