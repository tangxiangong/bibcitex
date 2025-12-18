import { writable, derived } from 'svelte/store';
import type { Setting, Reference, BibliographyInfo } from '../types';

// Global application state
export const settings = writable<Setting>({
  bibliographies: {}
});

export const currentReferences = writable<Reference[] | null>(null);

export const drawerOpen = writable<boolean>(false);

export const drawerReference = writable<Reference | null>(null);

// Current selected bibliography name
export const currentBibName = writable<string | null>(null);

// Derived store for bibliography list
export const bibliographyList = derived(settings, ($settings) => {
  return Object.entries($settings.bibliographies).map(([name, info]) => ({
    name,
    ...info
  }));
});

// Helper functions
export function openDrawer(reference: Reference) {
  drawerReference.set(reference);
  drawerOpen.set(true);
}

export function closeDrawer() {
  drawerOpen.set(false);
  drawerReference.set(null);
}

export function updateSettings(newSettings: Setting) {
  settings.set(newSettings);
}

export function setCurrentReferences(refs: Reference[] | null) {
  currentReferences.set(refs);
}
