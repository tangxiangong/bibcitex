import type { ImageAsset } from "nsh";
import { Image, token } from "nsh";
import { isWindows } from "./platform";
import addSvg from "../../.nsh/resources/icons/add.svg";
import alertSvg from "../../.nsh/resources/icons/alert.svg";
import bookSvg from "../../.nsh/resources/icons/book.svg";
import calendarSvg from "../../.nsh/resources/icons/calendar.svg";
import cancelSvg from "../../.nsh/resources/icons/cancel.svg";
import checkSvg from "../../.nsh/resources/icons/check.svg";
import checklistSvg from "../../.nsh/resources/icons/checklist.svg";
import chevronDownSvg from "../../.nsh/resources/icons/chevronDown.svg";
import chevronLeftSvg from "../../.nsh/resources/icons/chevronLeft.svg";
import chevronRightSvg from "../../.nsh/resources/icons/chevronRight.svg";
import clipboardSvg from "../../.nsh/resources/icons/clipboard.svg";
import copySvg from "../../.nsh/resources/icons/copy.svg";
import deleteSvg from "../../.nsh/resources/icons/delete.svg";
import detailsSvg from "../../.nsh/resources/icons/details.svg";
import downloadSvg from "../../.nsh/resources/icons/download.svg";
import errorSvg from "../../.nsh/resources/icons/error.svg";
import externalLinkSvg from "../../.nsh/resources/icons/externalLink.svg";
import fileTextSvg from "../../.nsh/resources/icons/fileText.svg";
import folderAddSvg from "../../.nsh/resources/icons/folderAdd.svg";
import folderOpenSvg from "../../.nsh/resources/icons/folderOpen.svg";
import infoSvg from "../../.nsh/resources/icons/info.svg";
import librarySvg from "../../.nsh/resources/icons/library.svg";
import linkSvg from "../../.nsh/resources/icons/link.svg";
import moonSvg from "../../.nsh/resources/icons/moon.svg";
import moreSvg from "../../.nsh/resources/icons/more.svg";
import okSvg from "../../.nsh/resources/icons/ok.svg";
import panelLeftCloseSvg from "../../.nsh/resources/icons/panelLeftClose.svg";
import panelLeftOpenSvg from "../../.nsh/resources/icons/panelLeftOpen.svg";
import panelRightCloseSvg from "../../.nsh/resources/icons/panelRightClose.svg";
import panelRightOpenSvg from "../../.nsh/resources/icons/panelRightOpen.svg";
import pinSvg from "../../.nsh/resources/icons/pin.svg";
import refreshSvg from "../../.nsh/resources/icons/refresh.svg";
import renameSvg from "../../.nsh/resources/icons/rename.svg";
import searchSvg from "../../.nsh/resources/icons/search.svg";
import settingsSvg from "../../.nsh/resources/icons/settings.svg";
import sunSvg from "../../.nsh/resources/icons/sun.svg";
import tagSvg from "../../.nsh/resources/icons/tag.svg";
import trashSvg from "../../.nsh/resources/icons/trash.svg";
import userSvg from "../../.nsh/resources/icons/user.svg";
import xSvg from "../../.nsh/resources/icons/x.svg";
import addWin from "../../.nsh/resources/icons/windows/add.svg";
import alertWin from "../../.nsh/resources/icons/windows/alert.svg";
import bookWin from "../../.nsh/resources/icons/windows/book.svg";
import calendarWin from "../../.nsh/resources/icons/windows/calendar.svg";
import checkWin from "../../.nsh/resources/icons/windows/check.svg";
import checklistWin from "../../.nsh/resources/icons/windows/checklist.svg";
import chevronDownWin from "../../.nsh/resources/icons/windows/chevronDown.svg";
import chevronLeftWin from "../../.nsh/resources/icons/windows/chevronLeft.svg";
import chevronRightWin from "../../.nsh/resources/icons/windows/chevronRight.svg";
import clipboardWin from "../../.nsh/resources/icons/windows/clipboard.svg";
import copyWin from "../../.nsh/resources/icons/windows/copy.svg";
import downloadWin from "../../.nsh/resources/icons/windows/download.svg";
import externalLinkWin from "../../.nsh/resources/icons/windows/externalLink.svg";
import fileTextWin from "../../.nsh/resources/icons/windows/fileText.svg";
import folderAddWin from "../../.nsh/resources/icons/windows/folderAdd.svg";
import folderOpenWin from "../../.nsh/resources/icons/windows/folderOpen.svg";
import infoWin from "../../.nsh/resources/icons/windows/info.svg";
import libraryWin from "../../.nsh/resources/icons/windows/library.svg";
import linkWin from "../../.nsh/resources/icons/windows/link.svg";
import moonWin from "../../.nsh/resources/icons/windows/moon.svg";
import moreWin from "../../.nsh/resources/icons/windows/more.svg";
import panelLeftCloseWin from "../../.nsh/resources/icons/windows/panelLeftClose.svg";
import panelLeftOpenWin from "../../.nsh/resources/icons/windows/panelLeftOpen.svg";
import panelRightCloseWin from "../../.nsh/resources/icons/windows/panelRightClose.svg";
import panelRightOpenWin from "../../.nsh/resources/icons/windows/panelRightOpen.svg";
import pinWin from "../../.nsh/resources/icons/windows/pin.svg";
import refreshWin from "../../.nsh/resources/icons/windows/refresh.svg";
import renameWin from "../../.nsh/resources/icons/windows/rename.svg";
import searchWin from "../../.nsh/resources/icons/windows/search.svg";
import settingsWin from "../../.nsh/resources/icons/windows/settings.svg";
import sunWin from "../../.nsh/resources/icons/windows/sun.svg";
import tagWin from "../../.nsh/resources/icons/windows/tag.svg";
import trashWin from "../../.nsh/resources/icons/windows/trash.svg";
import userWin from "../../.nsh/resources/icons/windows/user.svg";
import xWin from "../../.nsh/resources/icons/windows/x.svg";

