import React, {
  createContext,
  ReactNode,
  useCallback,
  useContext,
  useState,
} from "react";
import type { BibliographyInfo, Reference, Setting } from "../types.ts";

interface AppContextType {
  settings: Setting;
  setSettings: (settings: Setting) => void;
  updateSettings: (settings: Setting) => void;

  currentReferences: Reference[] | null;
  setCurrentReferences: (refs: Reference[] | null) => void;

  drawerOpen: boolean;
  drawerReference: Reference | null;
  openDrawer: (reference: Reference) => void;
  closeDrawer: () => void;

  currentBibName: string | null;
  setCurrentBibName: (name: string | null) => void;

  bibliographyList: Array<{ name: string } & BibliographyInfo>;
}

const AppContext = createContext<AppContextType | undefined>(undefined);

export function AppProvider({ children }: { children: ReactNode }) {
  const [settings, setSettings] = useState<Setting>({
    bibliographies: {},
  });

  const [currentReferences, setCurrentReferences] = useState<
    Reference[] | null
  >(null);
  const [drawerOpen, setDrawerOpen] = useState<boolean>(false);
  const [drawerReference, setDrawerReference] = useState<Reference | null>(
    null,
  );
  const [currentBibName, setCurrentBibName] = useState<string | null>(null);

  const openDrawer = useCallback((reference: Reference) => {
    setDrawerReference(reference);
    setDrawerOpen(true);
  }, []);

  const closeDrawer = useCallback(() => {
    setDrawerOpen(false);
    setDrawerReference(null);
  }, []);

  const updateSettings = useCallback((newSettings: Setting) => {
    setSettings(newSettings);
  }, []);

  // Derived bibliography list
  const bibliographyList = Object.entries(settings.bibliographies).map((
    [name, info],
  ) => ({
    name,
    ...info,
  }));

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

  return <AppContext.Provider value={value}>{children}</AppContext.Provider>;
}

export function useApp() {
  const context = useContext(AppContext);
  if (context === undefined) {
    throw new Error("useApp must be used within an AppProvider");
  }
  return context;
}
