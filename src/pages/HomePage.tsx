import { createSignal } from "solid-js";
import Bibliographies from "@/components/Bibliographies.tsx";
import AddBibliography from "@/components/AddBibliography.tsx";

function HomePage() {
  const [showModal, setShowModal] = createSignal(false);

  return (
    <>
      <Bibliographies
        onOpenModal={() => setShowModal(true)}
      />
      <AddBibliography show={showModal()} onClose={() => setShowModal(false)} />
    </>
  );
}

export default HomePage;
