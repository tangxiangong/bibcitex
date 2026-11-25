import { effect, mount, View } from "nsh";
import { displays, Menu, Window, type Display, type WindowRef } from "nsh/desktop";
import { installLanguage, tr } from "./src/i18n";
import { helper as helperMetrics, settings as settingsMetrics, tray as trayMetrics } from "./src/theme";
import { desktop, desktopCapabilities, desktopPlatform, prefs, preferencesPending, startDesktop, trayShown } from "./src/store/desktop";
import { createHelper } from "./src/store/helper";
import {
  aboutVisible,
  bindHelper,
  checkForUpdates,
  helperVisible,
  hideHelper,
  hideTray,
  installDesktopIntegrations,
  reopenHelper,
  setAboutOpen,
  setSettingsOpen,
  settingsVisible,
  shortcutRegistered,
  trayPanelAnchor,
  trayVisible,
  windowOpened,
} from "./src/store/surfaces";
import { startUpdater, updatesAvailable } from "./src/store/updater";
import { createWorkbench } from "./src/store/workbench";
import { HelperPanel } from "./src/screens/Helper";
import { AboutPanel, SettingsPanel } from "./src/screens/Settings";
import { TrayPanel } from "./src/screens/Tray";
import { Workbench } from "./src/screens/Workbench";
import { isMac, windowMaterial } from "./src/ui";

/** The display the floating panels open on: the main window's, else the primary one. */
function activeDisplay(): Display | undefined {
  return Window.get("main")?.state()?.display ?? displays().find((d) => d.primary) ?? displays()[0];
}

function helperWidth(display: Display | undefined): number {
  const visible = display?.visibleBounds.size.width ?? helperMetrics.maxWidth + helperMetrics.screenMargin;
  return Math.min(helperMetrics.maxWidth, visible - helperMetrics.screenMargin);
}

