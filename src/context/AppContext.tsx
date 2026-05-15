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

  drawerOpen: () => boolean;
  drawerReference: () => Reference | null;
  openDrawer: (reference: Reference) => void;
  closeDrawer: () => void;

  currentBibName: () => string | null;
  setCurrentBibName: (name: string | null) => void;

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

  const [currentReferences, setCurrentReferences] = createSignal<
    Reference[] | null
  >(null);
  const [drawerOpen, setDrawerOpen] = createSignal<boolean>(false);
  const [drawerReference, setDrawerReference] = createSignal<Reference | null>(
    null,
  );
  const [currentBibName, setCurrentBibName] = createSignal<string | null>(null);

  createEffect(() => {
    document.documentElement.setAttribute("data-theme", theme());
  });

  const toggleTheme = () => {
    setTheme((current) => (current === "latte" ? "mocha" : "latte"));
  };

  const openDrawer = (reference: Reference) => {
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
    drawerOpen,
    drawerReference,
    openDrawer,
    closeDrawer,
    currentBibName,
    setCurrentBibName,
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
