import React, { useEffect } from "react";
import { useApp } from "../context/AppContext.tsx";
import ReferenceDrawer from "./reference/ReferenceDrawer.tsx";
import ChunksComp from "./ChunksComp.tsx";

function Drawer() {
  const { drawerOpen, drawerReference, closeDrawer } = useApp();

  useEffect(() => {
    const checkbox = document.getElementById(
      "reference-drawer",
    ) as HTMLInputElement;
    if (checkbox) {
      checkbox.checked = drawerOpen;
    }
  }, [drawerOpen]);

  const handleClose = () => {
    closeDrawer();
  };

  return (
    <div className="drawer-side z-50">
      <label
        htmlFor="reference-drawer"
        aria-label="close sidebar"
        className="drawer-overlay backdrop-blur-sm"
        onClick={handleClose}
      >
      </label>
      <div className="min-h-full w-120 max-w-[90vw] bg-base-100 shadow-2xl p-0 flex flex-col border-l border-base-content/5">
        {/* Drawer Header */}
        <div className="p-4 border-b border-base-content/5 flex justify-between items-start bg-base-100/95 backdrop-blur sticky top-0 z-10">
          <div className="flex-1 pr-4">
            <h3 className="text-lg font-bold leading-tight">
              {drawerReference?.title && (
                <ChunksComp
                  chunks={drawerReference.title}
                  citeKey={drawerReference.cite_key}
                />
              )}
            </h3>
          </div>
          <button
            type="button"
            className="btn btn-sm btn-circle btn-ghost"
            onClick={handleClose}
          >
            ✕
          </button>
        </div>

        {/* Drawer Content */}
        <div className="flex-1 overflow-y-auto p-4">
          {drawerReference
            ? <ReferenceDrawer entry={drawerReference} />
            : (
              <div className="flex flex-col items-center justify-center h-full text-base-content/50">
                <span className="text-4xl mb-2">📄</span>
                <span>尚未选择任何参考文献</span>
              </div>
            )}
        </div>
      </div>
    </div>
  );
}

export default Drawer;
