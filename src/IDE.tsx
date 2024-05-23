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

  const openNewFile = useCallback(
    (file: string) => {
      setOpenFile(file);
      setOpenFiles((oF) => {
        console.log(oF, file);
        if (!oF.includes(file)) return [file, ...oF];
        return oF;
      });
    },
    [openFiles]
  );

  useEffect(() => {
    console.log("new save file", saveFile);
  }, [saveFile]);

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
          />
          <Tile title="Terminal">Terminal</Tile>
        </Splitter>
      </Splitter>
    </div>
  );
};
