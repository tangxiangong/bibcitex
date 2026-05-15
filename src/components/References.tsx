import { createMemo, createSignal, For, Show } from "solid-js";
import { useApp } from "../context/AppContext.tsx";
import { searchByField, searchReferences } from "../tauri.ts";
import type { FilterField, FilterType, Reference } from "../types.ts";
import ReferenceSelector from "./reference/ReferenceSelector.tsx";

function References() {
  const { currentReferences } = useApp();
  const [query, setQuery] = createSignal("");
  const [isSearching, setIsSearching] = createSignal(false);
  const [searchResult, setSearchResult] = createSignal<Reference[]>([]);
  const [filterField, setFilterField] = createSignal<FilterField>("All");
  const [filterType, setFilterType] = createSignal<FilterType>("All");

  const allRefs = createMemo(() => currentReferences() || []);
  const totalNum = createMemo(() => allRefs().length);

  const filteredByType = createMemo(() => {
    if (filterType() === "All") return allRefs();
    return allRefs().filter((ref) => {
      const type = typeof ref.type_ === "string" ? ref.type_ : "Unknown";
      switch (filterType()) {
        case "Book":
          return type === "Book";
        case "Article":
          return type === "Article";
        case "Thesis":
          return type === "Thesis" || type === "MastersThesis" ||
            type === "PhdThesis";
        case "TechReport":
          return type === "TechReport";
        case "Misc":
          return type === "Misc";
        case "Booklet":
          return type === "Booklet";
        case "InBook":
          return type === "InBook";
        case "InCollection":
          return type === "InCollection";
        case "InProceedings":
          return type === "InProceedings";
        default:
          return true;
      }
    });
  });

  const displayRefs = createMemo(() =>
    isSearching() ? searchResult() : filteredByType()
  );

  const showType = createMemo(() => {
    switch (filterType()) {
      case "All":
        return "全部文献";
      case "Article":
        return "期刊论文";
      case "Book":
        return "图书";
      case "Thesis":
        return "学位论文";
      case "TechReport":
        return "技术报告";
      case "Misc":
        return "其他";
      case "Booklet":
        return "小册子";
      case "InBook":
        return "书籍章节";
      case "InCollection":
        return "文集章节";
      case "InProceedings":
        return "会议论文";
      default:
        return "全部文献";
    }
  });

  const handleSearch = async (e: Event & { currentTarget: HTMLInputElement }) => {
    const value = e.currentTarget.value;
    setQuery(value);

    if (value.trim() === "") {
      setIsSearching(false);
      setSearchResult([]);
      return;
    }

    setIsSearching(true);
    try {
      if (filterField() === "All") {
        const result = await searchReferences(filteredByType(), value);
        setSearchResult(result);
      } else {
        const result = await searchByField(filteredByType(), value, filterField());
        setSearchResult(result);
      }
    } catch (e) {
      console.error("Search failed:", e);
      setSearchResult([]);
    }
  };

  const filterTypes: FilterType[] = [
    "All",
    "Article",
    "Book",
    "Thesis",
    "TechReport",
    "Misc",
    "Booklet",
    "InBook",
    "InCollection",
    "InProceedings",
  ];
  const filterFields: FilterField[] = [
    "All",
    "Author",
    "Title",
    "Journal",
    "Year",
  ];

  return (
    <div class="flex flex-col h-full overflow-hidden">
      <div class="shrink-0 p-4 bg-base-100 border-b border-base-300 overflow-hidden">
        <div class="join w-full max-w-full overflow-hidden">
          <select
            class="select select-bordered join-item w-24 sm:w-32 md:w-40"
            value={filterType()}
            onChange={(e) => setFilterType(e.currentTarget.value as FilterType)}
          >
            <For each={filterTypes}>
              {(type) => (
                <option value={type}>
                  {{
                    All: "类型",
                    Article: "期刊论文",
                    Book: "图书",
                    Thesis: "学位论文",
                    TechReport: "技术报告",
                    Misc: "其他",
                    Booklet: "小册子",
                    InBook: "书籍章节",
                    InCollection: "文集章节",
                    InProceedings: "会议论文",
                  }[type]}
                </option>
              )}
            </For>
          </select>

          <select
            class="select select-bordered join-item w-24 sm:w-32 md:w-40"
            value={filterField()}
            onChange={(e) => setFilterField(e.currentTarget.value as FilterField)}
          >
            <For each={filterFields}>
              {(field) => (
                <option value={field}>
                  {{
                    All: "字段",
                    Author: "作者",
                    Title: "标题",
                    Journal: "期刊",
                    Year: "年份",
                  }[field]}
                </option>
              )}
            </For>
          </select>

          <input
            type="search"
            class="input input-primary join-item flex-1 min-w-0"
            placeholder="搜索文献..."
            value={query()}
            onInput={handleSearch}
          />
        </div>
      </div>

      <div class="flex-1 overflow-y-auto overflow-x-hidden">
        <h2 class="text-lg p-2">
          {showType()} ({displayRefs().length}/{totalNum()})
        </h2>

        <Show
          when={!isSearching()}
          fallback={
            <Show
              when={searchResult().length > 0}
              fallback={<p class="p-2 text-lg text-red-500">没有结果</p>}
            >
              <For each={searchResult()}>
                {(entry) => <ReferenceSelector entry={entry} />}
              </For>
            </Show>
          }
        >
          <For each={filteredByType()}>
            {(entry) => <ReferenceSelector entry={entry} />}
          </For>
        </Show>
      </div>
    </div>
  );
}

export default References;
