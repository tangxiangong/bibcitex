import { For, Show, createSignal } from "solid-js";
import { useApp } from "../../context/AppContext.tsx";
import { loadBibliography, loadSettings, openFile, removeBibliography } from "../../tauri.ts";
import IconButton from "../ui/IconButton.tsx";
import SvgIcon from "../ui/SvgIcon.tsx";

interface LibraryPaneProps {
  onAddBibliography: () => void;
}

export default function LibraryPane(props: LibraryPaneProps) {
  const { bibliographyList, currentBibName, selectBibliography, updateSettings } =
    useApp();
  const [busyName, setBusyName] = createSignal<string | null>(null);
  const [errorMessage, setErrorMessage] = createSignal<string | null>(null);

  const handleSelect = async (name: string, path: string) => {
    setBusyName(name);
    setErrorMessage(null);
    try {
      const refs = await loadBibliography(path);
      selectBibliography(name, refs);
    } catch (e) {
      setErrorMessage(`Failed to load bibliography: ${e}`);
    } finally {
      setBusyName(null);
    }
  };

  const handleDelete = async (event: MouseEvent, name: string) => {
    event.stopPropagation();
    setBusyName(name);
    setErrorMessage(null);
    try {
      await removeBibliography(name);
      updateSettings(await loadSettings());
    } catch (e) {
      setErrorMessage(`Failed to remove bibliography: ${e}`);
    } finally {
      setBusyName(null);
    }
  };

  const handleOpen = async (event: MouseEvent, path: string) => {
    event.stopPropagation();
    setErrorMessage(null);
    try {
      await openFile(path);
    } catch (e) {
      setErrorMessage(`Failed to open file: ${e}`);
    }
  };

  return (
    <aside class="flex h-full min-w-64 max-w-80 basis-72 flex-col border-r border-base-300 bg-base-100">
      <div class="flex h-14 shrink-0 items-center justify-between border-b border-base-300 px-3">
        <div class="flex min-w-0 items-center gap-2">
          <SvgIcon name="library" class="h-4 w-4 shrink-0 text-base-content/70" aria-hidden />
          <h2 class="truncate text-sm font-semibold">Libraries</h2>
        </div>
        <IconButton
          icon="folderOpen"
          label="Add bibliography"
          size="sm"
          variant="primary"
          onClick={props.onAddBibliography}
        />
      </div>

      <Show when={errorMessage()}>
        <div class="m-3 rounded border border-error/30 bg-error/10 px-3 py-2 text-xs text-error">
          {errorMessage()}
        </div>
      </Show>

      <div class="min-h-0 flex-1 overflow-y-auto py-2">
        <Show
          when={bibliographyList().length > 0}
          fallback={
            <div class="px-4 py-8 text-center text-sm text-base-content/55">
              No bibliography yet.
            </div>
          }
        >
          <For each={bibliographyList()}>
            {(bib) => {
              const selected = () => currentBibName() === bib.name;
              const busy = () => busyName() === bib.name;
              return (
                <div
                  class={[
                    "group flex w-full items-center gap-2 border-l-4 px-3 py-2.5 text-left transition-colors",
                    selected()
                      ? "border-primary bg-primary/10"
                      : "border-transparent hover:bg-base-200",
                  ].join(" ")}
                >
                  <button
                    type="button"
                    class="flex min-w-0 flex-1 items-center gap-2 text-left"
                    onClick={() => handleSelect(bib.name, bib.path)}
                  >
                    <SvgIcon name="book" class="h-4 w-4 shrink-0 text-base-content/65" aria-hidden />
                    <div class="min-w-0 flex-1">
                      <div class="truncate text-sm font-medium">{bib.name}</div>
                      <div class="truncate text-xs text-base-content/50" title={bib.path}>
                        {bib.path}
                      </div>
                    </div>
                  </button>
                  <Show when={busy()}>
                    <span class="loading loading-spinner loading-xs shrink-0" />
                  </Show>
                  <IconButton
                    icon="externalLink"
                    label={`Open ${bib.name}`}
                    size="xs"
                    class="opacity-0 group-hover:opacity-100 focus:opacity-100"
                    onClick={(event) => handleOpen(event, bib.path)}
                  />
                  <IconButton
                    icon="trash"
                    label={`Remove ${bib.name}`}
                    size="xs"
                    variant="error"
                    class="opacity-0 group-hover:opacity-100 focus:opacity-100"
                    onClick={(event) => handleDelete(event, bib.name)}
                  />
                </div>
              );
            }}
          </For>
        </Show>
      </div>
    </aside>
  );
}
