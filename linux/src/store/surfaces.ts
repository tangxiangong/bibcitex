import { effect, signal, untracked, type Bounds } from "nsh";
import { Application, GlobalShortcut, Tray, Window, type WindowRef } from "nsh/desktop";
import { Shortcut, type DesktopAction, type DesktopEvent } from "bibcitex/native";
import { tr } from "../i18n";
import trayIcon from "../../.nsh/resources/favicon.png";
import {
  desktop,
  desktopCapabilities,
  desktopPlatform,
  prefs,
  savePreferences,
  setActivationHandler,
  setPasteFailureHandler,
  showError,
} from "./desktop";
import { errorText } from "./format";
import { checkUpdates, updatesAvailable } from "./updater";

const helperOpen = signal(false);
const trayOpen = signal(false);
const settingsOpen = signal(false);
const aboutOpen = signal(false);
/** Where the tray panel goes: beside the icon when the system tells. */
const trayAnchor = signal<{ x: number; top: number; bottom: number } | null>(null);

export const helperVisible = () => helperOpen();
export const trayVisible = () => trayOpen();
export const settingsVisible = () => settingsOpen();
export const aboutVisible = () => aboutOpen();
export const trayPanelAnchor = () => trayAnchor();

interface HelperHooks {
  /** Captures the target; resolves false when the panel was dismissed meanwhile. */
  begin(): Promise<boolean>;
  end(): void;
  showFailure(ticket: string, message: string): void;
}

let helperHooks: HelperHooks | null = null;

/** Connects the helper's transaction to the panel's showing and hiding. */
export function bindHelper(hooks: HelperHooks): void {
  helperHooks = hooks;
  setPasteFailureHandler((ticket, message) => hooks.showFailure(ticket, tr("粘贴失败：{0}", tr(message))));
}

// The windows' refs, as each opening hands them over. Activation waits for
// these before it gives a window the compositor's token or places it.
type SurfaceId = "main" | "helper" | "tray" | "settings" | "about";
const openWaiters = new Map<SurfaceId, ((ref: WindowRef) => void)[]>();

/** Called from a `<Window>`'s `ref`: the window of `id` opened. */
export function windowOpened(id: SurfaceId, ref: WindowRef): void {
  const waiting = openWaiters.get(id) ?? [];
  openWaiters.delete(id);
  for (const resolve of waiting) resolve(ref);
}

function nextFrame(): Promise<void> {
  return new Promise((resolve) => requestAnimationFrame(() => resolve()));
}

/** The window of `id` once it is open and mapped on screen, or `null` when
 * it closed (or never showed) within `timeoutMs`. */
async function visibleWindow(id: SurfaceId, isOpen: () => boolean, timeoutMs = 2000): Promise<WindowRef | null> {
  const deadline = Date.now() + timeoutMs;
  // A window that closed earlier hands over a new ref when it opens again.
  let ref: WindowRef | null = Window.get(id);
  if (!ref) {
    ref = await Promise.race([
      new Promise<WindowRef>((resolve) => openWaiters.set(id, [...(openWaiters.get(id) ?? []), resolve])),
      new Promise<null>((resolve) => setTimeout(() => resolve(null), timeoutMs)),
    ]);
  }
  while (ref && Date.now() < deadline) {
    if (!isOpen()) return null;
    if (ref.state()?.visible) return ref;
    await nextFrame();
  }
  return null;
}

export function showMain(): void {
  const main = Window.get("main");
  if (!main) return;
  main.show();
  main.focus();
}

// Each show or hide of the helper takes a generation: a show whose capture
// resolves after a later hide or toggle is abandoned.
let helperGeneration = 0;
let helperPending = false;

/** Captures the target app first, then shows the compact helper. Resolves
 * whether the helper ended up open. */
export async function showHelper(): Promise<boolean> {
  if (untracked(helperOpen)) return true;
  if (helperPending) return false;
  const mine = ++helperGeneration;
  helperPending = true;
  try {
    const ok = (await helperHooks?.begin()) ?? true;
    if (!ok || mine !== helperGeneration) return false;
    helperOpen.set(true);
    return true;
  } finally {
    if (mine === helperGeneration) helperPending = false;
  }
}

