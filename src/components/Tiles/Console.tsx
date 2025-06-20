import { useEffect, useRef, useState } from "react";
import "./Console.css";
import { listen } from "@tauri-apps/api/event";
import Convert from "ansi-to-html";
import { Virtuoso } from "react-virtuoso";

const convert = new Convert();

function escapeHtml(unsafe: string) {
  return unsafe
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;")
    .replace(/'/g, "&#039;");
}

export default function Console() {
  const [lines, setLines] = useState<string[]>([]);
  const listenerAdded = useRef(false);
  const unlisten = useRef<() => void>(() => {});
  const clearOnNext = useRef(false);

  useEffect(() => {
    if (!listenerAdded.current) {
      (async () => {
        const unlistenFn = await listen("build-output", (event) => {
          let line = event.payload as string;
          if (clearOnNext.current) {
            setLines([line]);
            clearOnNext.current = false;
            return;
          }
          if (line.includes("command.done")) {
            if (line.split(".")[2] === "999") {
              setLines((lines) => [...lines, "Command finished"]);
            } else {
              setLines((lines) => [
                ...lines,
                "Command finished with exit code: " + line.split(".")[2],
              ]);
            }
            clearOnNext.current = true;
          } else {
            setLines((lines) => [...lines, line]);
          }
        });
        unlisten.current = unlistenFn;
      })();
      listenerAdded.current = true;
    }
    return () => {
      unlisten.current();
    };
  }, []);

  return (
    <Virtuoso
      className="console-tile"
      atBottomThreshold={6}
      followOutput={"auto"}
      data={lines}
      itemContent={(_, line) => (
        <pre
          style={{ margin: 0 }}
          dangerouslySetInnerHTML={{ __html: convert.toHtml(escapeHtml(line)) }}
        />
      )}
    />
  );
}
