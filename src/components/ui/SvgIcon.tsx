import { splitProps } from "solid-js";
import { Dynamic } from "solid-js/web";
import {
  Book,
  Calendar,
  Check,
  ChevronLeft,
  ChevronRight,
  CircleAlert,
  Clipboard,
  Copy,
  Download,
  ExternalLink,
  FileText,
  FolderOpen,
  Info,
  Library,
  Link,
  Moon,
  PanelLeftClose,
  PanelLeftOpen,
  PanelRightClose,
  PanelRightOpen,
  RefreshCw,
  Search,
  Settings,
  Sun,
  Tag,
  Trash,
  User,
  X,
  type LucideIcon,
  type LucideProps,
} from "lucide-solid";

export const ICONS = {
  alert: CircleAlert,
  book: Book,
  calendar: Calendar,
  check: Check,
  chevronLeft: ChevronLeft,
  chevronRight: ChevronRight,
  clipboard: Clipboard,
  copy: Copy,
  download: Download,
  externalLink: ExternalLink,
  fileText: FileText,
  folderOpen: FolderOpen,
  info: Info,
  library: Library,
  link: Link,
  moon: Moon,
  panelLeftClose: PanelLeftClose,
  panelLeftOpen: PanelLeftOpen,
  panelRightClose: PanelRightClose,
  panelRightOpen: PanelRightOpen,
  refresh: RefreshCw,
  search: Search,
  settings: Settings,
  sun: Sun,
  tag: Tag,
  trash: Trash,
  user: User,
  x: X,
} as const satisfies Record<string, LucideIcon>;

export type SvgIconName = keyof typeof ICONS;

export interface SvgIconProps extends LucideProps {
  name: SvgIconName;
  title?: string;
}

export function SvgIcon(props: SvgIconProps) {
  const [local, iconProps] = splitProps(props, [
    "name",
    "title",
    "class",
    "size",
    "strokeWidth",
    "aria-label",
    "aria-hidden",
    "role",
  ]);
  const Icon = () => ICONS[local.name];
  const ariaHidden = () =>
    local["aria-hidden"] ?? (local.title || local["aria-label"] ? undefined : true);

  return (
    <Dynamic
      component={Icon()}
      {...iconProps}
      class={`inline-block shrink-0 ${local.class ?? ""}`.trim()}
      size={local.size ?? 18}
      strokeWidth={local.strokeWidth ?? 2}
      aria-label={local["aria-label"] ?? local.title}
      aria-hidden={ariaHidden()}
      role={local.role ?? (local.title || local["aria-label"] ? "img" : undefined)}
    >
      {local.title ? <title>{local.title}</title> : undefined}
    </Dynamic>
  );
}

export default SvgIcon;
