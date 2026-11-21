import { setColorScheme } from "nsh";

export type Appearance = "system" | "light" | "dark";

/** Applies the Settings appearance (跟随系统 / 浅色 / 深色). Theme tokens,
 * `colorScheme()` and the native chrome follow it. */
export function applyAppearance(mode: Appearance): void {
  setColorScheme(mode);
}
