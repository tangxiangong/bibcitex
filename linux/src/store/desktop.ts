import { batch, signal } from "nsh";
import { AlertDialog } from "nsh/base";
import {
  Catalog,
  DesktopSession,
  type DesktopAction,
  type DesktopCapabilities,
  type DesktopEvent,
  type DesktopPlatform,
  type LibraryDto,
  type PreferencePatch,
  type PreferencesDto,
} from "bibcitex/native";
import { setLanguagePreference, tr } from "../i18n";
import { applyAppearance, type Appearance } from "../theme";
import { setLinuxTranslucency } from "../ui";
import { errorText } from "./format";

/** The one handle to the process's desktop services. */
export const desktop = new DesktopSession();

const libraries = signal<readonly LibraryDto[]>([]);
const preferences = signal<PreferencesDto | null>(null);
const platformInfo = signal<DesktopPlatform | null>(null);
const capabilities = signal<DesktopCapabilities | null>(null);
const trayAvailable = signal(true);
const shortcutDescription = signal<string | null>(null);
const tagsRevision = signal(0);
const librariesLoaded = signal(false);

/** Registered bibliographies in registry order (reactive). */
export const libraryList = () => libraries();
/** Whether the registry has been read once (reactive). */
export const librariesReady = () => librariesLoaded();
/** The saved settings, `null` until the desktop has started (reactive). */
export const prefs = () => preferences();
/** Which process owns the tray and the global shortcut (reactive). */
export const desktopPlatform = () => platformInfo();
export const desktopCapabilities = () => capabilities();
/** On Linux: whether a StatusNotifier watcher currently shows the tray. */
export const trayShown = () => trayAvailable();
/** The binding the Wayland shortcut portal chose, when it reports one. */
export const portalShortcutDescription = () => shortcutDescription();
/** Bumped whenever the shared tag catalog or assignments change. */
export const tagsChanged = () => tagsRevision();

export function libraryByUid(uid: string | null | undefined): LibraryDto | undefined {
  return uid ? libraries().find((l) => l.uid === uid) : undefined;
}

export function libraryByName(name: string | null | undefined): LibraryDto | undefined {
  return name ? libraries().find((l) => l.name === name) : undefined;
}

/** Shows an error the way the macOS client does: an alert titled “错误”. */
export function showError(error: unknown): void {
  void AlertDialog.show({
    title: tr("错误"),
    description: errorText(error),
    actions: [{ id: "ok", label: tr("确定"), role: "default" }],
  }).catch(() => undefined);
}

// Events are ordered by a decimal sequence. Data events (what a snapshot
// projects) not newer than the snapshot are already in it. Commands
// (activations, failures, errors) are not in any snapshot: each is consumed
// once, in order, whatever the snapshot's revision; a backlog replayed after
// a hot reload's resubscription is still carried out.
function newer(a: string, b: string): boolean {
  return a.length !== b.length ? a.length > b.length : a > b;
}

const dataKinds: ReadonlySet<DesktopEvent["kind"]> = new Set(["libraries", "tags", "preferences", "trayAvailability"]);
let lastDataSequence: string | null = null;
let lastCommandSequence: string | null = null;
let queued: DesktopEvent[] | null = [];
let activationHandler: ((event: DesktopEvent) => void) | null = null;
let pasteFailureHandler: ((ticket: string, message: string) => void) | null = null;

/** Who carries out `activate` events (windows, commands). */
export function setActivationHandler(handler: (event: DesktopEvent) => void): void {
  activationHandler = handler;
}

/** Who shows a paste that failed after the helper was hidden. */
export function setPasteFailureHandler(handler: (ticket: string, message: string) => void): void {
  pasteFailureHandler = handler;
}

let libraryLoad = 0;
async function reloadLibraries(): Promise<void> {
  const version = ++libraryLoad;
  try {
    const loaded = await Catalog.libraries();
    if (version !== libraryLoad) return;
    batch(() => {
      libraries.set(loaded);
      librariesLoaded.set(true);
    });
  } catch (error) {
    if (version === libraryLoad) showError(tr("加载文献库失败：{0}", errorText(error)));
  }
}

let preferencesLoad = 0;
async function reloadPreferences(): Promise<void> {
  const version = ++preferencesLoad;
  try {
    const loaded = await desktop.preferences();
    if (version !== preferencesLoad) return;
    applyPreferences(loaded);
  } catch {
    // A failed read keeps the last confirmed settings.
  }
}

function applyLanguage(value: string): void {
  setLanguagePreference(value === "en" || value === "zh-Hans" ? value : "system");
}

function applyAppearanceValue(value: string): void {
  applyAppearance(value === "light" || value === "dark" ? (value as Appearance) : "system");
}

// Settings shown = the last confirmed settings with the saves still in
// flight replayed over them, in order. A late reply or a reload never hides
// a later optimistic change, and a failed save drops only its own patch.
let confirmed: PreferencesDto | null = null;
let pendingPatches: { id: number; patch: PreferencePatch }[] = [];
let patchId = 0;

