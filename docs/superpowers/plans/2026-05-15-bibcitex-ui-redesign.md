# BibCiTeX UI Redesign Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement the approved BibCiTeX UI redesign: Catppuccin daisyUI themes, SVG-only icon system, collapsible three-pane main workbench, semantic reference coloring, and command-palette helper flow.

**Architecture:** Keep the existing Tauri/SolidJS IPC boundary and backend commands. Refactor the frontend around shared presentational primitives: theme setup, icon buttons, semantic reference styling, three-pane workbench panes, and helper result rows. Prefer small focused components over duplicating per-entry-type card layouts.

**Tech Stack:** Tauri v2, Rust backend IPC, SolidJS, @solidjs/router, Tailwind CSS v4, daisyUI v5, Bun, `@catppuccin/daisyui`, `lucide-solid`.

---

## File Structure

Create or modify these files:

- Modify: `package.json`
- Modify: `bun.lock`
- Create: `src/catppuccinTheme.latte.ts`
- Create: `src/catppuccinTheme.mocha.ts`
- Modify: `src/app.css`
- Modify: `src/App.tsx`
- Modify: `src/context/AppContext.tsx`
- Create: `src/components/ui/IconButton.tsx`
- Create: `src/components/ui/SvgIcon.tsx`
- Create: `src/components/reference/semantic.ts`
- Create: `src/components/workbench/WorkbenchPage.tsx`
- Create: `src/components/workbench/LibraryPane.tsx`
- Create: `src/components/workbench/ReferenceListPane.tsx`
- Create: `src/components/workbench/ReferenceRow.tsx`
- Create: `src/components/workbench/ReferenceDetailPane.tsx`
- Create: `src/components/workbench/ReferenceMetadata.tsx`
- Modify: `src/pages/HomePage.tsx`
- Modify: `src/pages/DetailPage.tsx`
- Modify: `src/pages/HelperPage.tsx`
- Modify: `src/components/Nav.tsx`
- Modify: `src/components/AddBibliography.tsx`
- Modify: `src/components/updater/UpdateBanner.tsx`
- Leave in place during this plan: existing `src/components/reference/*`, `src/components/drawer/*`, and `src/components/helper/*` unless a task explicitly replaces an import. This plan does not delete those modules.

## Commit Rule

For every task commit in this plan:

1. Inspect `git status`, `git diff`, and `git diff --staged` before committing.
2. Create the temporary commit message file named in the task command.
3. Use the subject shown in the task command comment.
4. Include the required `Summary:`, `Rationale:`, and `Tests:` sections with bullets that describe only the staged diff.
5. End the message with `Co-authored-by: Codex <noreply@openai.com>`.
6. Run `git commit -F <message-file>`.

Do not use one-line `git commit -m` commands for this repository.

## Task 1: Install Theme and Icon Dependencies

**Files:**
- Modify: `package.json`
- Modify: `bun.lock`

- [ ] **Step 1: Add dependencies**

Run:

```bash
bun add lucide-solid
bun add -d @catppuccin/daisyui
```

Expected:

```text
package.json updated with lucide-solid under dependencies
package.json updated with @catppuccin/daisyui under devDependencies
bun.lock updated
```

- [ ] **Step 2: Verify dependency metadata**

Run:

```bash
rg -n '"lucide-solid"|"@catppuccin/daisyui"' package.json
```

Expected:

```text
package.json contains both dependencies
```

- [ ] **Step 3: Commit**

```bash
git add package.json bun.lock
# Subject: build(ui): add theme and icon dependencies
git commit -F /tmp/bibcitex-ui-task-1-commit.txt
```

## Task 2: Configure Catppuccin daisyUI Themes

**Files:**
- Create: `src/catppuccinTheme.latte.ts`
- Create: `src/catppuccinTheme.mocha.ts`
- Modify: `src/app.css`

- [ ] **Step 1: Create latte theme plugin**

Create `src/catppuccinTheme.latte.ts`:

```ts
import { createCatppuccinPlugin } from "@catppuccin/daisyui";

export default createCatppuccinPlugin("latte", {}, {
  default: true,
});
```

- [ ] **Step 2: Create mocha theme plugin**

Create `src/catppuccinTheme.mocha.ts`:

```ts
import { createCatppuccinPlugin } from "@catppuccin/daisyui";

export default createCatppuccinPlugin("mocha");
```

- [ ] **Step 3: Replace the daisyUI theme block**

In `src/app.css`, replace the current daisyUI plugin block:

```css
@plugin "daisyui" {
  themes: winter --default, dracula --prefersdark;
}
```

with:

```css
@plugin "daisyui" {
  themes: false;
}

@plugin "./catppuccinTheme.latte.ts";
@plugin "./catppuccinTheme.mocha.ts";
```

- [ ] **Step 4: Replace the dark variant rule**

In `src/app.css`, replace the current dark variant block with:

```css
@variant dark (&:where([data-theme="mocha"]) *);
```

- [ ] **Step 5: Run type and build checks**

Run:

```bash
bun run check
bun run build
```

Expected:

```text
bun run check exits 0
bun run build exits 0
```

- [ ] **Step 6: Commit**

```bash
git add src/catppuccinTheme.latte.ts src/catppuccinTheme.mocha.ts src/app.css
# Subject: feat(theme): use Catppuccin daisyUI themes
git commit -F /tmp/bibcitex-ui-task-2-commit.txt
```

## Task 3: Add App-Wide Theme Control

**Files:**
- Modify: `src/context/AppContext.tsx`
- Modify: `src/App.tsx`
- Modify: `src/main.tsx` only if theme initialization needs to happen before render.

- [ ] **Step 1: Extend context types and state**

In `src/context/AppContext.tsx`, add this type near the existing context types:

```ts
type AppTheme = "latte" | "mocha";
```

Extend `AppContextType` with:

