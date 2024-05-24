import Splitter, { SplitDirection } from "@devbookhq/splitter";
import Tile from "./components/Tiles/Tile";
import FileExplorer from "./components/Tiles/FileExplorer";
import { useCallback, useEffect, useState } from "react";
import Editor from "./components/Tiles/Editor";
import MenuBar from "./components/Menu/MenuBar";
import "./IDE.css";

export interface IDEProps {
  openFolder: string;
}

export default ({ openFolder }: IDEProps) => {
  const [openFile, setOpenFile] = useState<string | null>(null);
  const [openFiles, setOpenFiles] = useState<string[]>([]);
  const [saveFile, setSaveFile] = useState<(() => void) | null>(null);

  const [callbacks, setCallbacks] = useState<Record<string, () => void>>({});

  useEffect(() => {
    setCallbacks({
      save: saveFile ?? (() => {}),
    });
  }, [saveFile]);

  useEffect(() => {
    if (openFiles.length === 0) {
      setOpenFile(null);
    }
    if (!openFiles.includes(openFile!)) {
      setOpenFile(openFiles[0]);
    }
  }, [openFiles]);

  const openNewFile = useCallback((file: string) => {
    setOpenFile(file);
    setOpenFiles((oF) => {
      if (!oF.includes(file)) return [file, ...oF];
      return oF;
    });
  }, []);

  return (
    <div className="ide-container">
      <MenuBar callbacks={callbacks} />
      <Splitter direction={SplitDirection.Horizontal} initialSizes={[20, 80]}>
        <Tile>
          <FileExplorer openFolder={openFolder} setOpenFile={openNewFile} />
        </Tile>
        <Splitter direction={SplitDirection.Vertical} initialSizes={[70, 30]}>
          <Editor
            openFiles={openFiles}
            focusedFile={openFile}
            setSaveFile={setSaveFile}
            setOpenFiles={setOpenFiles}
          />
          <Tile title="Terminal">Terminal</Tile>
        </Splitter>
      </Splitter>
    </div>
  );
};
