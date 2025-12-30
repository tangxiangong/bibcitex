import React, { useState } from "react";
import Bibliographies from "../lib/components/Bibliographies.tsx";
import AddBibliography from "../lib/components/AddBibliography.tsx";

function HomePage() {
  const [showModal, setShowModal] = useState(false);

  return (
    <>
      <Bibliographies onOpenModal={() => setShowModal(true)} />
      <AddBibliography show={showModal} onClose={() => setShowModal(false)} />
    </>
  );
}

export default HomePage;
