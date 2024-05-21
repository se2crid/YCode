import { path } from "@tauri-apps/api";
import CodeEditor from "../CodeEditor";
import "./Editor.css";
import { Tab, TabList, TabPanel, Tabs } from "@mui/joy";
import { useEffect, useState } from "react";
export interface EditorProps {
  openFiles: string[];
  focusedFile: string | null;
}

export default ({ openFiles, focusedFile }: EditorProps) => {
  const [tabNames, setTabNames] = useState<string[]>([]);
  const [unsavedFiles, setUnsavedFiles] = useState<string[]>([]);
  const [focused, setFocused] = useState<string | null>(null);

  useEffect(() => {
    if (focusedFile !== null) setFocused(focusedFile);
  }, [focusedFile]);

  useEffect(() => {
    const fetchTabNames = async () => {
      const names = await Promise.all(
        openFiles.map((file) => path.basename(file))
      );
      setTabNames(names);
    };

    fetchTabNames();
  }, [openFiles]);
  return (
    <div className={"editor"}>
      <Tabs
        sx={{ height: "100%", overflow: "hidden" }}
        className={"editor-tabs"}
        defaultValue={focusedFile !== null ? openFiles.indexOf(focusedFile) : 0}
        onChange={(_, newValue) => {
          if (newValue === null) return;
          setFocused(openFiles[newValue as number]);
        }}
      >
        <TabList>
          {tabNames.map((name, index) => (
            <Tab key={openFiles[index]}>
              {name}
              {unsavedFiles.indexOf(openFiles[index]) != -1 ? " •" : ""}
            </Tab>
          ))}
        </TabList>
        {openFiles.map((file, index) => (
          <TabPanel value={index} key={index} sx={{ padding: 0 }}>
            <CodeEditor
              focused={focused === file}
              key={file}
              file={file}
              setUnsaved={(unsaved: boolean) => {
                if (unsaved) setUnsavedFiles((unsaved) => [...unsaved, file]);
                else
                  setUnsavedFiles((unsaved) =>
                    unsaved.filter((unsavedFile) => unsavedFile !== file)
                  );
              }}
            />
          </TabPanel>
        ))}
      </Tabs>
    </div>
  );
};
