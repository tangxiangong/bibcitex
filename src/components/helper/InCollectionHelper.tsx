import { Show, For } from "solid-js";
import { Reference } from "../../types.ts";
import ChunksComp from "../ChunksComp.tsx";

interface InCollectionHelperProps {
  entry: Reference;
}

function InCollectionHelper({ entry }: InCollectionHelperProps) {
  const key = entry.cite_key;

  return (
    <div className="w-full">
      <div className="flex justify-between items-center">
        <div className="flex items-center gap-2">
          <div className="badge badge-secondary badge-soft badge-sm font-bold">
            InCollection
          </div>
          <Show
            when={entry.title}
            fallback={
              <span className="text-gray-900 dark:text-gray-100 font-serif italic">
                No title available
              </span>
            }
          >
            <span className="text-gray-900 dark:text-gray-100 font-serif font-medium">
              <ChunksComp chunks={entry.title} citeKey={key} />
            </span>
          </Show>
        </div>
        <div className="flex items-center shrink-0">
          <div className="text-xs font-mono opacity-50 ml-2">{key}</div>
        </div>
      </div>
      <div className="mt-1 flex flex-wrap gap-1">
        <Show
          when={entry.author && entry.author.length > 0}
          fallback={
            <span className="text-xs text-base-content/50 italic">
              Unknown Author
            </span>
          }
        >
          <Show
            when={entry.author.length > 3}
            fallback={
              <For each={entry.author}>
                {(author, _idx) => (
                  <span className="badge badge-ghost badge-xs hover:badge-secondary transition-colors cursor-default">
                    {author}
                  </span>
                )}
              </For>
            }
          >
            <For each={entry.author.slice(0, 3)}>
              {(author, _idx) => (
                <span className="badge badge-ghost badge-xs hover:badge-secondary transition-colors cursor-default">
                  {author}
                </span>
              )}
            </For>
            <span className="badge badge-ghost badge-xs hover:badge-secondary transition-colors cursor-default">
              et al.
            </span>
          </Show>
        </Show>
      </div>
      <div className="mt-1 flex flex-wrap items-center gap-2 text-xs">
        <Show when={entry.book_title}>
          <span className="flex items-center gap-1">
            <span className="italic">
              📘{" "}
              <ChunksComp
                chunks={entry.book_title}
                citeKey={`InCollection-BT-${key}`}
              />
            </span>
          </span>
        </Show>
        <Show when={entry.publisher && entry.publisher.length > 0}>
          <For each={entry.publisher}>
            {(publisher, _idx) => (
              <span className="flex items-center gap-1">
                <span>🏢 {publisher}</span>
              </span>
            )}
          </For>
        </Show>
        <Show when={entry.year}>
          <span className="flex items-center gap-1 text-secondary">
            <span>📅 {entry.year}</span>
          </span>
        </Show>
      </div>
    </div>
  );
}

export default InCollectionHelper;
