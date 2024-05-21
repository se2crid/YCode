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
      >
        <TabList>
          {tabNames.map((name, index) => (
            <Tab key={openFiles[index]}>{name}</Tab>
          ))}
        </TabList>
        {openFiles.map((file, index) => (
          <TabPanel value={index} key={index} sx={{ padding: 0 }}>
            <CodeEditor key={file} file={file} />
          </TabPanel>
        ))}
      </Tabs>
    </div>
  );
};
