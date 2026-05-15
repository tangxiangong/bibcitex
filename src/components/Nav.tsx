import { A } from "@solidjs/router";
import { emit } from "@tauri-apps/api/event";
import { openHelperWindow } from "../tauri.ts";
import { useApp } from "../context/AppContext.tsx";
import { TRANSPARENT_LOGO } from "../constants/icons.ts";
import { IconButton } from "./ui/IconButton";
import { SvgIcon } from "./ui/SvgIcon";

const CMD_CTRL = navigator.platform.includes("Mac") ? "Cmd" : "Win";

function Nav() {
  const { theme, toggleTheme } = useApp();

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
    <div class="navbar bg-base-100 border-b border-base-300 shrink-0 z-40 sticky top-0 min-h-14 px-2">
      <div class="navbar-start pl-4">
        <A
          href="/"
          class="flex items-center gap-3 rounded-field px-2 py-1 hover:bg-base-200 transition-colors"
        >
          <div class="w-8 h-8 relative">
            <img
              src={TRANSPARENT_LOGO}
              alt="BibCiTeX Logo"
              class="w-full h-full object-contain"
            />
          </div>
          <span class="font-semibold text-lg tracking-tight text-base-content hidden sm:block">
            BibCiTeX
          </span>
        </A>
      </div>

      <div class="navbar-center"></div>

      <div class="navbar-end pr-4 gap-1">
        <IconButton
          icon="refresh"
          label="检查更新"
          size="sm"
          onClick={handleCheckUpdate}
        />
        <IconButton
          icon={theme() === "latte" ? "moon" : "sun"}
          label={theme() === "latte" ? "切换到深色主题" : "切换到浅色主题"}
          size="sm"
          onClick={toggleTheme}
        />
        <button
          type="button"
          class="btn btn-ghost btn-sm gap-2 font-normal text-base-content/80"
          onClick={handleOpenSpotlight}
        >
          <SvgIcon name="search" size={16} aria-hidden />
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
