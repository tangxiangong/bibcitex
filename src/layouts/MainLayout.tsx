import type { RouteSectionProps } from "@solidjs/router";
import Nav from "@components/Nav";
import Drawer from "@components/Drawer";

function MainLayout(props: RouteSectionProps) {
  return (
    <div class="drawer drawer-end">
      <input id="my-drawer" type="checkbox" class="drawer-toggle" />
      <div class="drawer-content flex flex-col">
        <Nav />
        <main class="flex-1 overflow-hidden">
          {props.children}
        </main>
      </div>
      <Drawer />
    </div>
  );
}

export default MainLayout;
