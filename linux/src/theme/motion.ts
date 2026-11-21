import { LayoutAnimation, reduceMotion } from "nsh";

/** Motion tokens shared by every surface. Values follow the macOS client:
 * panels arrive in 180 ms (ease-out) and leave in 120 ms (ease-in). */
export const duration = {
  instant: 120,
  fast: 180,
  base: 240,
  slow: 360,
  panelEnter: 180,
  panelExit: 120,
  copiedFeedback: 1500,
} as const;

export type DurationName = keyof typeof duration;

/** A duration that collapses to zero while the user asks for reduced motion. */
export function ms(name: DurationName): number {
  return reduceMotion() ? 0 : duration[name];
}

/** Transition classes for state styles; empty under reduced motion so state
 * changes still apply, instantly. Read inside a binding to stay reactive. */
export const transition = {
  colors: () => (reduceMotion() ? "" : "transition-colors duration-120 ease-out"),
  press: () => (reduceMotion() ? "" : "transition duration-180 ease-out active:scale-97"),
  opacity: () => (reduceMotion() ? "" : "transition-opacity duration-180 ease-out"),
  popup: () => (reduceMotion() ? "" : "transition duration-360 ease-out"),
} as const;

/** Starting / ending looks of anchored popups: grow from the trigger and
 * slide 8pt; leave faster than they arrive. Plain fades under reduced motion. */
export const popupMotion = {
  starting: () => (reduceMotion() ? "opacity-0" : "opacity-0 scale-95 -translate-y-2"),
  ending: () => (reduceMotion() ? "opacity-0" : "opacity-0 scale-97 duration-240 ease-in"),
} as const;

/** Animates the frames of the next commit (a pane opening or closing, rows
 * added or removed). Does nothing under reduced motion. */
export function animateNextLayout(name: DurationName = "base"): void {
  if (reduceMotion()) return;
  LayoutAnimation.configureNext(LayoutAnimation.create(duration[name]));
}
