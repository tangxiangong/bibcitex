import type { JSX, ViewStyle } from "nsh";
import { token, View } from "nsh";
import type { WindowAttributes } from "nsh/desktop";
import { cx } from "./cx";
import { isLinux, isMac, isWindows, liquidGlass, translucentLinux } from "./platform";

/**
 * - `pane`: the reference list, settings and form backgrounds.
 * - `sidebar`: the library list and the inspector (macOS sidebar material).
 * - `panel` / `trayPanel`: the borderless helper (r26) and tray (r20) windows.
 * - `capsule`: floating controls over content (helper key hints, batch bar).
 * - `card`: a framed group inside a pane (the library editor's file card).
 * - `detail`: the tray's detail sheet over its results.
 * - `popup`: menus, popovers, selects and tooltips share this treatment.
 */
export type SurfaceVariant = "pane" | "sidebar" | "panel" | "trayPanel" | "capsule" | "card" | "detail" | "popup";

export interface SurfaceProps {
  variant: SurfaceVariant;
  /** Layout classes only (flex, size, padding, gap). */
  class?: string;
  accessibilityLabel?: string;
  children?: JSX.Element;
}

/** The popup family's look, for `nsh/base` popups that take a `popupClass`. */
export const popupClass =
  "bg-popover border border-panel-border rounded-(--radius-popover) p-1 overflow-hidden";

function shadow(blur: number, y: number, x = 0): ViewStyle["boxShadow"] {
  return [{ x, y, blur, color: token("shadow") }];
}

function radiusClass(variant: SurfaceVariant): string {
  if (isWindows && (variant === "panel" || variant === "trayPanel")) return "rounded-(--radius-windows)";
  switch (variant) {
    case "panel":
      return "rounded-(--radius-panel)";
    case "trayPanel":
      return "rounded-(--radius-tray)";
    case "capsule":
      return "rounded-full";
    case "card":
      return "rounded-(--radius-card)";
    case "detail":
      return "rounded-(--radius-detail)";
    case "popup":
      return "rounded-(--radius-popover)";
    default:
      return "";
  }
}

/**
 * One container for every material in the app, so platform differences live
 * here alone:
 *
 * - macOS 26+: Liquid Glass on floating surfaces; older macOS: the
 *   `NSVisualEffectView` material behind the window, as the native client.
 * - Windows 11: the window's Mica / Acrylic backdrop shows through a light
 *   tint, with a 1pt highlight border on floating surfaces.
 * - Linux: solid surfaces, or a blurred tint once the compositor is known
 *   to draw transparent windows.
 *
 * Glass is never stacked: only the outermost floating surface carries it.
 */
export function Surface(props: SurfaceProps) {
  const variant = props.variant;
  const floating = variant === "panel" || variant === "trayPanel";

  const look = (): string => {
    switch (variant) {
      case "pane":
        return "bg-background";
      case "sidebar":
        if (isMac) return "";
        if (isWindows) return "bg-sidebar/60";
        return "bg-sidebar";
      case "panel":
      case "trayPanel":
        if (isMac) return "bg-scrim";
        if (isWindows) return "bg-popover/70 border border-panel-border";
        return translucentLinux() ? "bg-popover/85 border border-panel-border" : "bg-popover border border-panel-border";
      case "capsule":
        if (liquidGlass) return "";
        if (isMac) return "bg-control-surface";
        return "bg-popover border border-panel-border";
      case "card":
        return "bg-control-surface border border-separator";
      case "detail":
        return "bg-background border border-separator";
      case "popup":
        return "bg-popover border border-panel-border";
    }
  };

  const style = (): ViewStyle => {
    if (isMac && liquidGlass && (floating || variant === "capsule")) {
      return { glassEffect: "regular", backdropFilter: "blur(30px)", borderCurve: "continuous" };
    }
    if (isMac && floating) return { borderCurve: "continuous" };
    if (isLinux && floating && translucentLinux()) return { backdropFilter: "blur(24px)", boxShadow: shadow(24, 8) };
    if (isLinux && (floating || variant === "capsule")) return { boxShadow: shadow(18, 6) };
    if (variant === "detail") return { borderCurve: "continuous", boxShadow: shadow(12, 4, -3) };
    if (variant === "popup") return { boxShadow: shadow(16, 6) };
    return {};
  };

  // Pre-glass macOS draws the system material behind the window: HUD for the
  // floating panels, the sidebar material for the source list and inspector.
  const macBackdrop =
    isMac && !liquidGlass && floating
      ? { material: "hudWindow" as const, blending: "behindWindow" as const, state: "followsWindowActiveState" as const }
      : isMac && variant === "sidebar"
        ? { material: "sidebar" as const, blending: "behindWindow" as const, state: "followsWindowActiveState" as const }
        : undefined;

  return (
    <View
      class={cx(look(), radiusClass(variant), floating || variant === "detail" || variant === "card" ? "overflow-hidden" : "", props.class)}
      style={style()}
      macos={macBackdrop ? { backdrop: macBackdrop } : undefined}
      accessibilityLabel={props.accessibilityLabel}
    >
      {props.children}
    </View>
  );
}

export type WindowSurface = "main" | "floating" | "dialog";

/**
 * Window attributes that go with a surface: the system material behind the
 * window and whether it is transparent and framed. Floating windows (helper,
 * tray) draw their own rounded `Surface` inside a frameless window.
 */
export function windowMaterial(kind: WindowSurface): Partial<WindowAttributes> {
  if (kind === "main") return { backdrop: isWindows ? "window" : "none" };
  if (kind === "dialog") return { backdrop: isWindows ? "window" : "none" };
  if (isMac) return { frame: "none", transparent: true, backdrop: "none", shadow: true };
  if (isWindows) return { frame: "none", transparent: true, backdrop: "popover", shadow: true };
  return { frame: "none", transparent: translucentLinux(), backdrop: "none", shadow: true };
}
