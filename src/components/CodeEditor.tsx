import { useEffect, useRef, useState } from "react";
import * as monaco from "monaco-editor";
import "./CodeEditor.css";
import { useColorScheme } from "@mui/joy/styles";
import { fs } from "@tauri-apps/api";

export interface CodeEditorProps {
  file: string;
}
export default ({ file }: CodeEditorProps) => {
  const [editor, setEditor] =
    useState<monaco.editor.IStandaloneCodeEditor | null>(null);
  const monacoEl = useRef(null);
  const { mode } = useColorScheme();

  useEffect(() => {
    let colorScheme = mode;
    if (colorScheme === "system") {
      colorScheme = window.matchMedia("(prefers-color-scheme: dark)").matches
        ? "dark"
        : "light";
    }
    if (!editor) {
      setEditor((editor) => {
        if (editor) return editor;

        return monaco.editor.create(monacoEl.current!, {
          value: ["func x() {", '\tconsole.log("Hello world!");', "}"].join(
            "\n"
          ),
          language: "swift",
          theme: "vs-" + colorScheme,
          automaticLayout: true,
        });
      });
    } else {
      monaco.editor.setTheme("vs-" + colorScheme);
    }

    const resizeObserver = new ResizeObserver(() => {
      editor?.layout();
    });

    resizeObserver.observe(monacoEl.current!);

    return () => {
      resizeObserver.disconnect();
    };
  }, [monacoEl.current, mode]);

  useEffect(() => {
    if (editor) {
      fs.readTextFile(file).then((text) => {
        editor.setValue(text);
      });
    }
  }, [file, editor]);

  return <div className={"code-editor"} ref={monacoEl}></div>;
};
