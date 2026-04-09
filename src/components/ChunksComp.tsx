import { For } from "solid-js";
import type { Chunk } from "../types.ts";
import InlineMath from "./InlineMath.tsx";

interface ChunksCompProps {
  chunks: Chunk[];
  citeKey: string;
}

function ChunksComp(props: ChunksCompProps) {
  return (
    <>
      <For each={props.chunks || []}>
        {(chunk) => {
          if ("Normal" in chunk) {
            return <span>{chunk.Normal}</span>;
          } else if ("Verbatim" in chunk) {
            return <span>{chunk.Verbatim}</span>;
          } else if ("Math" in chunk) {
            return <InlineMath content={chunk.Math} />;
          }
          return null;
        }}
      </For>
    </>
  );
}

export default ChunksComp;
