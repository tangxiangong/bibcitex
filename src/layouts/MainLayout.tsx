import React from "react";
import { Outlet } from "react-router-dom";
import Nav from "@components/Nav";
import Drawer from "@components/Drawer";

function MainLayout() {
  return (
    <div className="drawer drawer-end">
      <input id="my-drawer" type="checkbox" className="drawer-toggle" />
      <div className="drawer-content flex flex-col">
        <Nav />
        <main className="flex-1 overflow-hidden">
          <Outlet />
        </main>
      </div>
      <Drawer />
    </div>
  );
}

export default MainLayout;