/** Shows the helper again after a failed paste, keeping its error. */
export function reopenHelper(): void {
  helperGeneration++;
  helperPending = false;
  helperOpen.set(true);
}

export function hideHelper(): void {
  const wasOpen = untracked(helperOpen);
  const wasPending = helperPending;
  helperGeneration++;
  helperPending = false;
  if (!wasOpen && !wasPending) return;
  helperOpen.set(false);
  helperHooks?.end();
}

/** Opens the helper, or closes it when it is open or opening. Resolves
 * whether it is open afterwards. */
export async function toggleHelper(): Promise<boolean> {
  if (untracked(helperOpen) || helperPending) {
    hideHelper();
    return false;
  }
  return showHelper();
}

/** The tray panel beside the icon's frame: below a top menu bar, above a
 * bottom taskbar. */
export function showTray(bounds?: Bounds | null): void {
  if (bounds) {
    trayAnchor.set({ x: bounds.origin.x + bounds.size.width / 2, top: bounds.origin.y, bottom: bounds.origin.y + bounds.size.height });
  }
  trayOpen.set(true);
}

export function hideTray(): void {
  trayOpen.set(false);
}

/** Resolves whether the tray panel is open afterwards. */
export function toggleTray(bounds?: Bounds | null): boolean {
  if (untracked(trayOpen)) {
    hideTray();
    return false;
  }
  showTray(bounds);
  return true;
}

export function setSettingsOpen(open: boolean): void {
  settingsOpen.set(open);
  if (open) Window.get("settings")?.focus();
}

export function setAboutOpen(open: boolean): void {
  aboutOpen.set(open);
  if (open) Window.get("about")?.focus();
}

export function checkForUpdates(): void {
  setSettingsOpen(true);
  checkUpdates().catch(showError);
}

/** Carries out one command, wherever it came from. Resolves whether the
 * surface it names is open afterwards (a toggle may close it). */
export async function perform(action: DesktopAction): Promise<boolean> {
  switch (action) {
    case "main":
      showMain();
      return true;
    case "helper":
      return toggleHelper();
    case "tray":
      return toggleTray();
    case "settings":
      setSettingsOpen(true);
      return true;
    case "about":
      setAboutOpen(true);
      return true;
    case "checkUpdates":
      checkForUpdates();
      return true;
    case "quit":
      Application.quit();
      return false;
    default:
      showMain();
      return true;
  }
}

const isOpen: Record<SurfaceId, () => boolean> = {
  main: () => true,
  helper: () => untracked(helperOpen),
  tray: () => untracked(trayOpen),
  settings: () => untracked(settingsOpen),
  about: () => untracked(aboutOpen),
};

async function activate(event: DesktopEvent): Promise<void> {
  const action = event.surface ?? "main";
  const open = await perform(action);
  const surface: SurfaceId | null =
    action === "main" || action === "helper" || action === "tray" || action === "settings" || action === "about"
      ? action
      : action === "checkUpdates"
        ? "settings"
        : null;
  // Nothing to focus or place when the command closed its surface.
  if (!open || !surface) return;
  const needsToken = !!event.token;
  const needsPlacement = surface === "tray" && event.x != null && event.y != null;
  if (!needsToken && !needsPlacement) return;
  const win = await visibleWindow(surface, isOpen[surface]);
  if (!win || !isOpen[surface]()) return;
  // The status notifier's anchor places the tray panel where X11 allows it;
  // the compositor's token lets the surface take the focus on Wayland.
  if (needsPlacement) await desktop.placeTray(event.x!, event.y!).catch(() => false);
  if (event.token && isOpen[surface]()) await desktop.activateWayland(event.token, surface).catch(() => undefined);
}

// Activations run one after another, each to its end, in arrival order.
let activations: Promise<void> = Promise.resolve();
function onActivate(event: DesktopEvent): void {
  activations = activations.then(() => activate(event)).catch(() => undefined);
}

// The global shortcut, when this process (not Rust) owns it.
let registered: string | null = null;
const shortcutLive = signal(false);

