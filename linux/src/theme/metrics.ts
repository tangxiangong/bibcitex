/**
 * Surface geometry measured from the macOS client, which is the reference
 * layout for every platform. These are physical boundaries of the surfaces
 * (window and pane sizes, row heights the helper sizes its panel from), so
 * they live here as named constants rather than as ad-hoc values in screens.
 */
export const workbench = {
  minWidth: 560,
  minHeight: 520,
  sidebar: { min: 180, ideal: 220, max: 300, popoverWidth: 260 },
  list: { min: 330 },
  inspector: { min: 280, ideal: 350, max: 520, popoverWidth: 350 },
  /** Below this window width the sidebar moves into a toolbar popover. */
  compactSidebarBelow: 900,
  /** Below this window width the inspector moves into a toolbar popover. */
  compactInspectorBelow: 1280,
  popoverMaxHeight: 560,
  logo: 48,
  emptyLogo: 96,
} as const;

export const helper = {
  maxWidth: 720,
  screenMargin: 40,
  /** A 44pt search row centred in 10pt of slack above and below. */
  header: 64,
  referenceRow: 80,
  libraryRow: 56,
  rowGap: 2,
  listPadding: 16,
  maxListHeight: 460,
  emptyRow: 112,
  errorBar: 40,
  bottomBar: 52,
  barButton: 28,
  keyCap: 18,
} as const;

export const tray = {
  width: 680,
  height: 580,
  detailWidth: 282,
  screenGap: 6,
} as const;

export const settings = { width: 480, height: 540 } as const;

export const dialogs = {
  libraryEditor: 520,
  advancedSearch: { width: 560, height: 640 },
  tagManager: { width: 460, height: 440 },
  tagAssignment: { width: 360, height: 340 },
  batchTags: { width: 460, height: 340 },
  invalidTagLinks: 420,
  tagFilter: 480,
} as const;

/** The helper panel's height for its content: the header alone while idle,
 * then the list grows to its rows up to `maxListHeight`. */
export function helperHeight(rows: number, kind: "reference" | "library", extra = 0): number {
  if (rows <= 0) return helper.header + extra;
  const row = kind === "reference" ? helper.referenceRow : helper.libraryRow;
  const list = Math.min(rows * (row + helper.rowGap) + helper.listPadding, helper.maxListHeight);
  return helper.header + 1 + list + extra;
}