```ts
theme: Accessor<AppTheme>;
setTheme: Setter<AppTheme>;
toggleTheme: () => void;
```

Inside `AppProvider`, add:

```ts
const getInitialTheme = (): AppTheme => {
  if (typeof window === "undefined") return "latte";
  return window.matchMedia("(prefers-color-scheme: dark)").matches
    ? "mocha"
    : "latte";
};

const [theme, setTheme] = createSignal<AppTheme>(getInitialTheme());

createEffect(() => {
  document.documentElement.setAttribute("data-theme", theme());
});

const toggleTheme = () => {
  setTheme((current) => current === "latte" ? "mocha" : "latte");
};
```

Add `theme`, `setTheme`, and `toggleTheme` to the provider value.

- [ ] **Step 2: Keep helper and main app globally consistent**

Do not set a separate hardcoded `data-theme` in `src/pages/HelperPage.tsx`. The helper window should use the same context-driven `data-theme` behavior when rendered under `AppProvider`.

If helper transparency code remains, keep only background transparency setup:

```ts
onMount(() => {
  document.documentElement.style.background = "transparent";
  document.body.style.background = "transparent";
});
```

- [ ] **Step 3: Run check**

```bash
bun run check
```

Expected:

```text
bun run check exits 0
```

- [ ] **Step 4: Commit**

```bash
git add src/context/AppContext.tsx src/App.tsx src/main.tsx src/pages/HelperPage.tsx
# Subject: feat(theme): apply global latte and mocha modes
git commit -F /tmp/bibcitex-ui-task-3-commit.txt
```

## Task 4: Create SVG-Only Icon Primitives

**Files:**
- Create: `src/components/ui/SvgIcon.tsx`
- Create: `src/components/ui/IconButton.tsx`
- Used by Task 9, Task 10, and Task 11 to replace inline emoji and ad-hoc icon buttons.

- [ ] **Step 1: Create icon registry**

Create `src/components/ui/SvgIcon.tsx`:

```tsx
import type { JSX } from "solid-js";
import { splitProps } from "solid-js";
import {
  AlertCircle,
  BookOpen,
  Calendar,
  ChevronLeft,
  ChevronRight,
  Clipboard,
  Copy,
  ExternalLink,
  FileText,
  FolderOpen,
  Info,
  Library,
  Link,
  PanelLeftClose,
  PanelLeftOpen,
  PanelRightClose,
  PanelRightOpen,
  RefreshCw,
  Search,
  Settings,
  Tag,
  User,
  X,
} from "lucide-solid";

const icons = {
  alert: AlertCircle,
  book: BookOpen,
  calendar: Calendar,
  chevronLeft: ChevronLeft,
  chevronRight: ChevronRight,
  clipboard: Clipboard,
  copy: Copy,
  externalLink: ExternalLink,
  fileText: FileText,
  folderOpen: FolderOpen,
  info: Info,
  library: Library,
  link: Link,
  panelLeftClose: PanelLeftClose,
  panelLeftOpen: PanelLeftOpen,
  panelRightClose: PanelRightClose,
  panelRightOpen: PanelRightOpen,
  refresh: RefreshCw,
  search: Search,
  settings: Settings,
  tag: Tag,
  user: User,
  x: X,
};

export type SvgIconName = keyof typeof icons;

export interface SvgIconProps extends JSX.SvgSVGAttributes<SVGSVGElement> {
  name: SvgIconName;
  size?: number;
  "aria-hidden"?: boolean | "true" | "false";
}

export function SvgIcon(props: SvgIconProps) {
  const [local, others] = splitProps(props, ["name", "size", "aria-hidden"]);
  const Icon = icons[local.name];
  return (
    <Icon
      size={local.size ?? 18}
      strokeWidth={1.8}
      aria-hidden={local["aria-hidden"] ?? true}
      {...others}
    />
  );
}
```

- [ ] **Step 2: Create accessible icon button**

Create `src/components/ui/IconButton.tsx`:

```tsx
import type { JSX } from "solid-js";
import { splitProps } from "solid-js";
import { SvgIcon, type SvgIconName } from "./SvgIcon";

interface IconButtonProps extends JSX.ButtonHTMLAttributes<HTMLButtonElement> {
  icon: SvgIconName;
  label: string;
  tooltip?: string;
  size?: "sm" | "md";
  variant?: "ghost" | "primary" | "error";
}

export function IconButton(props: IconButtonProps) {
  const [local, buttonProps] = splitProps(props, ["icon", "label", "tooltip", "size", "variant", "class", "type"]);
  const sizeClass = () => local.size === "sm" ? "btn-sm" : "";
  const variantClass = () => {
    switch (local.variant) {
      case "primary":
        return "btn-primary";
      case "error":
        return "btn-error btn-soft";
      default:
        return "btn-ghost";
    }
  };

  return (
    <div class={local.tooltip ? "tooltip" : undefined} data-tip={local.tooltip}>
      <button
        {...buttonProps}
        type={local.type ?? "button"}
        aria-label={local.label}
        title={local.tooltip ?? local.label}
        class={`btn btn-square min-h-11 h-11 w-11 ${sizeClass()} ${variantClass()} ${local.class ?? ""}`}
      >
        <SvgIcon name={local.icon} />
      </button>
    </div>
  );
}
```

- [ ] **Step 3: Run check**

```bash
bun run check
```

Expected:

```text
bun run check exits 0
```

- [ ] **Step 4: Commit**

```bash
git add src/components/ui/SvgIcon.tsx src/components/ui/IconButton.tsx
# Subject: feat(ui): add SVG icon primitives
git commit -F /tmp/bibcitex-ui-task-4-commit.txt
```

## Task 5: Add Reference Semantic Styling Maps

**Files:**
- Create: `src/components/reference/semantic.ts`
- Consumed by Task 7 and Task 10 in `ReferenceRow.tsx`, `ReferenceMetadata.tsx`, and `HelperPage.tsx`.

