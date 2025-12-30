import { Reference } from "../../types.ts";
import ChunksComp from "../ChunksComp.tsx";

interface ThesisHelperProps {
  entry: Reference;
}

function ThesisHelper({ entry }: ThesisHelperProps) {
  const key = entry.cite_key;

  const schoolAddress = entry.school
    ? entry.address ? `${entry.school} (${entry.address})` : entry.school
    : "";

  const thesisType = entry.type_ === "PhdThesis"
    ? "PhD Thesis"
    : entry.type_ === "MastersThesis"
    ? "Master Thesis"
    : "Thesis";

  const badgeClass = entry.type_ === "MastersThesis"
    ? "badge badge-outline mr-2 text-pink-800 dark:text-pink-200"
    : "badge badge-outline mr-2 text-rose-800 dark:text-rose-200";

  return (
    <div className="w-full">
      <div className="flex justify-between items-center">
        <div className="flex items-center">
          <div className={badgeClass}>
            {thesisType}
          </div>
          {entry.title
            ? (
              <span className="text-gray-900 dark:text-gray-100 font-serif">
                <ChunksComp chunks={entry.title} citeKey={key} />
              </span>
            )
            : (
              <span className="text-gray-900 dark:text-gray-100 font-serif">
                No title available
              </span>
            )}
        </div>
        <div className="flex items-center shrink-0">
          <div className="text-gray-600 dark:text-gray-400 text-xs font-mono ml-2">
            {key}
          </div>
        </div>
      </div>
      <p className="text-xs mt-2 break-all">
        {entry.author && entry.author.length > 0
          ? (
            entry.author.length > 3
              ? (
                <>
                  {entry.author.slice(0, 3).map((author, idx) => (
                    <span
                      key={idx}
                      className="badge badge-outline text-blue-700 dark:text-blue-300 font-semibold mr-2"
                    >
                      {author}
                    </span>
                  ))}
                  <span className="badge badge-outline text-blue-700 dark:text-blue-300 font-semibold mr-2">
                    et al.
                  </span>
                </>
              )
              : (
                entry.author.map((author, idx) => (
                  <span
                    key={idx}
                    className="badge badge-outline text-blue-700 dark:text-blue-300 font-semibold mr-2"
                  >
                    {author}
                  </span>
                ))
              )
          )
          : (
            <span className="badge badge-outline text-blue-700 dark:text-blue-300 font-semibold mr-2">
              Unknown
            </span>
          )}
      </p>
      <p className="text-xs mt-2 break-all">
        {schoolAddress
          ? (
            <span className="badge badge-outline text-purple-600 dark:text-purple-300 mr-2">
              {schoolAddress}
            </span>
          )
          : (
            <span className="badge badge-outline text-purple-600 dark:text-purple-300 mr-2">
              Unknown
            </span>
          )}
        {entry.year
          ? (
            <span className="badge badge-outline text-emerald-700 dark:text-emerald-300 mr-2">
              {entry.year}
            </span>
          )
          : (
            <span className="badge badge-outline text-emerald-700 dark:text-emerald-300 mr-2">
              year
            </span>
          )}
      </p>
    </div>
  );
}

export default ThesisHelper;
