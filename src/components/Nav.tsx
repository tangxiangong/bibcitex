import { A } from "@solidjs/router";
import { emit } from "@tauri-apps/api/event";
import { openHelperWindow } from "../tauri.ts";
import { TRANSPARENT_LOGO } from "../constants/icons.ts";

const CMD_CTRL = navigator.platform.includes("Mac") ? "⌘" : "Win";

function Nav() {
  const handleOpenSpotlight = async () => {
    try {
      await openHelperWindow();
    } catch (e) {
      console.error("Failed to open helper window:", e);
    }
  };

  const handleCheckUpdate = () => {
    emit("check-update-trigger");
  };

  return (
    <div class="navbar bg-base-100/80 backdrop-blur-md border-b border-base-content/5 shrink-0 z-40 sticky top-0">
      <div class="navbar-start pl-4">
        <A
          href="/"
          class="flex items-center gap-3 hover:opacity-80 transition-opacity"
        >
          <div class="w-10 h-10 relative">
            <img
              src={TRANSPARENT_LOGO}
              alt="BibCiTeX Logo"
              class="w-full h-full object-contain"
            />
          </div>
          <span class="font-bold text-xl tracking-tight gradient-text hidden sm:block">
            BibCiTeX
          </span>
        </A>
      </div>

      <div class="navbar-center"></div>

      <div class="navbar-end pr-4 gap-1">
        <button
          type="button"
          class="btn btn-ghost btn-sm btn-square text-base-content/50 hover:text-base-content/80 tooltip tooltip-bottom"
          data-tip="检查更新"
          onClick={handleCheckUpdate}
        >
          <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
            <path stroke-linecap="round" stroke-linejoin="round" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
          </svg>
        </button>
        <button
          type="button"
          class="btn btn-ghost btn-sm gap-2 hover:bg-base-content/5 font-normal text-base-content/70"
          onClick={handleOpenSpotlight}
        >
          <span>快捷助手</span>
          <div class="hidden md:flex gap-1">
            <kbd class="kbd kbd-sm font-mono bg-base-200 border-base-300">
              {CMD_CTRL}
            </kbd>
            <kbd class="kbd kbd-sm font-mono bg-base-200 border-base-300">
              shift
            </kbd>
            <kbd class="kbd kbd-sm font-mono bg-base-200 border-base-300">
              k
            </kbd>
          </div>
        </button>
      </div>
    </div>
  );
}

export default Nav;
