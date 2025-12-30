import React, { useMemo } from "react";
import type { Reference } from "../../types.ts";
import { openFile, openUrl } from "../../tauri.ts";
import ChunksComp from "../ChunksComp.tsx";

interface ReferenceDrawerProps {
  entry: Reference;
}

function ReferenceDrawer({ entry }: ReferenceDrawerProps) {
  const typeLabel = useMemo(() => {
    const type = entry.type_;
    if (typeof type === "string") {
      if (type === "Article") return "Journal Article";
      if (type === "MastersThesis") return "Master's Thesis";
      if (type === "PhdThesis") return "PhD Thesis";
      return type;
    }
    if (typeof type === "object" && "Unknown" in type) return type.Unknown;
    return "Unknown";
  }, [entry.type_]);

  const pagesString = useMemo(() => {
    if (!entry.pages) return "";
    if (entry.pages.start === entry.pages.end) {
      return entry.pages.start.toString();
    }
    return `${entry.pages.start}-${entry.pages.end}`;
  }, [entry.pages]);

  const bibtexLines = useMemo(
    () => entry.source.split("\n"),
    [entry.source],
  );

  const doiUrl = useMemo(
    () => (entry.doi ? `https://doi.org/${entry.doi}` : ""),
    [entry.doi],
  );

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
    <div className="space-y-2">
      {/* Info Collapse */}
      <div className="collapse collapse-arrow bg-base-200/30 hover:bg-base-200/50 transition-colors rounded-box">
        <input type="checkbox" defaultChecked />
        <div className="collapse-title font-medium">Info</div>
        <div className="collapse-content">
          <table className="table table-sm">
            <tbody>
              <tr>
                <td className="text-right opacity-70 font-semibold">Type</td>
                <td>{typeLabel}</td>
              </tr>
              <tr>
                <td className="text-right opacity-70 font-semibold">Key</td>
                <td>{entry.cite_key}</td>
              </tr>
              <tr>
                <td className="text-right opacity-70 font-semibold">Title</td>
                <td>
                  {entry.title && (
                    <ChunksComp
                      chunks={entry.title}
                      citeKey={`${entry.cite_key}-drawer-title`}
                    />
                  )}
                </td>
              </tr>
              {entry.author
                ? (
                  entry.author.map((author, idx) => (
                    <tr key={idx}>
                      <td className="text-right">Author</td>
                      <td>{author}</td>
                    </tr>
                  ))
                )
                : (
                  <tr>
                    <td className="text-right">Author</td>
                    <td></td>
                  </tr>
                )}
              {entry.full_journal
                ? (
                  <>
                    <tr>
                      <td className="text-right">Journal</td>
                      <td>{entry.full_journal}</td>
                    </tr>
                    <tr>
                      <td className="text-right">Journal Abbr</td>
                      <td>{entry.journal || ""}</td>
                    </tr>
                  </>
                )
                : (
                  entry.journal && (
                    <tr>
                      <td className="text-right">Journal</td>
                      <td>{entry.journal}</td>
                    </tr>
                  )
                )}
              <tr>
                <td className="text-right">Volume</td>
                <td>{entry.volume || ""}</td>
              </tr>
              <tr>
                <td className="text-right">Number</td>
                <td>{entry.number || ""}</td>
              </tr>
              <tr>
                <td className="text-right">Pages</td>
                <td>{pagesString}</td>
              </tr>
              <tr>
                <td className="text-right">Year</td>
                <td>{entry.year || ""}</td>
              </tr>
              <tr>
                <td className="text-right">DOI</td>
                <td className="break-all">
                  {entry.doi && (
                    <button
                      type="button"
                      className="tooltip cursor-pointer text-left break-all hover:text-primary"
                      data-tip="在浏览器中打开"
                      onClick={handleOpenDoi}
                    >
                      {entry.doi}
                    </button>
                  )}
                </td>
              </tr>
              <tr>
                <td className="text-right">URL</td>
                <td className="break-all">
                  {entry.url && (
                    <button
                      type="button"
                      className="tooltip cursor-pointer text-left break-all hover:text-primary"
                      data-tip="在浏览器中打开"
                      onClick={handleOpenUrl}
                    >
                      {entry.url}
                    </button>
                  )}
                </td>
              </tr>
              <tr>
                <td className="text-right">File</td>
                <td>
                  {entry.file && (
                    <button
                      type="button"
                      className="tooltip cursor-pointer text-left break-all hover:text-primary"
                      data-tip="打开"
                      onClick={handleOpenFile}
                    >
                      {entry.file}
                    </button>
                  )}
                </td>
              </tr>
            </tbody>
          </table>
        </div>
      </div>

      {/* Abstract Collapse */}
      <div className="collapse collapse-arrow bg-base-200/30 hover:bg-base-200/50 transition-colors rounded-box">
        <input type="checkbox" />
        <div className="collapse-title font-medium">Abstract</div>
        <div className="collapse-content">
          {entry.abstract_ && (
            <ChunksComp
              chunks={entry.abstract_}
              citeKey={`${entry.cite_key}-abstract`}
            />
          )}
        </div>
      </div>

      {/* Note Collapse */}
      <div className="collapse collapse-arrow bg-base-200/30 hover:bg-base-200/50 transition-colors rounded-box">
        <input type="checkbox" />
        <div className="collapse-title font-medium">Note</div>
        <div className="collapse-content">
          {entry.note && (
            <ChunksComp
              chunks={entry.note}
              citeKey={`${entry.cite_key}-note`}
            />
          )}
        </div>
      </div>

      {/* BibTeX Collapse */}
      <div className="collapse collapse-arrow bg-base-200/30 hover:bg-base-200/50 transition-colors rounded-box">
        <input type="checkbox" />
        <div className="collapse-title font-medium">BibTeX</div>
        <div className="collapse-content">
          {bibtexLines.map((line, idx) => (
            <p key={idx} className="font-mono text-xs">
              {line}
            </p>
          ))}
        </div>
      </div>
    </div>
  );
}

export default ReferenceDrawer;
