import { cleanup, effect, openUrl, signal, untracked } from "nsh";
import { checkForUpdate, downloadUpdate, installAndRestart, installOnQuit, runningPackage, shellInfo, type AppUpdate } from "nsh/updater";
import { updateChannels, appVersion } from "../../.nsh/generated/config";
import { language } from "../i18n";

export interface UpdatePreferences { updateChannel: string; automaticUpdates: boolean }
export interface UpdateState {
  currentVersion: string;
  channel: string;
  canCheck: boolean;
  checked: boolean;
  unavailableReasonKey: string | null;
  phase: "unavailable" | "idle" | "checking" | "available" | "installing" | "ready" | "error";
  id: string | null;
  version: string | null;
  notes: string | null;
  downloaded: number;
  total: number | null;
  message: string | null;
}

const shell = shellInfo();
const currentVersion = runningPackage()?.version ?? shell.version ?? appVersion;
const packaged = shell.version != null;
let configuration: UpdatePreferences = { updateChannel: "stable", automaticUpdates: false };
let generation = 0;
let found: AppUpdate | null = null;
let operation: Promise<void> | null = null;
let readPreferences: () => UpdatePreferences | null = () => null;
const preferencesSettled = () => readPreferences()?.updateChannel === configuration.updateChannel;

function initial(channel: string): UpdateState {
  const supported = channel === "stable" || updateChannels.includes(channel);
  const canCheck = packaged && supported;
  return {
    currentVersion, channel, canCheck, checked: false,
    unavailableReasonKey: !packaged ? "请先安装发布版本再检查更新" : !supported ? "自动更新不可用" : null,
    phase: canCheck ? "idle" : "unavailable", id: null, version: null, notes: null, downloaded: 0, total: null, message: null,
  };
}

const state = signal<UpdateState>(initial("stable"));
export const updateState = () => state();
export const updatesAvailable = () => state().canCheck && preferencesSettled();
export const updateBusy = () => state().phase === "checking" || state().phase === "installing";

function failed(): void {
  state.update((s) => ({ ...s, phase: "error", message: "暂时无法获取更新信息，请稍后重试。" }));
}

function exclusive(work: () => Promise<void>): Promise<void> {
  if (operation) return operation;
  operation = work().finally(() => { operation = null; });
  return operation;
}

/** One NSH host operation at a time: its last-found update and download are shared. */
export function checkUpdates(): Promise<void> {
  if (!state().canCheck || state().phase === "ready") return Promise.resolve();
  if (!preferencesSettled()) return Promise.reject(new Error("请等待当前操作完成后检查更新。"));
  return exclusive(async () => {
    const version = generation;
    const channel = configuration.updateChannel;
    state.update((s) => ({ ...s, phase: "checking", message: null }));
    try {
      const result = await checkForUpdate({
        locale: language(),
        channel: updateChannels.includes(channel) ? channel : null,
      });
      if (version !== generation) return;
      found = result;
      state.set({ ...initial(channel), checked: true, phase: result ? "available" : "idle", id: result?.version ?? null, version: result?.version ?? null, notes: result?.notes ?? null });
    } catch (error) {
      if (version === generation) failed();
      throw error;
    }
  });
}

async function prepare(): Promise<void> {
  const version = generation;
  const update = found;
  const current = () => version === generation && found === update;
  if (!update) return;
  if (!update.canInstall) { await openUrl(update.url); return; }
  state.update((s) => ({ ...s, phase: "installing", message: null, downloaded: 0, total: null }));
  try {
    await downloadUpdate(({ received, total }) => {
      if (current()) state.update((s) => ({ ...s, downloaded: received, total: total ?? null }));
    });
    if (!current()) return;
    await installOnQuit();
    if (current()) state.update((s) => ({ ...s, phase: "ready" }));
  } catch (error) { if (current()) failed(); throw error; }
}

/** Manual download stages installation on quit; the ready action can restart now. */
export function downloadAndPrepare(): Promise<void> {
  if (state().phase !== "available") return Promise.resolve();
  if (!preferencesSettled()) return Promise.reject(new Error("请等待当前操作完成后检查更新。"));
  return exclusive(prepare);
}

export function restartAfterUpdate(): Promise<void> {
  if (state().phase !== "ready") return Promise.resolve();
  return exclusive(async () => {
    try { await installAndRestart(); }
    catch (error) {
      state.update((s) => ({ ...s, phase: "ready", message: "暂时无法获取更新信息，请稍后重试。" }));
      throw error;
    }
  });
}

/** Install in the root, after preferences start loading; no work in a dev host. */
export function startUpdater(preferences: () => UpdatePreferences | null): void {
  readPreferences = preferences;
  effect(() => {
    const next = preferences();
    if (!next) return;
    untracked(() => {
      if (next.updateChannel !== configuration.updateChannel) {
        generation++;
        found = null;
        state.set(initial(next.updateChannel));
      }
      configuration = { updateChannel: next.updateChannel, automaticUpdates: next.automaticUpdates };
    });
  });
  if (!packaged) return;
  let stopped = false;
  let timer: ReturnType<typeof setTimeout>;
  const poll = async () => {
    try {
      if (!operation && state().phase !== "ready" && state().canCheck) {
        await checkUpdates();
        if (!stopped && configuration.automaticUpdates && found?.canInstall && state().phase === "available") await downloadAndPrepare();
      }
    } catch { /* The same state exposes a background failure in Settings. */ }
    if (!stopped) timer = setTimeout(() => void poll(), 12 * 60 * 60 * 1000);
  };
  timer = setTimeout(() => void poll(), 10_000);
  cleanup(() => { stopped = true; clearTimeout(timer); });
}
