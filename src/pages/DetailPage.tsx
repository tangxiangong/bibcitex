import { Show } from "solid-js";
import { A } from "@solidjs/router";
import { useApp } from "@/context/AppContext.tsx";
import References from "@/components/References.tsx";
import { TRANSPARENT_LOGO } from "@/constants/icons.ts";

function DetailPage() {
  const { currentBibName, currentReferences } = useApp();

  return (
    <div class="flex flex-col h-full overflow-hidden bg-base-200/30">
      <div class="shrink-0 p-4 bg-base-100 border-b border-base-300 flex items-center justify-between">
        <div class="flex items-center gap-4">
          <div>
            <h1 class="text-2xl font-bold gradient-text">
              {currentBibName() || "Bibliography Details"}
            </h1>
            <p class="text-sm text-base-content/60">
              {currentReferences()?.length || 0} references
            </p>
          </div>
        </div>
      </div>

      <div class="flex-1 overflow-hidden">
        <Show
          when={currentReferences() && currentReferences()!.length > 0}
          fallback={
            <div class="flex flex-col items-center justify-center h-full text-base-content/50">
              <img
                src={TRANSPARENT_LOGO}
                alt="No references"
                class="w-24 h-24 mb-4 opacity-30"
              />
              <p class="text-lg font-medium">No references loaded</p>
              <p class="text-sm">
                Select a bibliography from the home page
              </p>
              <A href="/" class="btn btn-primary mt-4">
                Go to Home
              </A>
            </div>
          }
        >
          <References />
        </Show>
      </div>
    </div>
  );
}

export default DetailPage;