- [ ] **Step 1: Create semantic map**

Create `src/components/reference/semantic.ts`:

```ts
import type { EntryType, Reference } from "@/types";
import type { SvgIconName } from "@/components/ui/SvgIcon";

export type ReferenceTypeKey =
  | "Article"
  | "Book"
  | "Booklet"
  | "InBook"
  | "InCollection"
  | "InProceedings"
  | "Thesis"
  | "TechReport"
  | "Misc"
  | "Unknown";

export interface TypeStyle {
  label: string;
  badgeClass: string;
  borderClass: string;
  icon: SvgIconName;
}

export interface MetadataStyle {
  label: string;
  chipClass: string;
  icon: SvgIconName;
}

export function getReferenceTypeKey(type: EntryType): ReferenceTypeKey {
  if (typeof type !== "string") return "Unknown";
  if (type === "MastersThesis" || type === "PhdThesis" || type === "Thesis") {
    return "Thesis";
  }
  if (
    type === "Article" ||
    type === "Book" ||
    type === "Booklet" ||
    type === "InBook" ||
    type === "InCollection" ||
    type === "InProceedings" ||
    type === "TechReport" ||
    type === "Misc"
  ) {
    return type;
  }
  return "Unknown";
}

export const TYPE_STYLES: Record<ReferenceTypeKey, TypeStyle> = {
  Article: {
    label: "Article",
    badgeClass: "badge-info",
    borderClass: "border-l-info",
    icon: "fileText",
  },
  Book: {
    label: "Book",
    badgeClass: "badge-success",
    borderClass: "border-l-success",
    icon: "book",
  },
  Booklet: {
    label: "Booklet",
    badgeClass: "badge-accent",
    borderClass: "border-l-accent",
    icon: "book",
  },
  InBook: {
    label: "InBook",
    badgeClass: "badge-secondary",
    borderClass: "border-l-secondary",
    icon: "book",
  },
  InCollection: {
    label: "InCollection",
    badgeClass: "badge-secondary",
    borderClass: "border-l-secondary",
    icon: "library",
  },
  InProceedings: {
    label: "InProceedings",
    badgeClass: "badge-primary",
    borderClass: "border-l-primary",
    icon: "fileText",
  },
  Thesis: {
    label: "Thesis",
    badgeClass: "badge-warning",
    borderClass: "border-l-warning",
    icon: "fileText",
  },
  TechReport: {
    label: "TechReport",
    badgeClass: "badge-neutral",
    borderClass: "border-l-neutral",
    icon: "fileText",
  },
  Misc: {
    label: "Misc",
    badgeClass: "badge-ghost",
    borderClass: "border-l-base-300",
    icon: "tag",
  },
  Unknown: {
    label: "Unknown",
    badgeClass: "badge-error",
    borderClass: "border-l-error",
    icon: "alert",
  },
};

export const METADATA_STYLES = {
  title: {
    label: "Title",
    chipClass: "bg-primary/10 text-base-content border border-primary/30",
    icon: "fileText",
  },
  author: {
    label: "Author",
    chipClass: "bg-secondary/10 text-base-content border border-secondary/30",
    icon: "user",
  },
  year: {
    label: "Year",
    chipClass: "bg-accent/10 text-base-content border border-accent/30 font-mono",
    icon: "calendar",
  },
  venue: {
    label: "Venue",
    chipClass: "bg-info/10 text-base-content border border-info/30",
    icon: "library",
  },
  link: {
    label: "Link",
    chipClass: "bg-success/10 text-base-content border border-success/30",
    icon: "link",
  },
  citeKey: {
    label: "Key",
    chipClass: "bg-warning/10 text-base-content border border-warning/30 font-mono",
    icon: "tag",
  },
  note: {
    label: "Note",
    chipClass: "bg-neutral/10 text-base-content border border-neutral/30",
    icon: "info",
  },
} satisfies Record<string, MetadataStyle>;

// The chip classes use distinct daisyUI semantic color variables for the
// background and border, while keeping text on `text-base-content` for contrast.
// Latte and mocha resolve these same classes to different Catppuccin palettes.

export function getReferenceVenue(reference: Reference): string | undefined {
  return reference.full_journal ||
    reference.journal ||
    reference.book_title?.map((chunk) =>
      "Normal" in chunk ? chunk.Normal : "Verbatim" in chunk ? chunk.Verbatim : chunk.Math
    ).join("") ||
    reference.publisher?.join(", ") ||
    reference.school ||
    reference.institution ||
    reference.organization?.join(", ");
}
```

- [ ] **Step 2: Run check**

```bash
bun run check
```

Expected:

```text
bun run check exits 0
```

- [ ] **Step 3: Commit**

```bash
git add src/components/reference/semantic.ts
# Subject: feat(reference): add semantic styling maps
git commit -F /tmp/bibcitex-ui-task-5-commit.txt
```

## Task 6: Extend App Context for Workbench Selection and Panes

**Files:**
- Modify: `src/context/AppContext.tsx`

- [ ] **Step 1: Add workbench state to context type**

Extend `AppContextType` with:

```ts
selectedReference: Accessor<Reference | null>;
setSelectedReference: Setter<Reference | null>;
leftPaneOpen: Accessor<boolean>;
setLeftPaneOpen: Setter<boolean>;
rightPaneOpen: Accessor<boolean>;
setRightPaneOpen: Setter<boolean>;
selectBibliography: (name: string, path: string) => Promise<void>;
```

- [ ] **Step 2: Add state and selection helper inside `AppProvider`**

Add inside `AppProvider`:

```ts
const [selectedReference, setSelectedReference] = createSignal<Reference | null>(null);
const [leftPaneOpen, setLeftPaneOpen] = createSignal(true);
const [rightPaneOpen, setRightPaneOpen] = createSignal(true);

const selectBibliography = async (name: string, path: string) => {
  const refs = await loadBibliography(path);
  setCurrentReferences(refs);
  setCurrentBibName(name);
  setSelectedReference(refs[0] ?? null);
};
```

