import { createEffect, Show } from "solid-js";
import { useApp } from "../context/AppContext.tsx";
import ReferenceDrawer from "./drawer/ReferenceDrawer.tsx";
import ChunksComp from "./ChunksComp.tsx";
import { IconButton } from "./ui/IconButton";
import { SvgIcon } from "./ui/SvgIcon";

function Drawer() {
  const { drawerOpen, drawerReference, closeDrawer } = useApp();

  createEffect(() => {
    const checkbox = document.getElementById(
      "my-drawer",
    ) as HTMLInputElement;
    if (checkbox) {
      checkbox.checked = drawerOpen();
    }
  });

  const handleClose = () => {
    closeDrawer();
  };

  return (
    <div class="drawer-side z-50">
      <label
        for="my-drawer"
        aria-label="关闭侧栏"
        class="drawer-overlay backdrop-blur-sm"
        onClick={handleClose}
      >
      </label>
      <div class="min-h-full w-120 max-w-[90vw] bg-base-100 shadow-2xl p-0 flex flex-col border-l border-base-content/5">
        {/* Drawer Header */}
        <div class="p-4 border-b border-base-content/5 flex justify-between items-start bg-base-100/95 backdrop-blur sticky top-0 z-10">
          <div class="flex-1 pr-4">
            <h3 class="text-lg font-bold leading-tight">
              <Show when={drawerReference()?.title}>
                <ChunksComp
                  chunks={drawerReference()!.title!}
                  citeKey={drawerReference()!.cite_key}
                />
              </Show>
            </h3>
          </div>
          <IconButton
            icon="x"
            label="关闭详情"
            size="sm"
            onClick={handleClose}
          />
        </div>

        {/* Drawer Content */}
        <div class="flex-1 overflow-y-auto p-4">
          <Show
            when={drawerReference()}
            fallback={
              <div class="flex flex-col items-center justify-center h-full text-base-content/50">
                <SvgIcon name="fileText" size={40} class="mb-2" aria-hidden />
                <span>尚未选择任何参考文献</span>
              </div>
            }
          >
            <ReferenceDrawer entry={drawerReference()!} />
          </Show>
        </div>
      </div>
    </div>
  );
}

export default Drawer;
