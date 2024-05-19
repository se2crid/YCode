import { useEffect, useRef, useState } from "react";
import * as monaco from "monaco-editor";
import "./CodeEditor.css";
import { useColorScheme } from "@mui/joy/styles";

export default () => {
  const [editor, setEditor] =
    useState<monaco.editor.IStandaloneCodeEditor | null>(null);
  const monacoEl = useRef(null);
  const { mode, setMode } = useColorScheme();

  useEffect(() => {
    if (!editor) {
      setEditor((editor) => {
        if (editor) return editor;

        return monaco.editor.create(monacoEl.current!, {
          value: ["func x() {", '\tconsole.log("Hello world!");', "}"].join(
            "\n"
          ),
          language: "swift",
          theme: "vs-" + mode,
          automaticLayout: true,
        });
      });
    } else {
      monaco.editor.setTheme("vs-" + mode);
    }

    const resizeObserver = new ResizeObserver(() => {
      editor?.layout();
    });

    resizeObserver.observe(monacoEl.current!);

    return () => {
      resizeObserver.disconnect();
    };
  }, [monacoEl.current, mode]);

  return <div className={"code-editor"} ref={monacoEl}></div>;
};