Add import:

```ts
import { loadBibliography } from "@/tauri";
```

Add the new values to the provider value object.

- [ ] **Step 3: Preserve drawer compatibility**

Keep `openDrawer` and `closeDrawer` in the context for mobile fallback and for any old components still imported during the transition.

- [ ] **Step 4: Run check**

```bash
bun run check
```

Expected:

```text
bun run check exits 0
```

- [ ] **Step 5: Commit**

```bash
git add src/context/AppContext.tsx
# Subject: feat(workbench): add selection and pane state
git commit -F /tmp/bibcitex-ui-task-6-commit.txt
```

## Task 7: Build Three-Pane Workbench Components

**Files:**
- Create: `src/components/workbench/WorkbenchPage.tsx`
- Create: `src/components/workbench/LibraryPane.tsx`
- Create: `src/components/workbench/ReferenceListPane.tsx`
- Create: `src/components/workbench/ReferenceRow.tsx`
- Create: `src/components/workbench/ReferenceDetailPane.tsx`
- Create: `src/components/workbench/ReferenceMetadata.tsx`

- [ ] **Step 1: Create `ReferenceRow`**

Create `src/components/workbench/ReferenceRow.tsx`:

```tsx
import { Show } from "solid-js";
import type { Reference } from "@/types";
import { ChunksComp } from "@/components/ChunksComp";
import { SvgIcon } from "@/components/ui/SvgIcon";
import {
  getReferenceTypeKey,
  getReferenceVenue,
  METADATA_STYLES,
  TYPE_STYLES,
} from "@/components/reference/semantic";

interface ReferenceRowProps {
  reference: Reference;
  selected: boolean;
  onSelect: () => void;
}

export function ReferenceRow(props: ReferenceRowProps) {
  const typeKey = () => getReferenceTypeKey(props.reference.type_);
  const typeStyle = () => TYPE_STYLES[typeKey()];
  const venue = () => getReferenceVenue(props.reference);
  const authors = () => props.reference.author?.slice(0, 3).join(", ");

  return (
    <button
      type="button"
      class={`w-full border-l-4 ${typeStyle().borderClass} ${
        props.selected ? "bg-primary/10 border-y border-r border-primary/30" : "bg-base-100 hover:bg-base-200 border-y border-r border-base-300"
      } rounded-box p-3 text-left transition-colors focus:outline-none focus:ring-2 focus:ring-primary`}
      onClick={props.onSelect}
    >
      <div class="flex items-start gap-3">
        <span class={`badge ${typeStyle().badgeClass} badge-sm gap-1 shrink-0`}>
          <SvgIcon name={typeStyle().icon} size={12} />
          {typeStyle().label}
        </span>
        <div class="min-w-0 flex-1">
          <div class="font-medium leading-snug">
            <Show when={props.reference.title} fallback="No title available">
              <ChunksComp chunks={props.reference.title!} citeKey={props.reference.cite_key} />
            </Show>
          </div>
          <div class="mt-2 flex flex-wrap gap-1.5 text-xs text-base-content/70">
            <span class={`inline-flex items-center gap-1 rounded-field px-2 py-1 ${METADATA_STYLES.citeKey.chipClass}`}>
              <SvgIcon name="tag" size={12} />
              {props.reference.cite_key}
            </span>
            <Show when={authors()}>
              <span class={`inline-flex items-center gap-1 rounded-field px-2 py-1 ${METADATA_STYLES.author.chipClass}`}>
                <SvgIcon name="user" size={12} />
                {authors()}
              </span>
            </Show>
            <Show when={props.reference.year}>
              <span class={`inline-flex items-center gap-1 rounded-field px-2 py-1 ${METADATA_STYLES.year.chipClass}`}>
                <SvgIcon name="calendar" size={12} />
                {props.reference.year}
              </span>
            </Show>
            <Show when={venue()}>
              <span class={`inline-flex items-center gap-1 rounded-field px-2 py-1 ${METADATA_STYLES.venue.chipClass}`}>
                <SvgIcon name="library" size={12} />
                {venue()}
              </span>
            </Show>
          </div>
        </div>
      </div>
    </button>
  );
}
```

- [ ] **Step 2: Create `ReferenceMetadata`**

Create `src/components/workbench/ReferenceMetadata.tsx`:

```tsx
import { For, Show } from "solid-js";
import type { Reference } from "@/types";
import { ChunksComp } from "@/components/ChunksComp";
import { SvgIcon } from "@/components/ui/SvgIcon";
import { METADATA_STYLES } from "@/components/reference/semantic";

interface ReferenceMetadataProps {
  reference: Reference;
}

export function ReferenceMetadata(props: ReferenceMetadataProps) {
  const bibtex = () => props.reference.source.split("\n");

  return (
    <div class="space-y-3">
      <section class="rounded-box border border-base-300 bg-base-100 p-3">
        <h3 class="mb-2 flex items-center gap-2 text-sm font-semibold">
          <SvgIcon name="info" size={16} />
          Core Metadata
        </h3>
        <div class="flex flex-wrap gap-2 text-sm">
          <span class={`inline-flex items-center gap-1 rounded-field px-2 py-1 ${METADATA_STYLES.citeKey.chipClass}`}>
            <SvgIcon name="tag" size={13} />
            {props.reference.cite_key}
          </span>
          <Show when={props.reference.year}>
            <span class={`inline-flex items-center gap-1 rounded-field px-2 py-1 ${METADATA_STYLES.year.chipClass}`}>
              <SvgIcon name="calendar" size={13} />
              {props.reference.year}
            </span>
          </Show>
        </div>
      </section>

      <Show when={props.reference.author?.length}>
        <section class="rounded-box border border-base-300 bg-base-100 p-3">
          <h3 class="mb-2 flex items-center gap-2 text-sm font-semibold">
            <SvgIcon name="user" size={16} />
            Authors
          </h3>
          <div class="flex flex-wrap gap-2">
            <For each={props.reference.author!}>
              {(author) => (
                <span class={`rounded-field px-2 py-1 text-sm ${METADATA_STYLES.author.chipClass}`}>
                  {author}
                </span>
              )}
            </For>
          </div>
        </section>
      </Show>

      <Show when={props.reference.abstract_}>
        <section class="rounded-box border border-base-300 bg-base-100 p-3">
          <h3 class="mb-2 flex items-center gap-2 text-sm font-semibold">
            <SvgIcon name="fileText" size={16} />
            Abstract
          </h3>
          <div class="text-sm leading-relaxed">
            <ChunksComp chunks={props.reference.abstract_!} citeKey={`${props.reference.cite_key}-abstract`} />
          </div>
        </section>
      </Show>

      <section class="rounded-box border border-base-300 bg-base-100 p-3">
        <h3 class="mb-2 flex items-center gap-2 text-sm font-semibold">
          <SvgIcon name="fileText" size={16} />
          BibTeX
        </h3>
        <pre class="max-h-72 overflow-auto rounded-field bg-base-200 p-3 text-xs"><code>
          <For each={bibtex()}>{(line) => `${line}\n`}</For>
        </code></pre>
      </section>
    </div>
  );
}
```

