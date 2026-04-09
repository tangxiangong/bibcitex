import { For, Show } from "solid-js";
import type { Reference } from "../../types.ts";
import ChunksComp from "../ChunksComp.tsx";
import { openFile, openUrl } from "../../tauri.ts";

interface UnimplementedDrawerProps {
  entry: Reference;
}

function UnimplementedDrawer({ entry }: UnimplementedDrawerProps) {
  const key = entry.cite_key;
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
        <input type="checkbox" checked />
        <div className="collapse-title font-medium">Info</div>
        <div className="collapse-content">
          <table className="table table-sm">
            <tbody>
              <tr>
                <td className="text-right opacity-70 font-semibold">Type</td>
                <td>
                  {typeof entry.type_ === "string"
                    ? entry.type_
                    : entry.type_.Unknown}
                </td>
              </tr>
              <tr>
                <td className="text-right opacity-70 font-semibold">Key</td>
                <td>{key}</td>
              </tr>
              <tr>
                <td className="text-right opacity-70 font-semibold">Title</td>
                <td>
                  <Show when={entry.title} fallback={""}>
                    <ChunksComp
                      chunks={entry.title}
                      citeKey={`UnimplementedDrawer-${key}`}
                    />
                  </Show>
                </td>
              </tr>
              <Show
                when={entry.author && entry.author.length > 0}
                fallback={
                  <tr>
                    <td className="text-right">Author</td>
                    <td></td>
                  </tr>
                }
              >
                <For each={entry.author}>
                  {(author, idx) => (
                    <tr>
                      <td className="text-right">Author</td>
                      <td>{author}</td>
                    </tr>
                  )}
                </For>
              </Show>
              <tr>
                <td className="text-right">Year</td>
                <td>{entry.year || ""}</td>
              </tr>
              <tr>
                <td className="text-right">DOI</td>
                <td className="break-all">
                  <Show when={entry.doi} fallback={""}>
                    <button
                      type="button"
                      className="tooltip cursor-pointer text-left break-all"
                      data-tip="在浏览器中打开"
                      onClick={handleOpenDoi}
                    >
                      {entry.doi}
                    </button>
                  </Show>
                </td>
              </tr>
              <tr>
                <td className="text-right">URL</td>
                <td className="break-all">
                  <Show when={entry.url} fallback={""}>
                    <button
                      type="button"
                      className="tooltip cursor-pointer text-left break-all"
                      data-tip="在浏览器中打开"
                      onClick={handleOpenUrl}
                    >
                      {entry.url}
                    </button>
                  </Show>
                </td>
              </tr>
              <tr>
                <td className="text-right">File</td>
                <td>
                  <Show when={entry.file} fallback={""}>
                    <button
                      type="button"
                      className="tooltip cursor-pointer text-left break-all"
                      data-tip="打开"
                      onClick={handleOpenFile}
                    >
                      {entry.file}
                    </button>
                  </Show>
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
          <Show when={entry.abstract_}>
            <ChunksComp chunks={entry.abstract_} citeKey={`${key}-abstract`} />
          </Show>
        </div>
      </div>

      {/* Note Section */}
      <div className="collapse collapse-arrow bg-base-200/30 hover:bg-base-200/50 transition-colors rounded-box">
        <input type="checkbox" />
        <div className="collapse-title font-medium">Note</div>
        <div className="collapse-content">
          <Show when={entry.note}>
            <ChunksComp chunks={entry.note} citeKey={`${key}-note`} />
          </Show>
        </div>
      </div>

      {/* BibTeX Section */}
      <div className="collapse collapse-arrow bg-base-200/30 hover:bg-base-200/50 transition-colors rounded-box">
        <input type="checkbox" />
        <div className="collapse-title font-medium">BibTeX</div>
        <div className="collapse-content">
          <For each={bibtex}>
            {(line, idx) => (
              <p className="font-mono text-xs">
                {line}
              </p>
            )}
          </For>
        </div>
      </div>
    </div>
  );
}

export default UnimplementedDrawer;
