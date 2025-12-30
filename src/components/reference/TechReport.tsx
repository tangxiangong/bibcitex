import { useState } from "react";
import {
  COPY_ICON,
  DETAILS_ICON,
  ERROR_ICON,
  OK_ICON,
} from "../../constants/icons.ts";
import type { Reference } from "../../types.ts";
import { useApp } from "../../context/AppContext.tsx";
import { copyToClipboard, openFile, openUrl } from "../../tauri.ts";
import ChunksComp from "../ChunksComp.tsx";

interface TechReportProps {
  entry: Reference;
}

function TechReport({ entry }: TechReportProps) {
  const { openDrawer } = useApp();
  const [copied, setCopied] = useState(false);
  const [copySuccess, setCopySuccess] = useState(true);

  const key = entry.cite_key;
  const doiUrl = entry.doi ? `https://doi.org/${entry.doi}` : "";

  const handleCopyKey = async () => {
    setCopied(true);
    try {
      await copyToClipboard(key);
      setCopySuccess(true);
    } catch {
      setCopySuccess(false);
    }
    setTimeout(() => {
      setCopied(false);
    }, 1500);
  };

  const handleOpenDrawer = () => {
    openDrawer(entry);
  };

  const handleOpenDoi = async () => {
    if (doiUrl) {
      await openUrl(doiUrl);
    }
  };

  const handleOpenUrl = async () => {
    if (entry.url) {
      await openUrl(entry.url);
    }
  };

  const handleOpenFile = async () => {
    if (entry.file) {
      await openFile(entry.file);
    }
  };

  return (
    <div className="card-modern card-shine group hover:-translate-y-1 transition-all duration-300 m-4 border-l-4 border-l-amber-500">
      <div className="card-body p-5">
        {/* Header: Type + Title + Actions */}
        <div className="flex justify-between items-start gap-4">
          <div className="flex-1">
            <div className="flex items-center gap-2 mb-2">
              <span className="badge badge-warning badge-soft badge-sm font-bold">
                TechReport
              </span>
              <span className="text-xs font-mono opacity-50 select-all">
                {key}
              </span>
            </div>
            {entry.title
              ? (
                <h3 className="text-xl font-bold leading-snug gradient-text">
                  <ChunksComp chunks={entry.title} citeKey={key} />
                </h3>
              )
              : (
                <span className="text-lg text-base-content/50 italic">
                  No title available
                </span>
              )}
          </div>

          {/* Actions */}
          <div className="flex items-center gap-1 opacity-0 group-hover:opacity-100 transition-opacity duration-200">
            <button
              type="button"
              className="btn btn-ghost btn-sm btn-circle tooltip tooltip-left"
              data-tip="Copy Key"
              onClick={handleCopyKey}
            >
              {!copied
                ? (
                  <img
                    width={18}
                    src={COPY_ICON}
                    alt="Copy"
                    className="opacity-70"
                  />
                )
                : copySuccess
                ? (
                  <img
                    width={18}
                    src={OK_ICON}
                    alt="Success"
                    className="text-success"
                  />
                )
                : (
                  <img
                    width={18}
                    src={ERROR_ICON}
                    alt="Error"
                    className="text-error"
                  />
                )}
            </button>
            <button
              type="button"
              className="btn btn-ghost btn-sm btn-circle tooltip tooltip-left"
              data-tip="Details"
              onClick={handleOpenDrawer}
            >
              <img
                width={18}
                src={DETAILS_ICON}
                alt="Details"
                className="opacity-70"
              />
            </button>
          </div>
        </div>

        {/* Authors */}
        <div className="mt-3 flex flex-wrap gap-2">
          {entry.author && entry.author.length > 0
            ? (
              entry.author.map((author, idx) => (
                <span
                  key={idx}
                  className="badge badge-ghost hover:badge-warning transition-colors cursor-default bg-base-200/50"
                >
                  {author}
                </span>
              ))
            )
            : (
              <span className="text-sm text-base-content/50 italic">
                Unknown Author
              </span>
            )}
        </div>

        {/* Metadata Row */}
        <div className="mt-4 flex flex-wrap items-center gap-4 text-sm text-base-content/70 border-t border-base-content/5 pt-3">
          {entry.institution && (
            <div className="flex items-center gap-1">
              <span className="font-semibold text-primary">🏢</span>
              <span>{entry.institution}</span>
            </div>
          )}
          {entry.year && (
            <div className="flex items-center gap-1">
              <span className="font-semibold text-secondary">📅</span>
              <span>{entry.year}</span>
            </div>
          )}

          {/* Links */}
          <div className="flex-1"></div>

          {entry.doi && (
            <button
              type="button"
              className="btn btn-xs btn-ghost gap-1 hover:text-warning"
              onClick={handleOpenDoi}
            >
              DOI
            </button>
          )}
          {entry.url && (
            <button
              type="button"
              className="btn btn-xs btn-ghost gap-1 hover:text-warning"
              onClick={handleOpenUrl}
            >
              URL
            </button>
          )}
          {entry.file && (
            <button
              type="button"
              className="btn btn-xs btn-warning btn-soft gap-1"
              onClick={handleOpenFile}
            >
              PDF
            </button>
          )}
        </div>
      </div>
    </div>
  );
}

export default TechReport;