- [ ] **Step 3: Create `ReferenceDetailPane`**

Create `src/components/workbench/ReferenceDetailPane.tsx`:

```tsx
import { Show } from "solid-js";
import { copyToClipboard, openFile, openUrl } from "@/tauri";
import { useApp } from "@/context/AppContext";
import { ChunksComp } from "@/components/ChunksComp";
import { IconButton } from "@/components/ui/IconButton";
import { ReferenceMetadata } from "./ReferenceMetadata";

export function ReferenceDetailPane() {
  const { selectedReference, rightPaneOpen, setRightPaneOpen } = useApp();

  const copyKey = async () => {
    const ref = selectedReference();
    if (ref) await copyToClipboard(ref.cite_key);
  };

  return (
    <aside class={`${rightPaneOpen() ? "flex" : "hidden"} min-w-[18rem] max-w-[26rem] w-[24rem] flex-col border-l border-base-300 bg-base-200`}>
      <header class="flex h-14 items-center justify-between border-b border-base-300 px-3">
        <h2 class="text-sm font-semibold">Reference Detail</h2>
        <IconButton icon="panelRightClose" label="Hide detail pane" tooltip="Hide detail" size="sm" onClick={() => setRightPaneOpen(false)} />
      </header>
      <div class="min-h-0 flex-1 overflow-y-auto p-3">
        <Show
          when={selectedReference()}
          fallback={<p class="text-sm text-base-content/60">Select a reference to inspect its metadata.</p>}
        >
          {(ref) => (
            <div class="space-y-3">
              <h1 class="text-lg font-semibold leading-snug">
                <Show when={ref().title} fallback="No title available">
                  <ChunksComp chunks={ref().title!} citeKey={`${ref().cite_key}-detail-title`} />
                </Show>
              </h1>
              <div class="flex flex-wrap gap-2">
                <button type="button" class="btn btn-primary btn-sm" onClick={copyKey}>Copy key</button>
                <Show when={ref().doi}>
                  <button type="button" class="btn btn-ghost btn-sm" onClick={() => openUrl(`https://doi.org/${ref().doi}`)}>DOI</button>
                </Show>
                <Show when={ref().url}>
                  <button type="button" class="btn btn-ghost btn-sm" onClick={() => openUrl(ref().url!)}>URL</button>
                </Show>
                <Show when={ref().file}>
                  <button type="button" class="btn btn-ghost btn-sm" onClick={() => openFile(ref().file!)}>PDF</button>
                </Show>
              </div>
              <ReferenceMetadata reference={ref()} />
            </div>
          )}
        </Show>
      </div>
    </aside>
  );
}
```

- [ ] **Step 4: Create `LibraryPane`**

Create `src/components/workbench/LibraryPane.tsx`:

```tsx
import { For, Show } from "solid-js";
import { openFile, removeBibliography } from "@/tauri";
import { useApp } from "@/context/AppContext";
import { IconButton } from "@/components/ui/IconButton";

interface LibraryPaneProps {
  onAdd: () => void;
  onError: (message: string) => void;
}

export function LibraryPane(props: LibraryPaneProps) {
  const {
    bibliographyList,
    currentBibName,
    leftPaneOpen,
    setLeftPaneOpen,
    selectBibliography,
  } = useApp();

  const handleRemove = async (name: string) => {
    try {
      await removeBibliography(name);
      window.location.reload();
    } catch (error) {
      props.onError(`删除失败: ${error}`);
    }
  };

  return (
    <aside class={`${leftPaneOpen() ? "flex" : "hidden"} min-w-[16rem] max-w-[22rem] w-[18rem] flex-col border-r border-base-300 bg-base-200`}>
      <header class="flex h-14 items-center justify-between border-b border-base-300 px-3">
        <h2 class="text-sm font-semibold">Bibliographies</h2>
        <IconButton icon="panelLeftClose" label="Hide bibliography pane" tooltip="Hide libraries" size="sm" onClick={() => setLeftPaneOpen(false)} />
      </header>
      <div class="min-h-0 flex-1 overflow-y-auto p-2">
        <Show
          when={bibliographyList().length > 0}
          fallback={<p class="p-3 text-sm text-base-content/60">No bibliographies yet.</p>}
        >
          <For each={bibliographyList()}>
            {(bib) => (
              <button
                type="button"
                class={`mb-2 w-full rounded-box border p-3 text-left transition-colors focus:outline-none focus:ring-2 focus:ring-primary ${
                  currentBibName() === bib.name ? "border-primary bg-primary/10" : "border-base-300 bg-base-100 hover:bg-base-300"
                }`}
                onClick={() => selectBibliography(bib.name, bib.path).catch((error) => props.onError(`加载失败: ${error}`))}
              >
                <div class="font-medium">{bib.name}</div>
                <div class="truncate text-xs text-base-content/60" title={bib.path}>{bib.path}</div>
                <div class="mt-2 flex gap-1">
                  <button type="button" class="btn btn-ghost btn-xs" onClick={(event) => { event.stopPropagation(); openFile(bib.path); }}>Open</button>
                  <button type="button" class="btn btn-error btn-soft btn-xs" onClick={(event) => { event.stopPropagation(); handleRemove(bib.name); }}>Remove</button>
                </div>
              </button>
            )}
          </For>
        </Show>
      </div>
      <div class="border-t border-base-300 p-2">
        <button type="button" class="btn btn-primary w-full" onClick={props.onAdd}>Add .bib</button>
      </div>
    </aside>
  );
}
```

- [ ] **Step 5: Create `ReferenceListPane`**

Create `src/components/workbench/ReferenceListPane.tsx`:

```tsx
import { createMemo, createSignal, For, Show } from "solid-js";
import type { FilterField, FilterType } from "@/types";
import { searchByField, searchReferences } from "@/tauri";
import { useApp } from "@/context/AppContext";
import { IconButton } from "@/components/ui/IconButton";
import { ReferenceRow } from "./ReferenceRow";

