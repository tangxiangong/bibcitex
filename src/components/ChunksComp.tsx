import React from "react";
import type { Chunk } from "../types.ts";
import InlineMath from "./InlineMath.tsx";

interface ChunksCompProps {
  chunks: Chunk[];
  citeKey: string;
}

function ChunksComp({ chunks = [], citeKey }: ChunksCompProps) {
  return (
    <>
      {chunks.map((chunk, i) => {
        const key = `${citeKey}-${i}`;

        if ("Normal" in chunk) {
          return <span key={key}>{chunk.Normal}</span>;
        } else if ("Verbatim" in chunk) {
          return <span key={key}>{chunk.Verbatim}</span>;
        } else if ("Math" in chunk) {
          return <InlineMath key={key} content={chunk.Math} />;
        }

        return null;
      })}
    </>
  );
}

export default ChunksComp;
