import { useEffect, useRef, useState } from "react";
import "./Console.css";
import { listen } from "@tauri-apps/api/event";
import Convert from "ansi-to-html";

const convert = new Convert();

export default function Console() {
  const [body, setBody] = useState("");
  const [html, setHtml] = useState("");
  const preRef = useRef<HTMLPreElement | null>(null);
  const listenerAdded = useRef(false);

  useEffect(() => {
    if (!listenerAdded.current) {
      listen("build-output", (event) => {
        let line = event.payload as string;
        console.log(line, line.includes("command.done"));
        if (line.includes("command.done")) {
          setBody(
            (body) =>
              body +
              "Command finished with exit code: " +
              line.split(".")[2] +
              "\n" +
              "<hr/>"
          );
        } else {
          setBody((body) => body + line + "\n");
        }
      });
      listenerAdded.current = true;
    }
  }, []);

  useEffect(() => {
    if (body.startsWith("\n")) {
      setHtml(convert.toHtml(body.slice(1)));
    } else {
      setHtml(convert.toHtml(body));
    }
  }, [body]);

  useEffect(() => {
    if (preRef.current) {
      const element = preRef.current;
      setTimeout(() => {
        element.scrollIntoView({ behavior: "smooth", block: "end" });
      }, 0);
    }
  }, [html]);

  return (
    <div className="console-tile">
      <pre ref={preRef} dangerouslySetInnerHTML={{ __html: html }}></pre>
    </div>
  );
}
