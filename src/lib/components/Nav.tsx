import React from "react";
import { Link } from "react-router-dom";
import { openHelperWindow } from "../tauri.ts";

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
    <div className="navbar bg-base-100/80 backdrop-blur-md border-b border-base-content/5 shrink-0 z-40 sticky top-0">
      <div className="navbar-start pl-4">
        <Link
          to="/"
          className="flex items-center gap-3 hover:opacity-80 transition-opacity"
        >
          <div className="w-10 h-10 relative">
            <img
              src="/assets/transparent_logo.png"
              alt="BibCiTeX Logo"
              className="w-full h-full object-contain"
            />
          </div>
          <span className="font-bold text-xl tracking-tight gradient-text hidden sm:block">
            BibCiTeX
          </span>
        </Link>
      </div>

      <div className="navbar-center"></div>

      <div className="navbar-end pr-4">
        <button
          type="button"
          className="btn btn-ghost btn-sm gap-2 hover:bg-base-content/5 font-normal text-base-content/70"
          onClick={handleOpenSpotlight}
        >
          <span>快捷助手</span>
          <div className="hidden md:flex gap-1">
            <kbd className="kbd kbd-sm font-mono bg-base-200 border-base-300">
              {CMD_CTRL}
            </kbd>
            <kbd className="kbd kbd-sm font-mono bg-base-200 border-base-300">
              shift
            </kbd>
            <kbd className="kbd kbd-sm font-mono bg-base-200 border-base-300">
              k
            </kbd>
          </div>
        </button>
      </div>
    </div>
  );
}

export default Nav;
