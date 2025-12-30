import React from "react";
import type { Reference } from "../../types.ts";
import ChunksComp from "../ChunksComp.tsx";
import { openFile, openUrl } from "../../tauri.ts";

interface InProceedingsDrawerProps {
  entry: Reference;
}

function InProceedingsDrawer({ entry }: InProceedingsDrawerProps) {
  const key = entry.cite_key;
  const pagesString = entry.pages
    ? entry.pages.start === entry.pages.end
      ? entry.pages.start.toString()
      : `${entry.pages.start}-${entry.pages.end}`
    : "";
  const bibtex = entry.source.split("\n");
  const doiUrl = entry.doi ? `https://doi.org/${entry.doi}` : "";

  const handleOpenDoi = () => {
    if (doiUrl) {
      openUrl(doiUrl);
    }
  };

  const handleOpenUrl = () => {
    if (entry.url) {
      openUrl(entry.url);
    }
  };

  const handleOpenFile = () => {
    if (entry.file) {
      openFile(entry.file);
    }
  };

  return (
    <div className="space-y-2">
      {/* Info Section */}
      <div className="collapse collapse-arrow bg-base-200/30 hover:bg-base-200/50 transition-colors rounded-box">
        <input type="checkbox" defaultChecked />
        <div className="collapse-title font-medium">Info</div>
        <div className="collapse-content">
          <table className="table table-sm">
            <tbody>
              <tr>
                <td className="text-right opacity-70 font-semibold">Type</td>
                <td>InProceedings</td>
              </tr>
              <tr>
                <td className="text-right opacity-70 font-semibold">Key</td>
                <td>{key}</td>
              </tr>
              <tr>
                <td className="text-right opacity-70 font-semibold">Title</td>
                <td>
                  {entry.title
                    ? (
                      <ChunksComp
                        chunks={entry.title}
                        citeKey={`InProceedingsDrawer-${key}`}
                      />
                    )
                    : ""}
                </td>
              </tr>
              {entry.author && entry.author.length > 0
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
              <tr>
                <td className="text-right">Book Title</td>
                <td>
                  {entry.book_title
                    ? (
                      <ChunksComp
                        chunks={entry.book_title}
                        citeKey={`booktitle-drawer-${key}`}
                      />
                    )
                    : ""}
                </td>
              </tr>
              <tr>
                <td className="text-right">Series</td>
                <td>{entry.series || ""}</td>
              </tr>
              {entry.editor && entry.editor.length > 0
                ? (
                  entry.editor.map(([editor, type_], idx) => (
                    <tr key={idx}>
                      <td className="text-right">{type_}</td>
                      <td>{editor}</td>
                    </tr>
                  ))
                )
                : (
                  <tr>
                    <td className="text-right">Editor</td>
                    <td></td>
                  </tr>
                )}
              {entry.publisher && entry.publisher.length > 0
                ? (
                  entry.publisher.map((publisher, idx) => (
                    <tr key={idx}>
                      <td className="text-right">Publisher</td>
                      <td>{publisher}</td>
                    </tr>
                  ))
                )
                : (
                  <tr>
                    <td className="text-right">Publisher</td>
                    <td></td>
                  </tr>
                )}
              {entry.organization && entry.organization.length > 0
                ? (
                  entry.organization.map((organization, idx) => (
                    <tr key={idx}>
                      <td className="text-right">Organization</td>
                      <td>{organization}</td>
                    </tr>
                  ))
                )
                : (
                  <tr>
                    <td className="text-right">Organization</td>
                    <td></td>
                  </tr>
                )}
              <tr>
                <td className="text-right">Address</td>
                <td>{entry.address || ""}</td>
              </tr>
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
                <td className="text-right">Month</td>
                <td>{entry.month || ""}</td>
              </tr>
              <tr>
                <td className="text-right">DOI</td>
                <td className="break-all">
                  {entry.doi
                    ? (
                      <button
                        type="button"
                        className="tooltip cursor-pointer text-left break-all"
                        data-tip="在浏览器中打开"
                        onClick={handleOpenDoi}
                      >
                        {entry.doi}
                      </button>
                    )
                    : ""}
                </td>
              </tr>
              <tr>
                <td className="text-right">URL</td>
                <td className="break-all">
                  {entry.url
                    ? (
                      <button
                        type="button"
                        className="tooltip cursor-pointer text-left break-all"
                        data-tip="在浏览器中打开"
                        onClick={handleOpenUrl}
                      >
                        {entry.url}
                      </button>
                    )
                    : ""}
                </td>
              </tr>
              <tr>
                <td className="text-right">File</td>
                <td>
                  {entry.file
                    ? (
                      <button
                        type="button"
                        className="tooltip cursor-pointer text-left break-all"
                        data-tip="打开"
                        onClick={handleOpenFile}
                      >
                        {entry.file}
                      </button>
                    )
                    : ""}
                </td>
              </tr>
            </tbody>
          </table>
        </div>
      </div>

      {/* Abstract Section */}
      <div className="collapse collapse-arrow bg-base-200/30 hover:bg-base-200/50 transition-colors rounded-box">
        <input type="checkbox" />
        <div className="collapse-title font-medium">Abstract</div>
        <div className="collapse-content">
          {entry.abstract_ && (
            <ChunksComp chunks={entry.abstract_} citeKey={`${key}-abstract`} />
          )}
        </div>
      </div>

      {/* Note Section */}
      <div className="collapse collapse-arrow bg-base-200/30 hover:bg-base-200/50 transition-colors rounded-box">
        <input type="checkbox" />
        <div className="collapse-title font-medium">Note</div>
        <div className="collapse-content">
          {entry.note && (
            <ChunksComp chunks={entry.note} citeKey={`${key}-note`} />
          )}
        </div>
      </div>

      {/* BibTeX Section */}
      <div className="collapse collapse-arrow bg-base-200/30 hover:bg-base-200/50 transition-colors rounded-box">
        <input type="checkbox" />
        <div className="collapse-title font-medium">BibTeX</div>
        <div className="collapse-content">
          {bibtex.map((line, idx) => (
            <p key={idx} className="font-mono text-xs">
              {line}
            </p>
          ))}
        </div>
      </div>
    </div>
  );
}

export default InProceedingsDrawer;
