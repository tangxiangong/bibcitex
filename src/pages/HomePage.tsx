import React, { useState } from "react";
import Bibliographies from "@/components/Bibliographies.tsx";
import AddBibliography from "@/components/AddBibliography.tsx";
import { UpdaterModal } from "@/components/updater/index.ts";

function HomePage() {
  const [showModal, setShowModal] = useState(false);
  const [showUpdaterModal, setShowUpdaterModal] = useState(false);

  return (
    <>
      <Bibliographies
        onOpenModal={() => setShowModal(true)}
        onCheckUpdate={() => setShowUpdaterModal(true)}
      />
      <AddBibliography show={showModal} onClose={() => setShowModal(false)} />
      <UpdaterModal
        isOpen={showUpdaterModal}
        onClose={() => setShowUpdaterModal(false)}
      />
    </>
  );
}

export default HomePage;
