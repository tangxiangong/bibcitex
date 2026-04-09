import { createSignal, For, Show } from "solid-js";
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

interface InProceedingsProps {
  entry: Reference;
}

function InProceedings({ entry }: InProceedingsProps) {
  const { openDrawer } = useApp();
  const [copied, setCopied] = createSignal(false);
  const [copySuccess, setCopySuccess] = createSignal(true);

  const key = entry.cite_key;
  const doiUrl = entry.doi ? `https://doi.org/${entry.doi}` : "";

  const date = (() => {
    if (entry.year) {
      if (entry.month) {
        return `${entry.year}-${entry.month}`;
      } else {
        return entry.year.toString();
      }
    } else {
      return "";
    }
  })();

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
    <div class="card-modern card-shine group hover:-translate-y-1 transition-all duration-300 m-4 border-l-4 border-l-purple-500">
      <div class="card-body p-5">
        {/* Header: Type + Title + Actions */}
        <div class="flex justify-between items-start gap-4">
          <div class="flex-1">
            <div class="flex items-center gap-2 mb-2">
              <span class="badge badge-purple badge-soft badge-sm font-bold">
                InProceedings
              </span>
              <span class="text-xs font-mono opacity-50 select-all">
                {key}
              </span>
            </div>
            <Show
              when={entry.title}
              fallback={
                <span class="text-lg text-base-content/50 italic">
                  No title available
                </span>
              }
            >
              <h3 class="text-xl font-bold leading-snug gradient-text">
                <ChunksComp chunks={entry.title} citeKey={key} />
              </h3>
            </Show>
          </div>

          {/* Actions */}
          <div class="flex items-center gap-1 opacity-0 group-hover:opacity-100 transition-opacity duration-200">
            <button
              type="button"
              class="btn btn-ghost btn-sm btn-circle tooltip tooltip-left"
              data-tip="Copy Key"
              onClick={handleCopyKey}
            >
              <Show
                when={!copied()}
                fallback={
                  <Show
                    when={copySuccess()}
                    fallback={
                      <img
                        width={18}
                        src={ERROR_ICON}
                        alt="Error"
                        class="text-error"
                      />
                    }
                  >
                    <img
                      width={18}
                      src={OK_ICON}
                      alt="Success"
                      class="text-success"
                    />
                  </Show>
                }
              >
                <img
                  width={18}
                  src={COPY_ICON}
                  alt="Copy"
                  class="opacity-70"
                />
              </Show>
            </button>
            <button
              type="button"
              class="btn btn-ghost btn-sm btn-circle tooltip tooltip-left"
              data-tip="Details"
              onClick={handleOpenDrawer}
            >
              <img
                width={18}
                src={DETAILS_ICON}
                alt="Details"
                class="opacity-70"
              />
            </button>
          </div>
        </div>

        {/* Authors */}
        <div class="mt-3 flex flex-wrap gap-2">
          <Show
            when={entry.author && entry.author.length > 0}
            fallback={
              <span class="text-sm text-base-content/50 italic">
                Unknown Author
              </span>
            }
          >
            <For each={entry.author}>
              {(author) => (
                <span class="badge badge-ghost hover:badge-purple transition-colors cursor-default bg-base-200/50">
                  {author}
                </span>
              )}
            </For>
          </Show>
        </div>

        {/* Metadata Row */}
        <div class="mt-4 flex flex-wrap items-center gap-4 text-sm text-base-content/70 border-t border-base-content/5 pt-3">
          <Show when={entry.book_title}>
            <div class="flex items-center gap-1">
              <span class="font-semibold text-primary">📚</span>
              <span class="italic">
                <ChunksComp
                  chunks={entry.book_title}
                  citeKey={`booktitle_${key}`}
                />
              </span>
            </div>
          </Show>
          <Show when={date}>
            <div class="flex items-center gap-1">
              <span class="font-semibold text-secondary">📅</span>
              <span>{date}</span>
            </div>
          </Show>

          {/* Links */}
          <div class="flex-1"></div>

          <Show when={entry.doi}>
            <button
              type="button"
              class="btn btn-xs btn-ghost gap-1 hover:text-purple"
              onClick={handleOpenDoi}
            >
              DOI
            </button>
          </Show>
          <Show when={entry.url}>
            <button
              type="button"
              class="btn btn-xs btn-ghost gap-1 hover:text-purple"
              onClick={handleOpenUrl}
            >
              URL
            </button>
          </Show>
          <Show when={entry.file}>
            <button
              type="button"
              class="btn btn-xs btn-purple btn-soft gap-1"
              onClick={handleOpenFile}
            >
              PDF
            </button>
          </Show>
        </div>
      </div>
    </div>
  );
}

export default InProceedings;
