import React from "react";
import { Link } from "react-router-dom";
import { useApp } from "../lib/context/AppContext.tsx";
import References from "../lib/components/References.tsx";

function DetailPage() {
  const { currentBibName, currentReferences } = useApp();

  return (
    <div className="flex flex-col h-full overflow-hidden bg-base-200/30">
      <div className="shrink-0 p-4 bg-base-100 border-b border-base-300 flex items-center justify-between">
        <div className="flex items-center gap-4">
          <Link to="/" className="btn btn-ghost btn-sm gap-2">
            <img
              src="/assets/icons/cancel.svg"
              alt="Back"
              className="h-4 w-4"
            />
            返回
          </Link>
          <div className="divider divider-horizontal m-0"></div>
          <div>
            <h1 className="text-2xl font-bold gradient-text">
              {currentBibName || "Bibliography Details"}
            </h1>
            <p className="text-sm text-base-content/60">
              {currentReferences?.length || 0} references
            </p>
          </div>
        </div>
      </div>

      <div className="flex-1 overflow-hidden">
        {currentReferences && currentReferences.length > 0
          ? <References />
          : (
            <div className="flex flex-col items-center justify-center h-full text-base-content/50">
              <img
                src="/assets/transparent_logo.png"
                alt="No references"
                className="w-24 h-24 mb-4 opacity-30"
              />
              <p className="text-lg font-medium">No references loaded</p>
              <p className="text-sm">
                Select a bibliography from the home page
              </p>
              <Link to="/" className="btn btn-primary mt-4">
                Go to Home
              </Link>
            </div>
          )}
      </div>
    </div>
  );
}

export default DetailPage;