/** The standalone SVG icons of `public/icons/`, synced into `.nsh/resources` by the build. */
const standard = {
  add: addSvg,
  alert: alertSvg,
  book: bookSvg,
  calendar: calendarSvg,
  cancel: cancelSvg,
  check: checkSvg,
  checklist: checklistSvg,
  chevronDown: chevronDownSvg,
  chevronLeft: chevronLeftSvg,
  chevronRight: chevronRightSvg,
  clipboard: clipboardSvg,
  copy: copySvg,
  delete: deleteSvg,
  details: detailsSvg,
  download: downloadSvg,
  error: errorSvg,
  externalLink: externalLinkSvg,
  fileText: fileTextSvg,
  folderAdd: folderAddSvg,
  folderOpen: folderOpenSvg,
  info: infoSvg,
  library: librarySvg,
  link: linkSvg,
  moon: moonSvg,
  more: moreSvg,
  ok: okSvg,
  panelLeftClose: panelLeftCloseSvg,
  panelLeftOpen: panelLeftOpenSvg,
  panelRightClose: panelRightCloseSvg,
  panelRightOpen: panelRightOpenSvg,
  pin: pinSvg,
  refresh: refreshSvg,
  rename: renameSvg,
  search: searchSvg,
  settings: settingsSvg,
  sun: sunSvg,
  tag: tagSvg,
  trash: trashSvg,
  user: userSvg,
  x: xSvg,
} as const;

/** Fluent variants from `public/icons/windows/`; others fall back to the standard set. */
const windows: Partial<Record<IconName, ImageAsset>> = {
  add: addWin,
  alert: alertWin,
  book: bookWin,
  calendar: calendarWin,
  check: checkWin,
  checklist: checklistWin,
  chevronDown: chevronDownWin,
  chevronLeft: chevronLeftWin,
  chevronRight: chevronRightWin,
  clipboard: clipboardWin,
  copy: copyWin,
  download: downloadWin,
  externalLink: externalLinkWin,
  fileText: fileTextWin,
  folderAdd: folderAddWin,
  folderOpen: folderOpenWin,
  info: infoWin,
  library: libraryWin,
  link: linkWin,
  moon: moonWin,
  more: moreWin,
  panelLeftClose: panelLeftCloseWin,
  panelLeftOpen: panelLeftOpenWin,
  panelRightClose: panelRightCloseWin,
  panelRightOpen: panelRightOpenWin,
  pin: pinWin,
  refresh: refreshWin,
  rename: renameWin,
  search: searchWin,
  settings: settingsWin,
  sun: sunWin,
  tag: tagWin,
  trash: trashWin,
  user: userWin,
  x: xWin,
};

export type IconName = keyof typeof standard;

/** The asset an icon draws on this platform. */
export function iconSource(name: IconName): ImageAsset {
  return (isWindows ? windows[name] : undefined) ?? standard[name];
}

export type IconTone = "primary" | "secondary" | "accent" | "success" | "danger" | "onPrimary";

const toneToken = {
  primary: "foreground",
  secondary: "muted-foreground",
  accent: "primary",
  success: "success",
  danger: "destructive",
  onPrimary: "primary-foreground",
} as const;

export interface IconProps {
  name: IconName;
  /** Points; the macOS client draws icons at 9–24pt by role (14 in rows, 16 in toolbars). */
  size?: number;
  tone?: IconTone;
  /** For an icon that stands alone; leave unset when a label beside it names it. */
  accessibilityLabel?: string;
}

/** A template SVG icon tinted with a semantic color (follows the appearance). */
export function Icon(props: IconProps) {
  return (
    <Image
      src={iconSource(props.name)}
      accessibilityLabel={props.accessibilityLabel}
      accessibilityElementsHidden={!props.accessibilityLabel}
      style={{
        width: props.size ?? 16,
        height: props.size ?? 16,
        tintColor: token(toneToken[props.tone ?? "primary"]),
      }}
    />
  );
}
