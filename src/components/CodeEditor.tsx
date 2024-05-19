import { useEffect, useRef, useState } from "react";
import * as monaco from "monaco-editor";
import "./CodeEditor.css";

export default () => {
  const [editor, setEditor] =
    useState<monaco.editor.IStandaloneCodeEditor | null>(null);
  const monacoEl = useRef(null);

  useEffect(() => {
    if (monacoEl) {
      setEditor((editor) => {
        if (editor) return editor;

        return monaco.editor.create(monacoEl.current!, {
          value: ["func x() {", '\tconsole.log("Hello world!");', "}"].join(
            "\n"
          ),
          language: "swift",
          theme: "vs-dark",
        });
      });
    }

    // Add a resize observer to the container of the Monaco Editor
    const resizeObserver = new ResizeObserver(() => {
      editor?.layout();
    });

    resizeObserver.observe(monacoEl.current!);

    return () => {
      editor?.dispose();
      resizeObserver.disconnect();
    };
  }, [monacoEl.current]);

  return <div className={"code-editor"} ref={monacoEl}></div>;
};
