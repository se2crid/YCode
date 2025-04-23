import {
  forwardRef,
  useCallback,
  useEffect,
  useImperativeHandle,
  useRef,
  useState,
} from "react";
import * as monaco from "monaco-editor";
import "./CodeEditor.css";
import { useColorScheme } from "@mui/joy/styles";
import { path } from "@tauri-apps/api";
import * as fs from "@tauri-apps/plugin-fs";

export interface CodeEditorProps {
  file: string;
  setUnsaved: (unsaved: boolean) => void;
}
export interface CodeEditorHandles {
  file: string;
  saveFile: () => void;
}

const getLanguage = async (filename: string) => {
  if (filename === "Makefile") {
    return "make";
  }
  const ext = await path.extname(filename);
  const extToLang: { [key: string]: string } = {
    js: "javascript",
    ts: "typescript",
    py: "python",
    rb: "ruby",
    go: "go",
    rs: "rust",
    swift: "swift",
    json: "json",
    // objc priority, this is an ios editor after all (sorry c)
    m: "objective-c",
    mi: "objective-c",
    h: "objective-c",
    xm: "objective-c",
    xmi: "objective-c",
    sh: "shell",
  };
  return extToLang[ext] || "plaintext";
};

const CodeEditor = forwardRef<CodeEditorHandles, CodeEditorProps>(
  ({ file, setUnsaved }, ref) => {
    const [editor, setEditor] =
      useState<monaco.editor.IStandaloneCodeEditor | null>(null);
    const monacoEl = useRef(null);
    const { mode } = useColorScheme();
    const [originalText, setOriginalText] = useState("");
    const [failedReason, setFailedReason] = useState<string | null>(null);

    const editorRef = useRef<monaco.editor.IStandaloneCodeEditor | null>(null);

    useEffect(() => {
      editorRef.current = editor;
    }, [editor]);

    const saveFile = useCallback(() => {
      if (editorRef.current) {
        const currentText = editorRef.current.getValue();
        setUnsaved(false);
        fs.writeTextFile(file, currentText).then(() => {
          setOriginalText(currentText);
          setUnsaved(false);
        });
      }
    }, [file, setUnsaved]);

    // Exposes parameters to the ref on the parent component
    useImperativeHandle(ref, () => ({
      saveFile,
      file,
    }));

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
            language: "plaintext",
            theme: "vs-" + colorScheme,
            automaticLayout: true,
          });
        });
      } else {
        monaco.editor.setTheme("vs-" + colorScheme);
      }

      if (monacoEl.current) {
        const resizeObserver = new ResizeObserver(() => {
          editor?.layout();
        });

        resizeObserver.observe(monacoEl.current);

        return () => {
          resizeObserver.disconnect();
        };
      }
    }, [monacoEl.current, mode]);

    useEffect(() => {
      if (editor) {
        fs.readTextFile(file)
          .then((text) => {
            editor.setValue(text);
            setOriginalText(text);
            // set the language
            getLanguage(file).then((lang) => {
              let model = editor.getModel();
              if (model === null) return;
              monaco.editor.setModelLanguage(model, lang);
            });
          })
          .catch((error) => {
            let err = error.toString();
            if (err.includes("did not contain valid UTF-8")) {
              err = "Unable to decode file as UTF-8";
            }
            setFailedReason(err);
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

    if (failedReason !== null) {
      return <div className={"editor-failed"}>{failedReason}</div>;
    }
    return <div className={"code-editor"} ref={monacoEl}></div>;
  }
);

export default CodeEditor;
