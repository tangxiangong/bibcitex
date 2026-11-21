import { platform, signal } from "nsh";

export type DesktopOS = "macos" | "windows" | "linux";

/** The desktop family the window chrome, materials and shortcut labels follow;
 * the web and anything else are treated as Linux (solid surfaces). */
export const desktopOS: DesktopOS =
  platform.os === "macos" ? "macos" : platform.os === "windows" ? "windows" : "linux";

export const isMac = desktopOS === "macos";
export const isWindows = desktopOS === "windows";
export const isLinux = desktopOS === "linux";

function majorVersion(): number {
  const major = Number.parseInt(platform.version.split(".")[0] ?? "0", 10);
  return Number.isFinite(major) ? major : 0;
}

/** Liquid Glass (`glassEffect`) is drawn on macOS 26 and later. */
export const liquidGlass = isMac && majorVersion() >= 26;

/**
 * Whether translucent window backgrounds render on this Linux session. Not
 * every compositor supports transparent windows, so surfaces stay solid
 * until the shell reports otherwise (`setLinuxTranslucency(true)`).
 */
const linuxTranslucency = signal(false);
export const translucentLinux = (): boolean => isLinux && linuxTranslucency();
export function setLinuxTranslucency(supported: boolean): void {
  linuxTranslucency.set(supported);
}
