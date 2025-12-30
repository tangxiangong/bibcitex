import { Reference } from "../../types.ts";
import ArticleHelper from "./ArticleHelper";
import BookHelper from "./BookHelper";
import BookletHelper from "./BookletHelper";
import InBookHelper from "./InBookHelper";
import InCollectionHelper from "./InCollectionHelper";
import InProceedingsHelper from "./InProceedingsHelper";
import ThesisHelper from "./ThesisHelper";
import TechReportHelper from "./TechReportHelper";
import MiscHelper from "./MiscHelper";
import UnimplementedHelper from "./UnimplementedHelper";

interface HelperSelectorProps {
  entry: Reference;
}

function HelperSelector({ entry }: HelperSelectorProps) {
  switch (entry.type_) {
    case "Article":
      return <ArticleHelper entry={entry} />;
    case "Book":
      return <BookHelper entry={entry} />;
    case "Booklet":
      return <BookletHelper entry={entry} />;
    case "InBook":
      return <InBookHelper entry={entry} />;
    case "InCollection":
      return <InCollectionHelper entry={entry} />;
    case "InProceedings":
      return <InProceedingsHelper entry={entry} />;
    case "MastersThesis":
    case "PhdThesis":
    case "Thesis":
      return <ThesisHelper entry={entry} />;
    case "TechReport":
      return <TechReportHelper entry={entry} />;
    case "Misc":
      return <MiscHelper entry={entry} />;
    default:
      return <UnimplementedHelper entry={entry} />;
  }
}

export default HelperSelector;