/** Whether this process holds the helper shortcut right now (macOS and
 * Windows; on Linux Rust owns it and this stays false). Reactive. */
export const shortcutRegistered = () => shortcutLive();

/** The stored accelerator (`Cmd+Shift+K`) as a keystroke (`cmd+shift+k`). */
function keystroke(value: string): string {
  return value.toLowerCase().replace(/\s+/g, "");
}

const tsxOwnsShortcut = (): boolean => {
  const p = desktopPlatform();
  return !!p && !p.shortcutManagedByRust && !!desktopCapabilities()?.shortcut;
};

async function register(value: string): Promise<boolean> {
  try {
    return await GlobalShortcut.register(keystroke(value), () => void toggleHelper());
  } catch {
    return false;
  }
}

/** While the recorder listens, the shortcut it may record does not fire. */
export function pauseShortcut(paused: boolean): void {
  if (!tsxOwnsShortcut() || !registered) return;
  if (paused) {
    GlobalShortcut.unregister(keystroke(registered));
    shortcutLive.set(false);
  } else {
    const value = registered;
    void register(value).then((ok) => {
      if (registered === value) shortcutLive.set(ok);
    });
  }
}

/**
 * Validates and applies a new helper shortcut: the new key is registered
 * first, the setting saved, and only then the old key given back. A failed
 * step rolls back to the old key. Rejects with a catalog key.
 */
export async function changeShortcut(value: string): Promise<void> {
  let parsed: string;
  try {
    parsed = Shortcut.parse(value).value;
  } catch (e) {
    throw new Error(errorText(e));
  }
  if (!tsxOwnsShortcut()) {
    await savePreferences({ helperShortcut: parsed });
    return;
  }
  const previous = registered;
  if (previous === parsed) return;
  if (!(await register(parsed))) {
    shortcutLive.set(previous ? await register(previous) : false);
    throw new Error(tr("error.helperShortcutUnavailable"));
  }
  try {
    await savePreferences({ helperShortcut: parsed });
  } catch (e) {
    GlobalShortcut.unregister(keystroke(parsed));
    shortcutLive.set(previous ? await register(previous) : false);
    throw e;
  }
  if (previous && keystroke(previous) !== keystroke(parsed)) GlobalShortcut.unregister(keystroke(previous));
  registered = parsed;
  shortcutLive.set(true);
}

/**
 * Installs the process-wide integrations this host leaves to TSX (the tray
 * and the global shortcut on macOS and Windows; Rust owns both on Linux) and
 * routes activations and later launches. Call once, in the root component.
 */
export function installDesktopIntegrations(): void {
  // Later launches reach this process only as `activate` events: Rust's
  // activation service owns the second-instance channel.
  setActivationHandler(onActivate);

  // Tray: one owner. Labels follow the language.
  effect(() => {
    const p = desktopPlatform();
    const caps = desktopCapabilities();
    const enabled = prefs()?.trayIconEnabled ?? false;
    if (!p || p.trayManagedByRust) return;
    if (!enabled || !caps?.tray) {
      Tray.set(null);
      return;
    }
    Tray.set({
      icon: trayIcon,
      tooltip: "BibCiTeX",
      onClick: ({ bounds }) => toggleTray(bounds),
      menu: [
        { label: tr("显示窗口"), onSelect: showMain },
        { label: tr("快捷助手"), onSelect: () => void showHelper() },
        { label: tr("检查更新"), onSelect: checkForUpdates, disabled: !updatesAvailable() },
        { type: "separator" },
        { label: tr("退出 BibCiTeX"), onSelect: () => Application.quit() },
      ],
    });
  });

  // Global shortcut: registered once the saved value is known.
  effect(() => {
    if (!tsxOwnsShortcut()) return;
    const saved = prefs()?.helperShortcut;
    if (!saved) return;
    untracked(() => {
      if (registered !== null) return;
      registered = saved;
      void register(saved).then((ok) => {
        if (registered !== saved) return;
        shortcutLive.set(ok);
        if (!ok) registered = null;
      });
    });
  });
}
