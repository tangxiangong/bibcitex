import { A } from "@solidjs/router";
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

      <div class="navbar-end pr-4">
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
