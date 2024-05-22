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
  const [tabs, setTabs] = useState<
    {
      name: string;
      file: string;
      index: number;
    }[]
  >([]);
  const [unsavedFiles, setUnsavedFiles] = useState<string[]>([]);
  const [focused, setFocused] = useState<number>();

  useEffect(() => {
    if (focusedFile !== null) setFocused(openFiles.indexOf(focusedFile));
  }, [focusedFile, openFiles]);

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
              focused={index === focused}
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
