import type { JSX } from "nsh";
import { Text, View } from "nsh";
import { cx } from "./cx";

/** The type ramp of the macOS client (body 13, callout 12, caption 10, ...). */
export type TextVariant =
  | "body"
  | "callout"
  | "caption"
  | "small"
  | "headline"
  | "title3"
  | "title2"
  | "hero"
  | "mono"
  | "monoSmall"
  | "monoCaption";

export type TextTone = "primary" | "secondary" | "tertiary" | "accent" | "success" | "danger" | "link" | "inherit";
export type TextWeight = "regular" | "medium" | "semibold";

const variantClass: Record<TextVariant, string> = {
  body: "text-(length:--fs-body)",
  callout: "text-(length:--fs-callout)",
  caption: "text-(length:--fs-caption)",
  small: "text-(length:--fs-small)",
  headline: "text-(length:--fs-body) font-semibold",
  title3: "text-(length:--fs-title3)",
  title2: "text-(length:--fs-title2)",
  hero: "text-(length:--fs-hero)",
  mono: "font-mono text-(length:--fs-body)",
  monoSmall: "font-mono text-(length:--fs-small)",
  monoCaption: "font-mono text-(length:--fs-caption)",
};

const toneClass: Record<TextTone, string> = {
  primary: "text-foreground",
  secondary: "text-muted-foreground",
  tertiary: "text-tertiary-foreground",
  accent: "text-primary",
  success: "text-success",
  danger: "text-destructive",
  link: "text-link",
  inherit: "",
};

const weightClass: Record<TextWeight, string> = {
  regular: "",
  medium: "font-medium",
  semibold: "font-semibold",
};

export interface UITextProps {
  variant?: TextVariant;
  tone?: TextTone;
  weight?: TextWeight;
  /** The most lines shown; 0 or unset shows all. */
  lines?: number;
  /** Only for content people copy (field values, BibTeX); fixed at creation. */
  selectable?: boolean;
  /** Layout classes only (flex, width, margins); never colors or sizes. */
  class?: string;
  accessibilityLabel?: string;
  children?: JSX.Element;
}

/** Interface text in the app's type ramp and semantic tones. */
export function UIText(props: UITextProps) {
  const text = (
    <Text
      class={cx(
        variantClass[props.variant ?? "body"],
        toneClass[props.tone ?? "primary"],
        props.weight ? weightClass[props.weight] : "",
        props.selectable ? "" : props.class,
      )}
      style={{ numberOfLines: props.lines ?? 0 }}
      accessibilityLabel={props.accessibilityLabel}
    >
      {props.children}
    </Text>
  );
  // Selection belongs to the view that holds the text, never to a whole pane.
  return props.selectable ? (
    <View selectable class={props.class}>
      {text}
    </View>
  ) : (
    text
  );
}
