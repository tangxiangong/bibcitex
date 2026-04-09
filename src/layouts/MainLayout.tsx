import { lazy, Suspense } from "solid-js";
import type { RouteSectionProps } from "@solidjs/router";
import Nav from "@components/Nav";
import Drawer from "@components/Drawer";

const UpdateBanner = lazy(() => import("@/components/updater/UpdateBanner.tsx"));

function MainLayout(props: RouteSectionProps) {
  return (
    <div class="drawer drawer-end h-screen">
      <input id="my-drawer" type="checkbox" class="drawer-toggle" />
      <div class="drawer-content flex flex-col">
        <Nav />
        <Suspense>
          <UpdateBanner />
        </Suspense>
        <main class="flex-1 overflow-hidden">
          {props.children}
        </main>
      </div>
      <Drawer />
    </div>
  );
}

export default MainLayout;
