import { Show } from "solid-js";
import ChunksComp from "../ChunksComp.tsx";
import SvgIcon from "../ui/SvgIcon.tsx";
import {
  getReferenceTitleText,
  getReferenceTypeKey,
  getReferenceVenue,
  TYPE_STYLES,
} from "../reference/semantic.ts";
import type { Reference } from "../../types.ts";
import { MetadataChip } from "./ReferenceMetadata.tsx";

interface ReferenceRowProps {
  reference: Reference;
  selected?: boolean;
  onSelect: () => void;
}

export default function ReferenceRow(props: ReferenceRowProps) {
  const typeStyle = () => TYPE_STYLES[getReferenceTypeKey(props.reference.type_)];
  const titleText = () => getReferenceTitleText(props.reference);
  const authors = () => props.reference.author?.join(", ");
  const venue = () => getReferenceVenue(props.reference);

  return (
    <button
      type="button"
      class={[
        "group grid w-full grid-cols-[auto_minmax(0,1fr)] gap-3 border-l-4 border-b border-base-300/70 px-3 py-3 text-left transition-colors",
        typeStyle().borderClass,
        props.selected
          ? "bg-primary/10"
          : "bg-base-100 hover:bg-base-200/70",
      ].join(" ")}
      onClick={props.onSelect}
    >
      <div class="pt-0.5">
        <span class={typeStyle().badgeClass}>
          <SvgIcon name={typeStyle().icon} class="h-3.5 w-3.5" aria-hidden />
          {typeStyle().label}
        </span>
      </div>

      <div class="min-w-0">
        <div class="line-clamp-2 min-h-10 text-sm font-semibold leading-5 text-base-content">
          <Show
            when={props.reference.title?.length}
            fallback={<span>{titleText()}</span>}
          >
            <ChunksComp
              chunks={props.reference.title ?? []}
              citeKey={props.reference.cite_key}
            />
          </Show>
        </div>
        <div class="mt-2 flex min-w-0 flex-wrap gap-1.5">
          <MetadataChip kind="citeKey" text={props.reference.cite_key} />
          <MetadataChip kind="author" text={authors()} />
          <MetadataChip kind="year" text={props.reference.year} />
          <MetadataChip kind="venue" text={venue()} />
        </div>
      </div>
    </button>
  );
}

