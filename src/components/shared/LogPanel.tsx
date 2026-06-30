import { useEffect, useRef } from "react";

interface LogPanelProps {
  logLines: string[];
}

/** Painel de log com auto-scroll para a última linha, igual ao legado. */
function LogPanel({ logLines }: LogPanelProps) {
  const containerRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const node = containerRef.current;
    if (node) {
      node.scrollTop = node.scrollHeight;
    }
  }, [logLines]);

  return (
    <div
      ref={containerRef}
      className="mt-5 min-h-0 flex-grow scroll-smooth overflow-y-auto rounded-lg border border-gray-700 bg-dark-blue-bg/50 p-4 text-sm"
    >
      <pre className="m-0 whitespace-pre-wrap break-words font-mono text-text-light">
        {logLines.join("\n")}
      </pre>
    </div>
  );
}

export default LogPanel;