function App() {
  installLanguage(() => desktop.systemLocale);
  startUpdater(() => preferencesPending() ? null : prefs());
  const mainWorkbench = createWorkbench("main");
  const trayWorkbench = createWorkbench("tray");
  const helper = createHelper(hideHelper, reopenHelper);
  bindHelper(helper);
  installDesktopIntegrations();
  void startDesktop();

  // Closing the main window hides it only while something can bring it
  // back for certain: a tray icon that is actually shown, or a helper
  // shortcut this process registered. A platform that merely could have a
  // shortcut (Linux's portal, a refused key) does not count: it minimizes.
  const closeMain = () => {
    const main = Window.get("main");
    if (!main) return;
    const trayReachable = (prefs()?.trayIconEnabled ?? false) && (desktopCapabilities()?.tray ?? false) && trayShown();
    if (trayReachable || shortcutRegistered()) main.hide();
    else main.minimize();
  };

  // macOS keeps its application menu; Windows and Linux show none in the window.
  effect(() => {
    if (!isMac) return;
    Menu.setApplicationMenu([
      {
        label: "BibCiTeX",
        submenu: [
          { label: tr("关于 BibCiTeX"), onSelect: () => setAboutOpen(true) },
          { type: "separator" },
          { label: tr("menu.system.Settings…"), shortcut: "mod+,", onSelect: () => setSettingsOpen(true) },
          { label: tr("检查更新"), disabled: !updatesAvailable(), onSelect: checkForUpdates },
          { type: "separator" },
          { role: "hide", label: tr("menu.hide") },
          { role: "hideOthers", label: tr("menu.hideOthers") },
          { role: "showAll", label: tr("menu.showAll") },
          { type: "separator" },
          { role: "quit", label: tr("退出 BibCiTeX") },
        ],
      },
      {
        label: tr("menu.edit"),
        submenu: [
          { role: "undo", label: tr("menu.system.Undo") },
          { role: "redo", label: tr("menu.system.Redo") },
          { type: "separator" },
          { role: "cut", label: tr("menu.cut") },
          { role: "copy", label: tr("menu.copy") },
          { role: "paste", label: tr("menu.paste") },
          { role: "selectAll", label: tr("menu.selectAll") },
        ],
      },
      {
        label: tr("menu.window"),
        submenu: [
          { role: "minimize", label: tr("menu.minimize") },
          { role: "zoom", label: tr("menu.zoom") },
          { role: "close", label: tr("menu.close") },
        ],
      },
    ]);
  });

  const placeHelper = (win: WindowRef) => {
    const display = activeDisplay();
    if (!display) return;
    const area = display.visibleBounds;
    const width = helperWidth(display);
    win.setBounds({
      origin: { x: area.origin.x + (area.size.width - width) / 2, y: area.origin.y + area.size.height * 0.2 },
      size: { width },
    });
    windowOpened("helper", win);
  };

  // Beside the tray icon when the system says where it is: below a top
  // menu bar, above a bottom taskbar. Otherwise the corner the tray is in.
  const placeTray = (win: WindowRef) => {
    windowOpened("tray", win);
    const display = activeDisplay();
    if (!display) return;
    const area = display.visibleBounds;
    const anchor = trayPanelAnchor();
    const maxX = area.origin.x + area.size.width - trayMetrics.width - trayMetrics.screenGap;
    if (anchor) {
      const below = anchor.top < area.origin.y + area.size.height / 2;
      const x = Math.min(Math.max(anchor.x - trayMetrics.width / 2, area.origin.x + trayMetrics.screenGap), maxX);
      const y = below ? anchor.bottom + trayMetrics.screenGap : anchor.top - trayMetrics.height - trayMetrics.screenGap;
      win.setBounds({ origin: { x, y } });
      return;
    }
    const top = desktopPlatform()?.os !== "windows";
    win.setBounds({
      origin: {
        x: maxX,
        y: top ? area.origin.y + trayMetrics.screenGap : area.origin.y + area.size.height - trayMetrics.height - trayMetrics.screenGap,
      },
    });
  };

  return (
    <>
      <Window
        id="main"
        title="BibCiTeX"
        defaultSize={{ width: 1200, height: 800 }}
        minSize={{ width: 560, height: 520 }}
        backdrop="window"
        titleBar={{ style: "hidden", height: 52 }}
        closeOnPress={false}
        onClosePress={closeMain}
        ref={(win: WindowRef) => windowOpened("main", win)}
      >
        <View class="flex-1 bg-background">
          <Workbench wb={mainWorkbench} />
        </View>
      </Window>
      <Window
        id="helper"
        kind="panel"
        title="BibCiTeX Helper"
        open={helperVisible()}
        onOpenChange={({ open }) => {
          if (!open) hideHelper();
        }}
        {...windowMaterial("floating")}
        level="floating"
        workspaces="all"
        taskbar={false}
        resizable={false}
        minimizable={false}
        maximizable={false}
        closeOnEscape={false}
        closeOnBlur
        sizeToContent="height"
        defaultSize={{ width: helperMetrics.maxWidth, height: helperMetrics.header }}
        ref={placeHelper}
      >
        <HelperPanel helper={helper} />
      </Window>
      <Window
        id="tray"
        kind="panel"
        title="BibCiTeX Tray"
        open={trayVisible()}
        onOpenChange={({ open }) => {
          if (!open) hideTray();
        }}
        {...windowMaterial("floating")}
        level="floating"
        taskbar={false}
        resizable={false}
        minimizable={false}
        maximizable={false}
        closeOnEscape
        closeOnBlur
        defaultSize={{ width: trayMetrics.width, height: trayMetrics.height }}
        ref={placeTray}
      >
        <TrayPanel wb={trayWorkbench} />
      </Window>
      <Window
        id="settings"
        kind="normal"
        title={tr("设置")}
        open={settingsVisible()}
        onOpenChange={({ open }) => {
          if (!open) setSettingsOpen(false);
        }}
        {...windowMaterial("dialog")}
        resizable={false}
        minimizable={false}
        maximizable={false}
        closeOnEscape
        defaultSize={{ width: settingsMetrics.width, height: settingsMetrics.height }}
        defaultPosition="center"
        ref={(win: WindowRef) => windowOpened("settings", win)}
      >
        <SettingsPanel />
      </Window>
      <Window
        id="about"
        kind="dialog"
        title={tr("关于 BibCiTeX")}
        open={aboutVisible()}
        onOpenChange={({ open }) => {
          if (!open) setAboutOpen(false);
        }}
        {...windowMaterial("dialog")}
        resizable={false}
        minimizable={false}
        maximizable={false}
        closeOnEscape
        defaultSize={{ width: 300, height: 220 }}
        defaultPosition="center"
        ref={(win: WindowRef) => windowOpened("about", win)}
      >
        <AboutPanel />
      </Window>
    </>
  );
}

mount(App);