export function ReferenceListPane() {
  const {
    currentReferences,
    currentBibName,
    selectedReference,
    setSelectedReference,
    leftPaneOpen,
    setLeftPaneOpen,
    rightPaneOpen,
    setRightPaneOpen,
  } = useApp();
  const [query, setQuery] = createSignal("");
  const [searchResult, setSearchResult] = createSignal(currentReferences() ?? []);
  const [filterField, setFilterField] = createSignal<FilterField>("All");
  const [filterType, setFilterType] = createSignal<FilterType>("All");

  const filterTypes: FilterType[] = ["All", "Article", "Book", "Thesis", "TechReport", "Misc", "Booklet", "InBook", "InCollection", "InProceedings"];
  const filterFields: FilterField[] = ["All", "Author", "Title", "Journal", "Year"];

  const filteredByType = createMemo(() => {
    const refs = currentReferences() ?? [];
    if (filterType() === "All") return refs;
    return refs.filter((ref) => {
      const type = typeof ref.type_ === "string" ? ref.type_ : "Unknown";
      if (filterType() === "Thesis") return type === "Thesis" || type === "MastersThesis" || type === "PhdThesis";
      return type === filterType();
    });
  });

  const displayRefs = createMemo(() => query().trim() ? searchResult() : filteredByType());

  const handleSearch = async (value: string) => {
    setQuery(value);
    if (!value.trim()) {
      setSearchResult(filteredByType());
      return;
    }
    const result = filterField() === "All"
      ? await searchReferences(filteredByType(), value)
      : await searchByField(filteredByType(), value, filterField());
    setSearchResult(result);
  };

  return (
    <section class="flex min-w-0 flex-1 flex-col bg-base-100">
      <header class="flex h-14 items-center gap-2 border-b border-base-300 px-3">
        <Show when={!leftPaneOpen()}>
          <IconButton icon="panelLeftOpen" label="Show bibliography pane" tooltip="Show libraries" size="sm" onClick={() => setLeftPaneOpen(true)} />
        </Show>
        <div class="min-w-0">
          <h1 class="truncate text-sm font-semibold">{currentBibName() ?? "Select a bibliography"}</h1>
          <p class="text-xs text-base-content/60">{displayRefs().length} / {currentReferences()?.length ?? 0} references</p>
        </div>
        <div class="ml-auto flex gap-2">
          <Show when={!rightPaneOpen()}>
            <IconButton icon="panelRightOpen" label="Show detail pane" tooltip="Show detail" size="sm" onClick={() => setRightPaneOpen(true)} />
          </Show>
        </div>
      </header>

      <div class="flex gap-2 border-b border-base-300 p-3">
        <select class="select select-bordered select-sm w-36" value={filterType()} onChange={(event) => setFilterType(event.currentTarget.value as FilterType)}>
          <For each={filterTypes}>{(type) => <option value={type}>{type === "All" ? "All Types" : type}</option>}</For>
        </select>
        <select class="select select-bordered select-sm w-36" value={filterField()} onChange={(event) => setFilterField(event.currentTarget.value as FilterField)}>
          <For each={filterFields}>{(field) => <option value={field}>{field === "All" ? "All Fields" : field}</option>}</For>
        </select>
        <input class="input input-bordered input-sm min-w-0 flex-1" type="search" placeholder="Search references..." value={query()} onInput={(event) => handleSearch(event.currentTarget.value)} />
      </div>

      <div class="min-h-0 flex-1 overflow-y-auto p-3">
        <Show
          when={currentReferences()?.length}
          fallback={<p class="text-sm text-base-content/60">Select a bibliography to load references.</p>}
        >
          <Show
            when={displayRefs().length > 0}
            fallback={<p class="text-sm text-base-content/60">No results. Try a different query or filter.</p>}
          >
            <div class="space-y-2">
              <For each={displayRefs()}>
                {(reference) => (
                  <ReferenceRow
                    reference={reference}
                    selected={selectedReference()?.cite_key === reference.cite_key}
                    onSelect={() => setSelectedReference(reference)}
                  />
                )}
              </For>
            </div>
          </Show>
        </Show>
      </div>
    </section>
  );
}
```

- [ ] **Step 6: Create `WorkbenchPage`**

Create `src/components/workbench/WorkbenchPage.tsx`:

```tsx
import { createSignal, onMount, Show } from "solid-js";
import { loadSettings } from "@/tauri";
import { useApp } from "@/context/AppContext";
import { AddBibliography } from "@/components/AddBibliography";
import { LibraryPane } from "./LibraryPane";
import { ReferenceDetailPane } from "./ReferenceDetailPane";
import { ReferenceListPane } from "./ReferenceListPane";

