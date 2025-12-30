import React, { useState } from "react";
import type { Reference } from "../../types.ts";
import { useApp } from "../../context/AppContext.tsx";
import { copyToClipboard, openFile, openUrl } from "../../tauri.ts";
import ChunksComp from "../ChunksComp.tsx";

interface ReferenceCardProps {
  entry: Reference;
}

function ReferenceCard({ entry }: ReferenceCardProps) {
  const { openDrawer } = useApp();
  const [copied, setCopied] = useState(false);
  const [copySuccess, setCopySuccess] = useState(true);

  const borderColorClass = React.useMemo(() => {
    const type = entry.type_;
    if (type === "Article") return "border-l-blue-500";
    if (type === "Book") return "border-l-green-500";
    if (
      type === "Thesis" || type === "MastersThesis" || type === "PhdThesis"
    ) {
      return "border-l-purple-500";
    }
    if (type === "InProceedings") return "border-l-orange-500";
    if (type === "TechReport") return "border-l-yellow-500";
    if (type === "Misc") return "border-l-gray-500";
    return "border-l-primary";
  }, [entry.type_]);

  const badgeClass = React.useMemo(() => {
    const type = entry.type_;
    if (type === "Article") return "badge-info";
    if (type === "Book") return "badge-success";
    if (
      type === "Thesis" || type === "MastersThesis" || type === "PhdThesis"
    ) {
      return "badge-secondary";
    }
    if (type === "InProceedings") return "badge-warning";
    if (type === "TechReport") return "badge-accent";
    return "badge-ghost";
  }, [entry.type_]);

  const typeLabel = React.useMemo(() => {
    const type = entry.type_;
    if (typeof type === "string") return type;
    if (typeof type === "object" && "Unknown" in type) return type.Unknown;
    return "Unknown";
  }, [entry.type_]);

  const doiUrl = React.useMemo(
    () => (entry.doi ? `https://doi.org/${entry.doi}` : ""),
    [entry.doi],
  );

  const handleCopyKey = async () => {
    setCopied(true);
    try {
      await copyToClipboard(entry.cite_key);
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
    <div
      className={`card-modern card-shine group hover:-translate-y-1 transition-all duration-300 m-4 border-l-4 ${borderColorClass}`}
    >
      <div className="card-body p-5">
        <div className="flex justify-between items-start gap-4">
          <div className="flex-1">
            <div className="flex items-center gap-2 mb-2">
              <span
                className={`badge ${badgeClass} badge-soft badge-sm font-bold`}
              >
                {typeLabel}
              </span>
              <span className="text-xs font-mono opacity-50 select-all">
                {entry.cite_key}
              </span>
            </div>
            {entry.title
              ? (
                <h3 className="text-xl font-bold leading-snug gradient-text">
                  <ChunksComp chunks={entry.title} citeKey={entry.cite_key} />
                </h3>
              )
              : (
                <span className="text-lg text-base-content/50 italic">
                  No title available
                </span>
              )}
          </div>

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
                    src="/assets/icons/copy.svg"
                    alt="Copy"
                    className="h-4 w-4 opacity-70"
                  />
                )
                : copySuccess
                ? (
                  <img
                    src="/assets/icons/ok.svg"
                    alt="Success"
                    className="h-4 w-4 text-success"
                  />
                )
                : (
                  <img
                    src="/assets/icons/error.svg"
                    alt="Error"
                    className="h-4 w-4 text-error"
                  />
                )}
            </button>
            <button
              type="button"
              className="btn btn-ghost btn-sm btn-circle tooltip tooltip-left"
              data-tip="Details"
              aria-label="Details"
              onClick={handleOpenDrawer}
            >
              <img
                src="/assets/icons/details.svg"
                alt="Details"
                className="h-4 w-4 opacity-70"
              />
            </button>
          </div>
        </div>

        <div className="mt-3 flex flex-wrap gap-2">
          {entry.author
            ? (
              entry.author.map((author, idx) => (
                <span
                  key={idx}
                  className="badge badge-ghost hover:badge-info transition-colors cursor-default bg-base-200/50"
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

        <div className="mt-4 flex flex-wrap items-center gap-4 text-sm text-base-content/70 border-t border-base-content/5 pt-3">
          {entry.journal && (
            <div className="flex items-center gap-1">
              <span className="font-semibold text-primary">📖</span>
              <span className="italic">{entry.journal}</span>
            </div>
          )}
          {entry.year && (
            <div className="flex items-center gap-1">
              <span className="font-semibold text-secondary">📅</span>
              <span>{entry.year}</span>
            </div>
          )}

          <div className="flex-1"></div>

          {entry.doi && (
            <button
              type="button"
              className="btn btn-xs btn-ghost gap-1 hover:text-info"
              onClick={handleOpenDoi}
            >
              DOI
            </button>
          )}
          {entry.url && (
            <button
              type="button"
              className="btn btn-xs btn-ghost gap-1 hover:text-info"
              onClick={handleOpenUrl}
            >
              URL
            </button>
          )}
          {entry.file && (
            <button
              type="button"
              className="btn btn-xs btn-primary btn-soft gap-1"
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

export default ReferenceCard;
