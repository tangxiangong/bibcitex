// Asset path helper for Tauri
// In development mode (Vite dev server), assets are served from /public
// In production mode (Tauri build), assets are bundled and need special handling

/**
 * Get the correct path for an asset based on the environment
 * @param path - The relative path from the public directory (e.g., "/icons/copy.svg")
 * @returns The correct asset URL for the current environment
 */
export function getAssetUrl(path: string): string {
  // In Tauri, we can check if we're running in a Tauri context
  if (typeof window !== "undefined" && window.__TAURI__) {
    // For Tauri production builds, use the asset protocol
    // The public folder gets copied to the build directory, so paths remain the same
    return path;
  }
  // For development or web builds, use the path as-is (Vite handles it)
  return path;
}

// Icon constants - all paths relative to public directory
export const ICONS = {
  COPY: getAssetUrl("/icons/copy.svg"),
  OK: getAssetUrl("/icons/ok.svg"),
  ERROR: getAssetUrl("/icons/error.svg"),
  DETAILS: getAssetUrl("/icons/details.svg"),
  ADD: getAssetUrl("/icons/add.svg"),
  DELETE: getAssetUrl("/icons/delete.svg"),
  CANCEL: getAssetUrl("/icons/cancel.svg"),
} as const;

export const LOGO = getAssetUrl("/logo.png");
export const TRANSPARENT_LOGO = getAssetUrl("/transparent_logo.png");
export const FAVICON = getAssetUrl("/favicon.png");

// Re-export individual icon paths for convenience
export const COPY_ICON = ICONS.COPY;
export const OK_ICON = ICONS.OK;
export const ERROR_ICON = ICONS.ERROR;
export const DETAILS_ICON = ICONS.DETAILS;
export const ADD_ICON = ICONS.ADD;
export const DELETE_ICON = ICONS.DELETE;
export const CANCEL_ICON = ICONS.CANCEL;
