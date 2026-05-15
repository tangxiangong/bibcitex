import { Show, createSignal, onMount } from "solid-js";
import AddBibliography from "../AddBibliography.tsx";
import { useApp } from "../../context/AppContext.tsx";
import { loadSettings } from "../../tauri.ts";
import LibraryPane from "./LibraryPane.tsx";
import ReferenceDetailPane from "./ReferenceDetailPane.tsx";
import ReferenceListPane from "./ReferenceListPane.tsx";

export default function WorkbenchPage() {
  const { updateSettings, leftPaneOpen, rightPaneOpen } = useApp();
  const [addBibliographyOpen, setAddBibliographyOpen] = createSignal(false);
  const [errorMessage, setErrorMessage] = createSignal<string | null>(null);

  const isBrowserPreviewError = (error: unknown) =>
    String(error).includes("invoke");

  onMount(async () => {
    try {
      updateSettings(await loadSettings());
      setErrorMessage(null);
    } catch (e) {
      if (isBrowserPreviewError(e)) {
        setErrorMessage(null);
        return;
      }
      setErrorMessage(`Failed to load settings: ${e}`);
    }
  });

  return (
    <div class="flex h-full min-h-0 w-full overflow-hidden bg-base-100 text-base-content">
      <Show when={leftPaneOpen()}>
        <LibraryPane onAddBibliography={() => setAddBibliographyOpen(true)} />
      </Show>

      <div class="flex min-w-0 flex-1 flex-col">
        <Show when={errorMessage()}>
          <div class="shrink-0 border-b border-error/30 bg-error/10 px-4 py-2 text-sm text-error">
            {errorMessage()}
          </div>
        </Show>
        <div class="flex min-h-0 flex-1">
          <ReferenceListPane />
          <Show when={rightPaneOpen()}>
            <ReferenceDetailPane />
          </Show>
        </div>
      </div>

      <AddBibliography
        show={addBibliographyOpen()}
        onClose={() => setAddBibliographyOpen(false)}
      />
    </div>
  );
}
