import { Route, Router } from "@solidjs/router";
import { AppProvider } from "./context/AppContext.tsx";
import MainLayout from "./layouts/MainLayout.tsx";
import HomePage from "./pages/HomePage.tsx";
import DetailPage from "./pages/DetailPage.tsx";
import HelperPage from "./pages/HelperPage.tsx";

function App() {
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
    </AppProvider>
  );
}

export default App;
