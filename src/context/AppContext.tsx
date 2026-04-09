import {
  createContext,
  createMemo,
  createSignal,
  type JSX,
  useContext,
} from "solid-js";
import type { BibliographyInfo, Reference, Setting } from "../types.ts";

interface AppContextType {
  settings: () => Setting;
  setSettings: (settings: Setting) => void;
  updateSettings: (settings: Setting) => void;

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

  const [currentReferences, setCurrentReferences] = createSignal<
    Reference[] | null
  >(null);
  const [drawerOpen, setDrawerOpen] = createSignal<boolean>(false);
  const [drawerReference, setDrawerReference] = createSignal<Reference | null>(
    null,
  );
  const [currentBibName, setCurrentBibName] = createSignal<string | null>(null);

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
