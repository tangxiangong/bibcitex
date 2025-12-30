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
          {entry.title
            ? (
              <span className="text-gray-900 dark:text-gray-100 font-serif font-medium">
                <ChunksComp chunks={entry.title} citeKey={key} />
              </span>
            )
            : (
              <span className="text-gray-900 dark:text-gray-100 font-serif italic">
                No title available
              </span>
            )}
        </div>
        <div className="flex items-center shrink-0">
          <div className="text-xs font-mono opacity-50 ml-2">{key}</div>
        </div>
      </div>
      <div className="mt-1 flex flex-wrap gap-1">
        {entry.author && entry.author.length > 0
          ? (
            entry.author.length > 3
              ? (
                <>
                  {entry.author.slice(0, 3).map((author, idx) => (
                    <span
                      key={idx}
                      className="badge badge-ghost badge-xs hover:badge-secondary transition-colors cursor-default"
                    >
                      {author}
                    </span>
                  ))}
                  <span className="badge badge-ghost badge-xs hover:badge-secondary transition-colors cursor-default">
                    et al.
                  </span>
                </>
              )
              : (
                entry.author.map((author, idx) => (
                  <span
                    key={idx}
                    className="badge badge-ghost badge-xs hover:badge-secondary transition-colors cursor-default"
                  >
                    {author}
                  </span>
                ))
              )
          )
          : (
            <span className="text-xs text-base-content/50 italic">
              Unknown Author
            </span>
          )}
      </div>
      <div className="mt-1 flex flex-wrap items-center gap-2 text-xs">
        {entry.book_title && (
          <span className="flex items-center gap-1">
            <span className="italic">
              📘{" "}
              <ChunksComp
                chunks={entry.book_title}
                citeKey={`InCollection-BT-${key}`}
              />
            </span>
          </span>
        )}
        {entry.publisher && entry.publisher.length > 0 && (
          entry.publisher.map((publisher, idx) => (
            <span key={idx} className="flex items-center gap-1">
              <span>🏢 {publisher}</span>
            </span>
          ))
        )}
        {entry.year && (
          <span className="flex items-center gap-1 text-secondary">
            <span>📅 {entry.year}</span>
          </span>
        )}
      </div>
    </div>
  );
}

export default InCollectionHelper;
