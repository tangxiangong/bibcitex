import { createMemo } from "solid-js";
import katex from "katex";

interface InlineMathProps {
  content: string;
}

function InlineMath(props: InlineMathProps) {
  const html = createMemo(() => {
    try {
      return katex.renderToString(props.content, {
        output: "mathml",
        throwOnError: false,
      });
    } catch {
      return props.content;
    }
  });

  return (
    <span
      class="inline-math"
      innerHTML={html()}
    />
  );
}

export default InlineMath;
