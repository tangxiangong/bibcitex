import { For, Show } from "solid-js";
import type { Reference } from "../../types.ts";
import ChunksComp from "../ChunksComp.tsx";
import { openFile, openUrl } from "../../tauri.ts";

interface ArticleDrawerProps {
  entry: Reference;
}

function ArticleDrawer({ entry }: ArticleDrawerProps) {
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
    <div class="space-y-2">
      {/* Info Section */}
      <div class="collapse collapse-arrow bg-base-200/30 hover:bg-base-200/50 transition-colors rounded-box">
        <input type="checkbox" checked />
        <div class="collapse-title font-medium">信息</div>
        <div class="collapse-content">
          <table class="table table-sm">
            <tbody>
              <tr>
                <td class="text-right opacity-70 font-semibold">类型</td>
                <td>期刊论文</td>
              </tr>
              <tr>
                <td class="text-right opacity-70 font-semibold">引用键</td>
                <td>{key}</td>
              </tr>
              <tr>
                <td class="text-right opacity-70 font-semibold">标题</td>
                <td>
                  <Show when={entry.title} fallback={""}>
                    <ChunksComp
                      chunks={entry.title!}
                      citeKey={`ArticleDrawer-${key}`}
                    />
                  </Show>
                </td>
              </tr>
              <Show
                when={entry.author && entry.author!.length > 0}
                fallback={
                  <tr>
                    <td class="text-right">作者</td>
                    <td></td>
                  </tr>
                }
              >
                <For each={entry.author!}>
                  {(author, idx) => (
                    <tr>
                      <td class="text-right">作者</td>
                      <td>{author}</td>
                    </tr>
                  )}
                </For>
              </Show>
              <Show
                when={entry.full_journal}
                fallback={
                  <tr>
                    <td class="text-right">期刊</td>
                    <td>{entry.journal || ""}</td>
                  </tr>
                }
              >
                <tr>
                  <td class="text-right">期刊</td>
                  <td>{entry.full_journal}</td>
                </tr>
                <tr>
                  <td class="text-right">期刊缩写</td>
                  <td>{entry.journal || ""}</td>
                </tr>
              </Show>
              <tr>
                <td class="text-right">卷</td>
                <td>{entry.volume || ""}</td>
              </tr>
              <tr>
                <td class="text-right">编号</td>
                <td>{entry.number || ""}</td>
              </tr>
              <tr>
                <td class="text-right">页码</td>
                <td>{pagesString}</td>
              </tr>
              <tr>
                <td class="text-right">年份</td>
                <td>{entry.year || ""}</td>
              </tr>
              <tr>
                <td class="text-right">DOI</td>
                <td class="break-all">
                  <Show when={entry.doi} fallback={""}>
                    <button
                      type="button"
                      class="tooltip cursor-pointer text-left break-all"
                      data-tip="在浏览器中打开"
                      onClick={handleOpenDoi}
                    >
                      {entry.doi}
                    </button>
                  </Show>
                </td>
              </tr>
              <tr>
                <td class="text-right">URL</td>
                <td class="break-all">
                  <Show when={entry.url} fallback={""}>
                    <button
                      type="button"
                      class="tooltip cursor-pointer text-left break-all"
                      data-tip="在浏览器中打开"
                      onClick={handleOpenUrl}
                    >
                      {entry.url}
                    </button>
                  </Show>
                </td>
              </tr>
              <tr>
                <td class="text-right">文件</td>
                <td>
                  <Show when={entry.file} fallback={""}>
                    <button
                      type="button"
                      class="tooltip cursor-pointer text-left break-all"
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
      <div class="collapse collapse-arrow bg-base-200/30 hover:bg-base-200/50 transition-colors rounded-box">
        <input type="checkbox" />
        <div class="collapse-title font-medium">摘要</div>
        <div class="collapse-content">
          <Show when={entry.abstract_}>
            <ChunksComp chunks={entry.abstract_!} citeKey={`${key}-abstract`} />
          </Show>
        </div>
      </div>

      {/* Note Section */}
      <div class="collapse collapse-arrow bg-base-200/30 hover:bg-base-200/50 transition-colors rounded-box">
        <input type="checkbox" />
        <div class="collapse-title font-medium">备注</div>
        <div class="collapse-content">
          <Show when={entry.note}>
            <ChunksComp chunks={entry.note!} citeKey={`${key}-note`} />
          </Show>
        </div>
      </div>

      {/* BibTeX Section */}
      <div class="collapse collapse-arrow bg-base-200/30 hover:bg-base-200/50 transition-colors rounded-box">
        <input type="checkbox" />
        <div class="collapse-title font-medium">BibTeX</div>
        <div class="collapse-content">
          <For each={bibtex}>
            {(line, idx) => (
              <p class="font-mono text-xs">
                {line}
              </p>
            )}
          </For>
        </div>
      </div>
    </div>
  );
}

export default ArticleDrawer;
