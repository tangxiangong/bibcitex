import React, { useEffect, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import Bibliographies from "@/components/Bibliographies.tsx";
import AddBibliography from "@/components/AddBibliography.tsx";
import { UpdaterModal } from "@/components/updater/index.ts";

function HomePage() {
  const [showModal, setShowModal] = useState(false);

  return (
    <>
      <Bibliographies
        onOpenModal={() => setShowModal(true)}
      />
      <AddBibliography show={showModal} onClose={() => setShowModal(false)} />
    </>
  );
}

export default HomePage;
