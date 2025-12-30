import type { Reference } from "../../types.ts";
import ArticleDrawer from "./ArticleDrawer.tsx";
import BookDrawer from "./BookDrawer.tsx";
import ThesisDrawer from "./ThesisDrawer.tsx";
import BookletDrawer from "./BookletDrawer.tsx";
import InBookDrawer from "./InBookDrawer.tsx";
import InCollectionDrawer from "./InCollectionDrawer.tsx";
import InProceedingsDrawer from "./InProceedingsDrawer.tsx";
import TechReportDrawer from "./TechReportDrawer.tsx";
import MiscDrawer from "./MiscDrawer.tsx";
import UnimplementedDrawer from "./UnimplementedDrawer.tsx";

interface ReferenceDrawerProps {
  entry: Reference;
}

function ReferenceDrawer({ entry }: ReferenceDrawerProps) {
  // Handle EntryType which can be a string or { Unknown: string }
  const type = typeof entry.type_ === "string"
    ? entry.type_.toLowerCase()
    : "unknown";

  switch (type) {
    case "article":
      return <ArticleDrawer entry={entry} />;
    case "book":
      return <BookDrawer entry={entry} />;
    case "thesis":
    case "mastersthesis":
    case "phdthesis":
      return <ThesisDrawer entry={entry} />;
    case "booklet":
      return <BookletDrawer entry={entry} />;
    case "inbook":
      return <InBookDrawer entry={entry} />;
    case "incollection":
      return <InCollectionDrawer entry={entry} />;
    case "inproceedings":
    case "conference":
      return <InProceedingsDrawer entry={entry} />;
    case "techreport":
      return <TechReportDrawer entry={entry} />;
    case "misc":
      return <MiscDrawer entry={entry} />;
    default:
      return <UnimplementedDrawer entry={entry} />;
  }
}

export default ReferenceDrawer;
