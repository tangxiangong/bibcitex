import { Show, For } from "solid-js";
import { Reference } from "../../types.ts";
import ChunksComp from "../ChunksComp.tsx";

interface TechReportHelperProps {
  entry: Reference;
}

function TechReportHelper({ entry }: TechReportHelperProps) {
  const key = entry.cite_key;

  return (
    <div class="w-full">
      <div class="flex justify-between items-center">
        <div class="flex items-center">
          <div class="badge badge-outline mr-2 text-amber-800 dark:text-amber-200">
            技术报告
          </div>
          <Show
            when={entry.title}
            fallback={
              <span class="text-gray-900 dark:text-gray-100 font-serif">
                暂无标题
              </span>
            }
          >
            <span class="text-gray-900 dark:text-gray-100 font-serif">
              <ChunksComp chunks={entry.title!} citeKey={key} />
            </span>
          </Show>
        </div>
        <div class="flex items-center shrink-0">
          <div class="text-gray-600 dark:text-gray-400 text-xs font-mono ml-2">
            {key}
          </div>
        </div>
      </div>
      <p class="text-xs mt-2 break-all">
        <Show
          when={entry.author && entry.author!.length > 0}
          fallback={
            <span class="badge badge-outline text-blue-700 dark:text-blue-300 font-semibold mr-2">
              未知作者
            </span>
          }
        >
          <Show
            when={entry.author!.length > 3}
            fallback={
              <For each={entry.author!}>
                {(author, _idx) => (
                  <span class="badge badge-outline text-blue-700 dark:text-blue-300 font-semibold mr-2">
                    {author}
                    {" "}
                  </span>
                )}
              </For>
            }
          >
            <For each={entry.author!.slice(0, 3)}>
              {(author, _idx) => (
                <span class="badge badge-outline text-blue-700 dark:text-blue-300 font-semibold mr-2">
                  {author}
                  {" "}
                </span>
              )}
            </For>
            <span class="badge badge-outline text-blue-700 dark:text-blue-300 font-semibold mr-2">
              等
            </span>
          </Show>
        </Show>
      </p>
      <p class="text-xs mt-2 break-all">
        <Show
          when={entry.institution}
          fallback={
            <span class="badge badge-outline text-purple-600 dark:text-purple-300 mr-2">
              未知机构
            </span>
          }
        >
          <span class="badge badge-outline text-purple-600 dark:text-purple-300 mr-2">
            {entry.institution}
          </span>
        </Show>
        <Show
          when={entry.year}
          fallback={
            <span class="badge badge-outline text-emerald-700 dark:text-emerald-300 mr-2">
              年份
            </span>
          }
        >
          <span class="badge badge-outline text-emerald-700 dark:text-emerald-300 mr-2">
            {entry.year}
          </span>
        </Show>
      </p>
    </div>
  );
}

export default TechReportHelper;
