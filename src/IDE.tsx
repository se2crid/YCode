import Splitter, { SplitDirection } from "@devbookhq/splitter";
import Tile from "./components/Tiles/Tile";
import FileExplorer from "./components/Tiles/FileExplorer";
import { useState } from "react";
import Editor from "./components/Tiles/Editor";
import MenuBar from "./components/Menu/MenuBar";
import "./IDE.css";

export interface IDEProps {
  openFolder: string;
}

export default ({ openFolder }: IDEProps) => {
  const [openFile, setOpenFile] = useState<string | null>(null);
  const [openFiles, setOpenFiles] = useState<string[]>([]);

  return (
    <div className="ide-container">
      <MenuBar />
      <Splitter direction={SplitDirection.Horizontal} initialSizes={[20, 80]}>
        <Tile>
          <FileExplorer
            openFolder={openFolder}
            setOpenFile={(file: string) => {
              setOpenFile(file);
              setOpenFiles((oF) => [file, ...oF]);
            }}
          />
        </Tile>
        <Splitter direction={SplitDirection.Vertical} initialSizes={[70, 30]}>
          <Editor openFiles={openFiles} focusedFile={openFile} />
          <Tile title="Terminal">Terminal</Tile>
        </Splitter>
      </Splitter>
    </div>
  );
};