/** Update installation waits until every optimistic preference write has settled. */
export const preferencesPending = (): boolean => {
  preferences();
  return pendingPatches.length > 0;
};
// What the interface really shows, compared with the projection instead of
// with the previous signal value (which an optimistic write already moved).
let appliedLanguage: string | null = null;
let appliedAppearance: string | null = null;

function project(): void {
  if (!confirmed) return;
  let next = confirmed;
  for (const { patch } of pendingPatches) next = optimistic(next, patch);
  preferences.set(next);
  if (appliedLanguage !== next.language) {
    appliedLanguage = next.language;
    applyLanguage(next.language);
  }
  if (appliedAppearance !== next.appearance) {
    appliedAppearance = next.appearance;
    applyAppearanceValue(next.appearance);
  }
}

/** A settings value read or saved by the service; an older revision than
 * the one already confirmed is ignored. */
function applyPreferences(next: PreferencesDto): void {
  if (confirmed && confirmed.revision !== next.revision && !newer(next.revision, confirmed.revision)) return;
  confirmed = next;
  project();
}

function handle(event: DesktopEvent): void {
  if (dataKinds.has(event.kind)) {
    if (lastDataSequence !== null && !newer(event.sequence, lastDataSequence)) return;
    lastDataSequence = event.sequence;
  } else {
    if (lastCommandSequence !== null && !newer(event.sequence, lastCommandSequence)) return;
    lastCommandSequence = event.sequence;
  }
  switch (event.kind) {
    case "libraries":
      void reloadLibraries();
      break;
    case "tags":
      tagsRevision.update((n) => n + 1);
      break;
    case "preferences":
      void reloadPreferences();
      break;
    case "activate":
      activationHandler?.(event);
      break;
    case "trayAvailability":
      trayAvailable.set(event.available ?? true);
      break;
    case "shortcutChanged":
      shortcutDescription.set(event.message ?? null);
      break;
    case "pasteFailed":
      if (event.ticket) pasteFailureHandler?.(event.ticket, event.message ?? "");
      break;
    case "desktopError":
      if (event.message) showError(event.message);
      break;
    default:
      break;
  }
}

let started: Promise<void> | null = null;

/** Starts the desktop services once: subscribes first, then reads the
 * snapshot, then replays the notifications that came after it. */
export function startDesktop(): Promise<void> {
  if (started) return started;
  desktop.onChange((event) => {
    if (queued) queued.push(event);
    else handle(event);
  });
  started = desktop
    .start()
    .then((snapshot) => {
      lastDataSequence = snapshot.revision;
      batch(() => {
        libraries.set(snapshot.libraries);
        librariesLoaded.set(true);
        platformInfo.set(snapshot.platform);
        capabilities.set(snapshot.capabilities);
        trayAvailable.set(snapshot.trayAvailable);
        shortcutDescription.set(snapshot.shortcutDescription ?? null);
        applyPreferences(snapshot.preferences);
      });
      setLinuxTranslucency(snapshot.platform.linuxTranslucencySupported);
      const pending = queued ?? [];
      queued = null;
      for (const event of pending) handle(event);
    })
    .catch((error) => {
      queued = null;
      showError(error);
    });
  return started;
}

// Saves are serialized: the service accepts one at a time, each against
// the revision the previous one produced.
let saveChain: Promise<unknown> = Promise.resolve();

/** Saves a partial change. The visible value changes at once; a failed save
 * reloads the confirmed settings and rejects. */
export function savePreferences(patch: PreferencePatch): Promise<PreferencesDto> {
  const id = ++patchId;
  pendingPatches = [...pendingPatches, { id, patch }];
  project();
  const settle = () => {
    pendingPatches = pendingPatches.filter((p) => p.id !== id);
  };
  const run = saveChain.then(async () => {
    try {
      const revision = confirmed?.revision ?? (await desktop.preferences()).revision;
      const saved = await desktop.savePreferences(patch, revision);
      settle();
      applyPreferences(saved);
      project();
      return saved;
    } catch (error) {
      await reloadPreferences();
      settle();
      project();
      throw error;
    }
  });
  saveChain = run.catch(() => undefined);
  return run;
}

function optimistic(current: PreferencesDto, patch: PreferencePatch): PreferencesDto {
  const next: PreferencesDto = { ...current };
  for (const [key, value] of Object.entries(patch)) {
    if (value === null || value === undefined) continue;
    if (key === "clearLibrary") {
      if (value === true) next.library = null;
    } else if (key === "clearTrayLibrary") {
      if (value === true) next.trayLibrary = null;
    } else (next as unknown as Record<string, unknown>)[key] = value;
  }
  return next;
}

/** Saves and reports a failure in an alert. */
export function savePreference(patch: PreferencePatch): void {
  savePreferences(patch).catch(showError);
}

/** The commands that `activate` events, launches and menus name. */
export type SurfaceCommand = DesktopAction;
