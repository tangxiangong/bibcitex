import { Image, Text } from "nsh";
import { cx } from "./cx";
import type { TextTone, TextVariant } from "./Typography";

/** A run of a rich field as the backend splits it. */
export interface TextChunk {
  kind: "normal" | "verbatim" | "math";
  text: string;
}

/** A search match, in UTF-16 code units over the chunks' texts joined in order. */
export interface HighlightRange {
  start: number;
  end: number;
}

/** A typeset formula: an SVG `data:` URL and its size in points at the requested font size. */
export interface MathFormula {
  dataUrl: string;
  width: number;
  height: number;
}

interface Segment {
  text: string;
  kind: TextChunk["kind"];
  highlighted: boolean;
}

/** Splits chunks at highlight boundaries; math is never split. Exported for tests. */
export function segments(chunks: readonly TextChunk[], ranges: readonly HighlightRange[]): Segment[] {
  const out: Segment[] = [];
  let offset = 0;
  for (const chunk of chunks) {
    const start = offset;
    const end = offset + chunk.text.length;
    offset = end;
    if (chunk.kind === "math") {
      out.push({ text: chunk.text, kind: "math", highlighted: false });
      continue;
    }
    const cuts = new Set<number>([start, end]);
    for (const r of ranges) {
      if (r.end <= start || r.start >= end) continue;
      cuts.add(Math.max(r.start, start));
      cuts.add(Math.min(r.end, end));
    }
    const points = [...cuts].sort((a, b) => a - b);
    for (let i = 0; i + 1 < points.length; i++) {
      const a = points[i];
      const b = points[i + 1];
      if (a === b) continue;
      out.push({
        text: chunk.text.slice(a - start, b - start),
        kind: chunk.kind,
        highlighted: ranges.some((r) => r.start <= a && r.end >= b),
      });
    }
  }
  return out;
}

const variantClass: Partial<Record<TextVariant, string>> = {
  body: "text-(length:--fs-body)",
  callout: "text-(length:--fs-callout)",
  caption: "text-(length:--fs-caption)",
  small: "text-(length:--fs-small)",
  title3: "text-(length:--fs-title3)",
};

const toneClass: Partial<Record<TextTone, string>> = {
  primary: "text-foreground",
  secondary: "text-muted-foreground",
};

export interface MathTextProps {
  chunks: readonly TextChunk[];
  highlights?: readonly HighlightRange[];
  /** The typeset formula for a TeX source, `null` when it failed (the source
   * shows instead), `undefined` while it is being typeset. */
  formula: (tex: string) => MathFormula | null | undefined;
  variant?: "body" | "callout" | "caption" | "small" | "title3";
  tone?: "primary" | "secondary";
  weight?: "regular" | "medium" | "semibold";
  lines?: number;
  class?: string;
}

/**
 * Text with inline mathematics, in the order the backend gives the chunks:
 * normal text, verbatim text in monospace, and formulas as images in the
 * line. Search matches are tinted; a formula that cannot be typeset shows its
 * TeX source.
 */
export function MathText(props: MathTextProps) {
  return (
    <Text
      class={cx(
        variantClass[props.variant ?? "body"],
        toneClass[props.tone ?? "primary"],
        props.weight === "semibold" ? "font-semibold" : props.weight === "medium" ? "font-medium" : "",
        props.class,
      )}
      style={{ numberOfLines: props.lines ?? 0 }}
    >
      <Each of={segments(props.chunks, props.highlights ?? [])}>
        {(segment) =>
          segment().kind === "math" ? (
            <MathRun tex={segment().text} formula={props.formula} />
          ) : (
            <Text
              class={cx(segment().kind === "verbatim" ? "font-mono" : "", segment().highlighted ? "bg-highlight" : "")}
            >
              {segment().text}
            </Text>
          )
        }
      </Each>
    </Text>
  );
}

function MathRun(props: { tex: string; formula: MathTextProps["formula"] }) {
  const result = () => props.formula(props.tex);
  return (
    <If when={result()} else={<Text class="font-mono">{props.tex}</Text>}>
      {(f) => (
        <Image
          src={f().dataUrl}
          accessibilityLabel={props.tex}
          style={{ width: f().width, height: f().height, verticalAlign: "middle", tintColor: undefined }}
        />
      )}
    </If>
  );
}
