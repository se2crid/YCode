import { useEffect, useRef, useState } from "react";
import * as monaco from "monaco-editor";
import "./CodeEditor.css";
import { useColorScheme } from "@mui/joy/styles";
import { fs } from "@tauri-apps/api";

export interface CodeEditorProps {
  file: string;
  setUnsaved: (unsaved: boolean) => void;
  focused: boolean;
}
export default ({ file, setUnsaved, focused }: CodeEditorProps) => {
  const [editor, setEditor] =
    useState<monaco.editor.IStandaloneCodeEditor | null>(null);
  const monacoEl = useRef(null);
  const { mode } = useColorScheme();
  const [originalText, setOriginalText] = useState("");

  const saveFile = () => {
    if (editor) {
      const currentText = editor.getValue();
      fs.writeFile({ path: file, contents: currentText }).then(() => {
        setOriginalText(currentText);
        setUnsaved(false); // Assuming setUnsaved is a toggle function
      });
    }
  };

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
          value: "",
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
        setOriginalText(text);
      });
    }
  }, [file, editor]);

  useEffect(() => {
    if (editor && originalText) {
      const model = editor.getModel();
      if (model) {
        const listener = model.onDidChangeContent(() => {
          if (editor.getValue() !== originalText) {
            setUnsaved(true);
          }
        });

        return () => {
          listener.dispose();
        };
      }
    }
  }, [editor, setUnsaved, originalText]);

  return <div className={"code-editor"} ref={monacoEl}></div>;
};
