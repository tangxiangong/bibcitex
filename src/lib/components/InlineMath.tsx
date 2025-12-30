import React, { useMemo } from "react";
import katex from "katex";

interface InlineMathProps {
  content: string;
}

function InlineMath({ content }: InlineMathProps) {
  const html = useMemo(() => {
    try {
      return katex.renderToString(content, {
        output: "mathml",
        throwOnError: false,
      });
    } catch {
      return content;
    }
  }, [content]);

  return (
    <span
      className="inline-math"
      dangerouslySetInnerHTML={{ __html: html }}
    />
  );
}

export default InlineMath;
