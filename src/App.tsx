import React from "react";
import { Route, Routes } from "react-router-dom";
import { AppProvider } from "./context/AppContext.tsx";
import MainLayout from "./layouts/MainLayout.tsx";
import HomePage from "./pages/HomePage.tsx";
import DetailPage from "./pages/DetailPage.tsx";
import HelperPage from "./pages/HelperPage.tsx";

function App() {
  return (
    <AppProvider>
      <Routes>
        <Route path="/" element={<MainLayout />}>
          <Route index element={<HomePage />} />
          <Route path="detail/:citeKey" element={<DetailPage />} />
        </Route>
        <Route path="/helper" element={<HelperPage />} />
      </Routes>
    </AppProvider>
  );
}

export default App;
