import { Reference } from "../../types.ts";
import Article from "./Article";
import Book from "./Book";
import Booklet from "./Booklet";
import InBook from "./InBook";
import InCollection from "./InCollection";
import InProceedings from "./InProceedings";
import Thesis from "./Thesis";
import TechReport from "./TechReport";
import Misc from "./Misc";
import Unimplemented from "./Unimplemented";

interface ReferenceSelectorProps {
  entry: Reference;
}
function ReferenceSelector({ entry }: ReferenceSelectorProps) {
  switch (entry.type_) {
    case "Article":
      return <Article entry={entry} />;
    case "Book":
      return <Book entry={entry} />;
    case "Booklet":
      return <Booklet entry={entry} />;
    case "InBook":
      return <InBook entry={entry} />;
    case "InCollection":
      return <InCollection entry={entry} />;
    case "InProceedings":
      return <InProceedings entry={entry} />;
    case "MastersThesis":
    case "PhdThesis":
    case "Thesis":
      return <Thesis entry={entry} />;
    case "TechReport":
      return <TechReport entry={entry} />;
    case "Misc":
      return <Misc entry={entry} />;
    default:
      return <Unimplemented entry={entry} />;
  }
}

export default ReferenceSelector;
