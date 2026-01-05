import React, { useMemo, useState } from "react";
import { useApp } from "../context/AppContext.tsx";
import { searchByField, searchReferences } from "../tauri.ts";
import type { FilterField, FilterType, Reference } from "../types.ts";
import ReferenceSelector from "./reference/ReferenceSelector.tsx";

function References() {
  const { currentReferences } = useApp();
  const [query, setQuery] = useState("");
  const [isSearching, setIsSearching] = useState(false);
  const [searchResult, setSearchResult] = useState<Reference[]>([]);
  const [filterField, setFilterField] = useState<FilterField>("All");
  const [filterType, setFilterType] = useState<FilterType>("All");

  const allRefs = useMemo(() => currentReferences || [], [currentReferences]);
  const totalNum = useMemo(() => allRefs.length, [allRefs]);

  const filteredByType = useMemo(() => {
    if (filterType === "All") return allRefs;
    return allRefs.filter((ref) => {
      const type = typeof ref.type_ === "string" ? ref.type_ : "Unknown";
      switch (filterType) {
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
  }, [allRefs, filterType]);

  const displayRefs = useMemo(
    () => (isSearching ? searchResult : filteredByType),
    [isSearching, searchResult, filteredByType],
  );

  const showType = useMemo(() => {
    switch (filterType) {
      case "All":
        return "References";
      case "Article":
        return "Articles";
      case "Book":
        return "Books";
      case "Thesis":
        return "Thesis";
      case "TechReport":
        return "TechReports";
      case "Misc":
        return "Misc";
      case "Booklet":
        return "Booklets";
      case "InBook":
        return "InBooks";
      case "InCollection":
        return "InCollections";
      case "InProceedings":
        return "InProceedings";
      default:
        return "References";
    }
  }, [filterType]);

  const handleSearch = async (e: React.ChangeEvent<HTMLInputElement>) => {
    const value = e.target.value;
    setQuery(value);

    if (value.trim() === "") {
      setIsSearching(false);
      setSearchResult([]);
      return;
    }

    setIsSearching(true);
    try {
      if (filterField === "All") {
        const result = await searchReferences(filteredByType, value);
        setSearchResult(result);
      } else {
        const result = await searchByField(filteredByType, value, filterField);
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
    <div className="flex flex-col h-full overflow-hidden">
      <div className="shrink-0 p-4 bg-base-100 border-b border-base-300 overflow-hidden">
        <div className="join w-full max-w-full overflow-hidden">
          <select
            className="select select-bordered join-item w-24 sm:w-32 md:w-40"
            value={filterType}
            onChange={(e) => setFilterType(e.target.value as FilterType)}
          >
            {filterTypes.map((type) => (
              <option key={type} value={type}>
                {type === "All" ? "Type" : type}
              </option>
            ))}
          </select>

          <select
            className="select select-bordered join-item w-24 sm:w-32 md:w-40"
            value={filterField}
            onChange={(e) => setFilterField(e.target.value as FilterField)}
          >
            {filterFields.map((field) => (
              <option key={field} value={field}>
                {field === "All" ? "Field" : field}
              </option>
            ))}
          </select>

          <input
            type="search"
            className="input input-primary join-item flex-1 min-w-0"
            placeholder="搜索文献..."
            value={query}
            onChange={handleSearch}
          />
        </div>
      </div>

      <div className="flex-1 overflow-y-auto overflow-x-hidden">
        <h2 className="text-lg p-2">
          {showType} ({displayRefs.length}/{totalNum})
        </h2>

        {!isSearching
          ? (
            filteredByType.map((entry) => (
              <ReferenceSelector key={entry.cite_key} entry={entry} />
            ))
          )
          : (
            searchResult.length > 0
              ? (
                searchResult.map((entry) => (
                  <ReferenceSelector key={entry.cite_key} entry={entry} />
                ))
              )
              : <p className="p-2 text-lg text-red-500">No results</p>
          )}
      </div>
    </div>
  );
}

export default References;
