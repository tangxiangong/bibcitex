import { Show, For } from "solid-js";
import { Reference } from "../../types.ts";
import ChunksComp from "../ChunksComp.tsx";

interface InProceedingsHelperProps {
  entry: Reference;
}

function InProceedingsHelper({ entry }: InProceedingsHelperProps) {
  const key = entry.cite_key;

  const date = entry.year
    ? entry.month ? `${entry.year}-${entry.month}` : entry.year.toString()
    : "";

  return (
    <div class="w-full">
      <div class="flex justify-between items-center">
        <div class="flex items-center gap-2">
          <div class="badge badge-purple badge-soft badge-sm font-bold">
            会议论文
          </div>
          <Show
            when={entry.title}
            fallback={
              <span class="text-gray-900 dark:text-gray-100 font-serif italic">
                暂无标题
              </span>
            }
          >
            <span class="text-gray-900 dark:text-gray-100 font-serif font-medium">
              <ChunksComp chunks={entry.title!} citeKey={key} />
            </span>
          </Show>
        </div>
        <div class="flex items-center shrink-0">
          <div class="text-xs font-mono opacity-50 ml-2">{key}</div>
        </div>
      </div>
      <div class="mt-1 flex flex-wrap gap-1">
        <Show
          when={entry.author && entry.author!.length > 0}
          fallback={
            <span class="text-xs text-base-content/50 italic">
              未知作者
            </span>
          }
        >
          <Show
            when={entry.author!.length > 3}
            fallback={
              <For each={entry.author!}>
                {(author, _idx) => (
                  <span class="badge badge-ghost badge-xs hover:badge-purple transition-colors cursor-default">
                    {author}
                  </span>
                )}
              </For>
            }
          >
            <For each={entry.author!.slice(0, 3)}>
              {(author, _idx) => (
                <span class="badge badge-ghost badge-xs hover:badge-purple transition-colors cursor-default">
                  {author}
                </span>
              )}
            </For>
            <span class="badge badge-ghost badge-xs hover:badge-purple transition-colors cursor-default">
              et al.
            </span>
          </Show>
        </Show>
      </div>
      <div class="mt-1 flex flex-wrap items-center gap-2 text-xs">
        <Show when={entry.book_title}>
          <span class="flex items-center gap-1">
            <span class="italic">
              会议论文集{" "}
              <ChunksComp
                chunks={entry.book_title!}
                citeKey={`booktitle_${key}`}
              />
            </span>
          </span>
        </Show>
        <Show when={date}>
          <span class="flex items-center gap-1 text-secondary">
            <span>年份 {date}</span>
          </span>
        </Show>
      </div>
    </div>
  );
}

export default InProceedingsHelper;
