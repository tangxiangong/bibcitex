import React, { useEffect, useState } from "react";
import { Route, Routes } from "react-router-dom";
import { listen } from "@tauri-apps/api/event";
import { AppProvider } from "./context/AppContext.tsx";
import MainLayout from "./layouts/MainLayout.tsx";
import HomePage from "./pages/HomePage.tsx";
import DetailPage from "./pages/DetailPage.tsx";
import HelperPage from "./pages/HelperPage.tsx";
import { UpdaterModal } from "@/components/updater/index.ts";

function App() {
  const [showUpdaterModal, setShowUpdaterModal] = useState(false);

  useEffect(() => {
    // Global listener for update check menu item
    const unlisten = listen("check-update-trigger", () => {
      setShowUpdaterModal(true);
    });

    return () => {
      unlisten.then((f) => f());
    };
  }, []);

  return (
    <AppProvider>
      <Routes>
        <Route path="/" element={<MainLayout />}>
          <Route index element={<HomePage />} />
          <Route path="detail" element={<DetailPage />} />
          <Route path="detail/:citeKey" element={<DetailPage />} />
        </Route>
        <Route path="/helper" element={<HelperPage />} />
      </Routes>
      <UpdaterModal
        isOpen={showUpdaterModal}
        onClose={() => setShowUpdaterModal(false)}
      />
    </AppProvider>
  );
}

export default App;