export function WorkbenchPage() {
  const { updateSettings } = useApp();
  const [showAdd, setShowAdd] = createSignal(false);
  const [errorMessage, setErrorMessage] = createSignal<string | null>(null);

  onMount(async () => {
    try {
      updateSettings(await loadSettings());
    } catch (error) {
      setErrorMessage(`加载设置失败: ${error}`);
    }
  });

  return (
    <div class="flex h-full min-h-0 overflow-hidden bg-base-100">
      <LibraryPane onAdd={() => setShowAdd(true)} onError={setErrorMessage} />
      <ReferenceListPane />
      <ReferenceDetailPane />
      <AddBibliography show={showAdd()} onClose={() => setShowAdd(false)} />
      <Show when={errorMessage()}>
        <div class="toast toast-end toast-top z-50">
          <div class="alert alert-error">
            <span>{errorMessage()}</span>
            <button type="button" class="btn btn-ghost btn-xs" onClick={() => setErrorMessage(null)}>Close</button>
          </div>
        </div>
      </Show>
    </div>
  );
}
```

- [ ] **Step 7: Run check**

```bash
bun run check
```

Expected:

```text
bun run check exits 0
```

- [ ] **Step 8: Commit**

```bash
git add src/components/workbench src/context/AppContext.tsx
# Subject: feat(workbench): add three-pane reference browser
git commit -F /tmp/bibcitex-ui-task-7-commit.txt
```

## Task 8: Route Main Pages to the Workbench

**Files:**
- Modify: `src/pages/HomePage.tsx`
- Modify: `src/pages/DetailPage.tsx`
- Modify: `src/App.tsx` only if route simplification is needed.

- [ ] **Step 1: Replace `HomePage` content with workbench**

Replace `src/pages/HomePage.tsx` with:

```tsx
import { WorkbenchPage } from "@/components/workbench/WorkbenchPage";

export default function HomePage() {
  return <WorkbenchPage />;
}
```

- [ ] **Step 2: Replace `DetailPage` with same workbench shell**

Replace `src/pages/DetailPage.tsx` with:

```tsx
import { WorkbenchPage } from "@/components/workbench/WorkbenchPage";

export default function DetailPage() {
  return <WorkbenchPage />;
}
```

- [ ] **Step 3: Run check**

```bash
bun run check
```

Expected:

```text
bun run check exits 0
```

- [ ] **Step 4: Commit**

```bash
git add src/pages/HomePage.tsx src/pages/DetailPage.tsx src/App.tsx
# Subject: feat(workbench): route main pages to workbench
git commit -F /tmp/bibcitex-ui-task-8-commit.txt
```

## Task 9: Update Nav, Modal, and Banner Styling

**Files:**
- Modify: `src/components/Nav.tsx`
- Modify: `src/components/AddBibliography.tsx`
- Modify: `src/components/updater/UpdateBanner.tsx`

- [ ] **Step 1: Replace Nav icon markup and add theme toggle**

Modify `src/components/Nav.tsx` to import and use:

```tsx
import { useApp } from "@/context/AppContext";
import { IconButton } from "@/components/ui/IconButton";
```

Inside `Nav`, read `theme` and `toggleTheme`:

```tsx
const { theme, toggleTheme } = useApp();
```

Replace manual update SVG button with:

```tsx
<IconButton
  icon="refresh"
  label="检查更新"
  tooltip="检查更新"
  size="sm"
  onClick={handleCheckUpdate}
/>
```

Add a theme toggle:

```tsx
<button type="button" class="btn btn-ghost btn-sm" onClick={toggleTheme}>
  {theme() === "latte" ? "Mocha" : "Latte"}
</button>
```

- [ ] **Step 2: Remove decorative modal blobs**

In `src/components/AddBibliography.tsx`, remove the absolute decorative blob elements and replace `glass-panel rounded-3xl` modal classes with:

```tsx
class="modal-box w-full max-w-2xl rounded-box border border-base-300 bg-base-100 shadow-xl"
```

Keep labels, validation, and save behavior unchanged.

- [ ] **Step 3: Keep update banner semantic**

In `src/components/updater/UpdateBanner.tsx`, keep `alert` / `bg-info` / `bg-success` / `bg-error` semantic classes. Replace any ad-hoc inline SVG icons with `SvgIcon`.

Example:

```tsx
<SvgIcon name="refresh" size={16} />
```

- [ ] **Step 4: Run emoji/icon scan**

Run:

```bash
rg -n "📖|📅|📄|✕|→|⏱" src
```

Expected:

```text
No matches in files modified by this task
```

- [ ] **Step 5: Run check**

```bash
bun run check
```

Expected:

```text
bun run check exits 0
```

- [ ] **Step 6: Commit**

```bash
git add src/components/Nav.tsx src/components/AddBibliography.tsx src/components/updater/UpdateBanner.tsx
# Subject: feat(ui): align shell controls with redesign
git commit -F /tmp/bibcitex-ui-task-9-commit.txt
```

## Task 10: Refactor Helper Command Palette

**Files:**
- Modify: `src/pages/HelperPage.tsx`

- [ ] **Step 1: Replace emoji and ad-hoc icons**

In `src/pages/HelperPage.tsx`, import:

```tsx
import { SvgIcon } from "@/components/ui/SvgIcon";
import {
  getReferenceTypeKey,
  getReferenceVenue,
  TYPE_STYLES,
  METADATA_STYLES,
} from "@/components/reference/semantic";
```

Replace all emoji-like metadata markers and inline SVG search icons with `SvgIcon`.

Example replacement:

```tsx
<SvgIcon name="search" size={20} />
```

- [ ] **Step 2: Use semantic row styling**

Add helper functions inside `HelperPage`:

```tsx
const getTypeStyle = (ref: Reference) => TYPE_STYLES[getReferenceTypeKey(ref.type_)];
const getVenue = (ref: Reference) => getReferenceVenue(ref);
```

Use row class pattern:

```tsx
class={`w-full text-left rounded-box border border-base-300 border-l-4 ${getTypeStyle(ref).borderClass} ${
  i() === selectedIndex() ? "bg-primary/10 ring-1 ring-primary" : "bg-base-100 hover:bg-base-200"
}`}
```

- [ ] **Step 3: Preserve command behavior**

Keep these existing behaviors unchanged:

```tsx
Escape -> hideHelperWindow()
ArrowDown -> next result
ArrowUp -> previous result
Enter -> handleSelect(results()[selectedIndex()])
handleSelect -> copyToClipboard + pasteToApp + hideHelperWindow
```

- [ ] **Step 4: Add paste failure fallback state**

Add:

```tsx
const [failedPasteKey, setFailedPasteKey] = createSignal<string | null>(null);
```

Change `handleSelect` catch block to:

```tsx
} catch (e) {
  console.error("Paste failed:", e);
  setFailedPasteKey(ref.cite_key);
  setErrorMessage(`粘贴失败，已保留 cite key，可手动复制: ${ref.cite_key}`);
  updateWindowHeight();
}
```

Show fallback near the error:

```tsx
<Show when={failedPasteKey()}>
  <button
    type="button"
    class="btn btn-primary btn-sm"
    onClick={() => copyToClipboard(failedPasteKey()!)}
  >
    Copy cite key
  </button>
