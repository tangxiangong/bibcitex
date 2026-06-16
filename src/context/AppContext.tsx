import {
  type Accessor,
  createContext,
  createEffect,
  createMemo,
  createSignal,
  type JSX,
  type Setter,
  useContext,
} from "solid-js";
import type { BibliographyInfo, Reference, Setting } from "../types.ts";
import { setNativeHelperTheme } from "@/tauri.ts";

const isMacOS = () => {
  if (typeof navigator === "undefined") return false;

  return /Mac/i.test(navigator.platform) || /Macintosh|Mac OS X/i.test(navigator.userAgent);
};

export type AppTheme = "latte" | "mocha";

interface AppContextType {
  settings: () => Setting;
  setSettings: (settings: Setting) => void;
  updateSettings: (settings: Setting) => void;

  theme: Accessor<AppTheme>;
  setTheme: Setter<AppTheme>;
  toggleTheme: () => void;

  currentReferences: () => Reference[] | null;
  setCurrentReferences: (refs: Reference[] | null) => void;
  selectedReference: () => Reference | null;
  setSelectedReference: (reference: Reference | null) => void;
  selectBibliography: (name: string, refs: Reference[]) => void;

  drawerOpen: () => boolean;
  drawerReference: () => Reference | null;
  openDrawer: (reference: Reference) => void;
  closeDrawer: () => void;

  currentBibName: () => string | null;
  setCurrentBibName: (name: string | null) => void;
  leftPaneOpen: () => boolean;
  setLeftPaneOpen: Setter<boolean>;
  toggleLeftPane: () => void;
  rightPaneOpen: () => boolean;
  setRightPaneOpen: Setter<boolean>;
  toggleRightPane: () => void;

  bibliographyList: () => Array<{ name: string } & BibliographyInfo>;
}

const AppContext = createContext<AppContextType | undefined>(undefined);

export function AppProvider(props: { children: JSX.Element }) {
  const [settings, setSettings] = createSignal<Setting>({
    bibliographies: {},
  });
  const getInitialTheme = (): AppTheme => {
    if (typeof window === "undefined") return "latte";
    return window.matchMedia("(prefers-color-scheme: dark)").matches
      ? "mocha"
      : "latte";
  };
  const [theme, setTheme] = createSignal<AppTheme>(getInitialTheme());

  const [currentReferences, setCurrentReferencesState] = createSignal<
    Reference[] | null
  >(null);
  const [selectedReference, setSelectedReference] =
    createSignal<Reference | null>(null);
  const [drawerOpen, setDrawerOpen] = createSignal<boolean>(false);
  const [drawerReference, setDrawerReference] = createSignal<Reference | null>(
    null,
  );
  const [currentBibName, setCurrentBibName] = createSignal<string | null>(null);
  const [leftPaneOpen, setLeftPaneOpen] = createSignal(true);
  const [rightPaneOpen, setRightPaneOpen] = createSignal(true);

  createEffect(() => {
    const nextTheme = theme();
    document.documentElement.setAttribute("data-theme", nextTheme);
    if (!isMacOS()) return;

    setNativeHelperTheme(nextTheme).catch((error) => {
      console.error("Failed to sync native helper theme:", error);
    });
  });

  const toggleTheme = () => {
    setTheme((current) => (current === "latte" ? "mocha" : "latte"));
  };

  const setCurrentReferences = (refs: Reference[] | null) => {
    setCurrentReferencesState(refs);
    setSelectedReference(refs?.[0] ?? null);
  };

  const selectBibliography = (name: string, refs: Reference[]) => {
    setCurrentBibName(name);
    setCurrentReferences(refs);
  };

  const toggleLeftPane = () => {
    setLeftPaneOpen((current) => !current);
  };

  const toggleRightPane = () => {
    setRightPaneOpen((current) => !current);
  };

  const openDrawer = (reference: Reference) => {
    setSelectedReference(reference);
    setDrawerReference(reference);
    setDrawerOpen(true);
  };

  const closeDrawer = () => {
    setDrawerOpen(false);
    setDrawerReference(null);
  };

  const updateSettings = (newSettings: Setting) => {
    setSettings(newSettings);
  };

  const bibliographyList = createMemo(() =>
    Object.entries(settings().bibliographies).map(
      ([name, info]) => ({
        name,
        ...info,
      }),
    )
  );

  const value: AppContextType = {
    settings,
    setSettings,
    updateSettings,
    theme,
    setTheme,
    toggleTheme,
    currentReferences,
    setCurrentReferences,
    selectedReference,
    setSelectedReference,
    selectBibliography,
    drawerOpen,
    drawerReference,
    openDrawer,
    closeDrawer,
    currentBibName,
    setCurrentBibName,
    leftPaneOpen,
    setLeftPaneOpen,
    toggleLeftPane,
    rightPaneOpen,
    setRightPaneOpen,
    toggleRightPane,
    bibliographyList,
  };

  return <AppContext.Provider value={value}>{props.children}</AppContext.Provider>;
}

export function useApp() {
  const context = useContext(AppContext);
  if (context === undefined) {
    throw new Error("useApp must be used within an AppProvider");
  }
  return context;
}
