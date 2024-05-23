import { path } from "@tauri-apps/api";
import CodeEditor, { CodeEditorHandles } from "../CodeEditor";
import "./Editor.css";
import { Tab, TabList, TabPanel, Tabs } from "@mui/joy";
import { useCallback, useEffect, useRef, useState } from "react";
export interface EditorProps {
  openFiles: string[];
  focusedFile: string | null;
  setSaveFile: (save: () => void) => void;
}

export default ({ openFiles, focusedFile, setSaveFile }: EditorProps) => {
  const [tabs, setTabs] = useState<
    {
      name: string;
      file: string;
      index: number;
    }[]
  >([]);
  const [unsavedFiles, setUnsavedFiles] = useState<string[]>([]);
  const [focused, setFocused] = useState<number>();
  const editors = useRef<(CodeEditorHandles | null)[]>([]);

  useEffect(() => {
    editors.current = editors.current.slice(0, openFiles.length);
  }, [openFiles]);

  useEffect(() => {
    if (focusedFile !== null) setFocused(openFiles.indexOf(focusedFile));
  }, [focusedFile, openFiles]);

  useEffect(() => {
    if (focused === undefined) return;
    let e = editors.current[focused];
    if (e === undefined || e === null) return;
    setSaveFile(() => e.saveFile);
  }, [focused, tabs]);

  useEffect(() => {
    const fetchTabNames = async () => {
      const names = await Promise.all(
        openFiles.map((file) => path.basename(file))
      );
      setTabs(
        names.map((name, index) => ({
          name,
          file: openFiles[index],
          index,
        }))
      );
    };

    fetchTabNames();
  }, [openFiles]);

  return (
    <div className={"editor"}>
      <Tabs
        sx={{ height: "100%", overflow: "hidden" }}
        className={"editor-tabs"}
        value={focused ?? 0}
        onChange={(_, newValue) => {
          if (newValue === null) return;
          setFocused(newValue as number);
        }}
      >
        <TabList>
          {tabs.map((tab, index) => (
            <Tab key={openFiles[index]}>
              {tab.name}
              {unsavedFiles.indexOf(openFiles[index]) != -1 ? " •" : ""}
            </Tab>
          ))}
        </TabList>
        {tabs.map((tab, index) => (
          <TabPanel value={index} key={index} sx={{ padding: 0 }}>
            <CodeEditor
              ref={(el) => (editors.current[index] = el)}
              key={tab.file}
              file={tab.file}
              setUnsaved={(unsaved: boolean) => {
                if (unsaved)
                  setUnsavedFiles((unsaved) => [...unsaved, tab.file]);
                else
                  setUnsavedFiles((unsaved) =>
                    unsaved.filter((unsavedFile) => unsavedFile !== tab.file)
                  );
              }}
            />
          </TabPanel>
        ))}
      </Tabs>
    </div>
  );
};
