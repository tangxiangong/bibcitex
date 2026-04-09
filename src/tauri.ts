import { invoke } from "@tauri-apps/api/core";
import type { BibliographyInfo, Reference, Setting } from "./types.ts";

// Settings commands
export async function loadSettings(): Promise<Setting> {
  return await invoke<Setting>("load_settings");
}

export async function saveSettings(settings: Setting): Promise<void> {
  return await invoke("save_settings", { settings });
}

export async function addBibliography(
  name: string,
  path: string,
  description?: string,
): Promise<BibliographyInfo | null> {
  return await invoke<BibliographyInfo | null>("add_bibliography", {
    name,
    path,
    description,
  });
}

export async function removeBibliography(name: string): Promise<void> {
  return await invoke("remove_bibliography", { name });
}

// Bibliography commands
export async function loadBibliography(path: string): Promise<Reference[]> {
  return await invoke<Reference[]>("load_bibliography", { path });
}

export async function parseBibFile(path: string): Promise<Reference[]> {
  return await invoke<Reference[]>("parse_bib_file", { path });
}

// Search commands
export async function searchReferences(
  references: Reference[],
  query: string,
): Promise<Reference[]> {
  return await invoke<Reference[]>("search_references", { references, query });
}

export async function searchByField(
  references: Reference[],
  query: string,
  field: string,
): Promise<Reference[]> {
  return await invoke<Reference[]>("search_by_field", {
    references,
    query,
    field,
  });
}

// Clipboard commands
export async function copyToClipboard(text: string): Promise<void> {
  return await invoke("copy_to_clipboard", { text });
}

export async function pasteToApp(text: string): Promise<void> {
  return await invoke("paste_to_app", { text });
}

// File dialog
export async function selectBibFile(): Promise<string | null> {
  return await invoke<string | null>("select_bib_file");
}

// Utility commands
export async function openUrl(url: string): Promise<void> {
  return await invoke("open_url", { url });
}

export async function openFile(path: string): Promise<void> {
  return await invoke("open_file", { path });
}

// Check for updates
export async function checkUpdate(): Promise<
  { available: boolean; version?: string; notes?: string }
> {
  return await invoke("check_update");
}

export async function installUpdate(): Promise<void> {
  return await invoke("install_update");
}

// Helper window
export async function resizeHelperWindow(height: number): Promise<void> {
  return await invoke("resize_helper_window", { height });
}

export async function openHelperWindow(): Promise<void> {
  return await invoke("open_helper_window");
}

export async function hideHelperWindow(): Promise<void> {
  return await invoke("hide_helper_window");
}

// Helper bib persistence (stored in Rust memory)
export async function getHelperBib(): Promise<[string, string] | null> {
  return await invoke<[string, string] | null>("get_helper_bib");
}

export async function setHelperBib(name: string, path: string): Promise<void> {
  return await invoke("set_helper_bib", { name, path });
}
