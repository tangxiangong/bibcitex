import { createSignal, onCleanup } from "solid-js";
import { Route, Router } from "@solidjs/router";
import { listen } from "@tauri-apps/api/event";
import { AppProvider } from "./context/AppContext.tsx";
import MainLayout from "./layouts/MainLayout.tsx";
import HomePage from "./pages/HomePage.tsx";
import DetailPage from "./pages/DetailPage.tsx";
import HelperPage from "./pages/HelperPage.tsx";
import { UpdaterModal } from "@/components/updater/index.ts";

function App() {
  const [showUpdaterModal, setShowUpdaterModal] = createSignal(false);

  const unlisten = listen("check-update-trigger", () => {
    setShowUpdaterModal(true);
  });

  onCleanup(() => {
    unlisten.then((f) => f());
  });

  return (
    <AppProvider>
      <Router>
        <Route path="/" component={MainLayout}>
          <Route path="/" component={HomePage} />
          <Route path="/detail" component={DetailPage} />
          <Route path="/detail/:citeKey" component={DetailPage} />
        </Route>
        <Route path="/helper" component={HelperPage} />
      </Router>
      <UpdaterModal
        isOpen={showUpdaterModal()}
        onClose={() => setShowUpdaterModal(false)}
      />
    </AppProvider>
  );
}

export default App;