</Show>
```

- [ ] **Step 5: Run check**

```bash
bun run check
```

Expected:

```text
bun run check exits 0
```

- [ ] **Step 6: Commit**

```bash
git add src/pages/HelperPage.tsx
# Subject: feat(helper): redesign citation command palette
git commit -F /tmp/bibcitex-ui-task-10-commit.txt
```

## Task 11: Remove Remaining Emoji UI Markers and Decorative Effects

**Files:**
- Modify: `src/app.css`
- Modify: any `src/**/*.tsx` files matched by the scans below.

- [ ] **Step 1: Remove decorative CSS utilities no longer used**

In `src/app.css`, remove or stop using these utility blocks if no imports reference them:

```css
.gradient-text
.card-shine::before
.animate-blob
.animation-delay-2000
.animation-delay-4000
@keyframes blob
@keyframes gradient-x
```

Keep functional utilities:

```css
.badge-soft
.animate-fade-in
.animate-fade-out
.backdrop-blur-spotlight
scrollbar styling
overflow safety rules
KaTeX styles
```

- [ ] **Step 2: Scan for emoji and decorative arrows**

Run:

```bash
rg -n "📖|📅|📄|✕|→|⏱|🎉|✨|🚀" src
```

For each match:

- Replace emoji icons with `SvgIcon`.
- Replace textual close glyph `✕` with `IconButton icon="x"`.
- Replace decorative arrow `→` with either text-only button label or `SvgIcon name="chevronRight"`.

- [ ] **Step 3: Scan for gradient/decorative classes**

Run:

```bash
rg -n "gradient-text|card-shine|animate-blob|blur-3xl|bg-linear-to-r|bg-linear-to-br" src
```

Expected after cleanup:

```text
No matches in core app UI components except intentional non-decorative gradients if explicitly justified in code review
```

- [ ] **Step 4: Run check**

```bash
bun run check
```

Expected:

```text
bun run check exits 0
```

- [ ] **Step 5: Commit**

```bash
git add src
# Subject: refactor(ui): remove emoji icons and decorative effects
git commit -F /tmp/bibcitex-ui-task-11-commit.txt
```

## Task 12: Visual and Interaction Verification

**Files:**
- No planned source modifications unless verification finds issues.

- [ ] **Step 1: Run static checks**

```bash
bun run check
bun run build
```

Expected:

```text
Both commands exit 0
```

- [ ] **Step 2: Start the dev server**

```bash
bun run dev
```

Expected:

```text
Vite dev server starts and prints a localhost URL
```

- [ ] **Step 3: Verify light and dark themes**

Open the dev server in the browser and verify:

```text
data-theme="latte" shows the whole main window in Catppuccin latte
data-theme="mocha" shows the whole main window in Catppuccin mocha
helper route /helper follows the same active theme
```

- [ ] **Step 4: Verify semantic color contrast**

Manual checks:

```text
Reference type badges and left borders are visibly distinct in latte and mocha
Metadata chips for cite key, author, year, venue, links, and notes have distinct backgrounds or borders
Chip text remains readable in both themes
No meaning depends on color alone; labels and icons remain visible
```

- [ ] **Step 5: Verify three-pane behavior**

Manual checks:

```text
Left pane shows bibliographies
Center pane shows references for selected bibliography
Right pane shows selected reference details
Left pane can hide and show
Right pane can hide and show
Both panes hidden leaves full-width reference list
Rows do not jump height when panes toggle
```

- [ ] **Step 6: Verify helper keyboard behavior**

Manual checks:

```text
Open /helper
Search input receives focus
ArrowDown moves active selection
ArrowUp moves active selection
Enter calls paste path for selected cite key
Escape closes or hides helper
Paste failure keeps fallback copy action visible
```

- [ ] **Step 7: Verify SVG-only icon rule**

Run:

```bash
rg -n "📖|📅|📄|✕|→|⏱|🎉|✨|🚀" src
```

Expected:

```text
No matches
```

- [ ] **Step 8: Commit verification fixes discovered during this task**

If verification changes source files:

```bash
git add src package.json bun.lock
# Subject: fix(ui): polish redesigned workbench verification
git commit -F /tmp/bibcitex-ui-task-12-commit.txt
```

If no fixes were needed, do not create an empty commit.
