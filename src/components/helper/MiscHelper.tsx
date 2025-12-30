import { Reference } from "../../types.ts";
import ChunksComp from "../ChunksComp.tsx";

interface MiscHelperProps {
  entry: Reference;
}

function ArXivHelperComponent({ entry }: MiscHelperProps) {
  const key = entry.cite_key;

  const arxiv = entry.eprint
    ? entry.arxiv_primary_class
      ? `arXiv:${entry.eprint} [${entry.arxiv_primary_class}]`
      : `arXiv:${entry.eprint}`
    : entry.arxiv_primary_class
    ? `arXiv [${entry.arxiv_primary_class}]`
    : "arXiv";

  return (
    <div className="w-full">
      <div className="flex justify-between items-center">
        <div className="flex items-center gap-2">
          <div className="badge badge-error badge-soft badge-sm font-bold">
            Misc
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
                      className="badge badge-ghost badge-xs hover:badge-error transition-colors cursor-default"
                    >
                      {author}
                    </span>
                  ))}
                  <span className="badge badge-ghost badge-xs hover:badge-error transition-colors cursor-default">
                    et al.
                  </span>
                </>
              )
              : (
                entry.author.map((author, idx) => (
                  <span
                    key={idx}
                    className="badge badge-ghost badge-xs hover:badge-error transition-colors cursor-default"
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
        <span className="flex items-center gap-1 text-error">
          <span>📜 {arxiv}</span>
        </span>
        {entry.year && (
          <span className="flex items-center gap-1 text-secondary">
            <span>📅 {entry.year}</span>
          </span>
        )}
      </div>
    </div>
  );
}

function MiscHelper({ entry }: MiscHelperProps) {
  const key = entry.cite_key;
  const isArxiv = entry.archive_prefix === "arXiv";

  if (isArxiv) {
    return <ArXivHelperComponent entry={entry} />;
  }

  return (
    <div className="w-full">
      <div className="flex justify-between items-center">
        <div className="flex items-center gap-2">
          <div className="badge badge-neutral badge-soft badge-sm font-bold">
            Misc
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
                      className="badge badge-ghost badge-xs hover:badge-neutral transition-colors cursor-default"
                    >
                      {author}
                    </span>
                  ))}
                  <span className="badge badge-ghost badge-xs hover:badge-neutral transition-colors cursor-default">
                    et al.
                  </span>
                </>
              )
              : (
                entry.author.map((author, idx) => (
                  <span
                    key={idx}
                    className="badge badge-ghost badge-xs hover:badge-neutral transition-colors cursor-default"
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
        {entry.archive_prefix
          ? (
            <span className="flex items-center gap-1">
              <span>📦 {entry.archive_prefix}</span>
            </span>
          )
          : entry.how_published
          ? (
            <span className="flex items-center gap-1">
              <span>📢 {entry.how_published}</span>
            </span>
          )
          : null}
        {entry.year && (
          <span className="flex items-center gap-1 text-secondary">
            <span>📅 {entry.year}</span>
          </span>
        )}
      </div>
    </div>
  );
}

export default MiscHelper;
