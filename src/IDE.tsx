import Splitter, { SplitDirection } from "@devbookhq/splitter";
import CodeEditor from "./components/CodeEditor";
import Tile from "./components/Tile";
import FileExplorer from "./components/FileExplorer";
import { useState } from "react";

export interface IDEProps {
  openFolder: string;
}

export default ({ openFolder }: IDEProps) => {
  const [openFile, setOpenFile] = useState<string | null>(null);
  const [openFiles, setOpenFiles] = useState<string[]>([]);

  return (
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
        {openFile && <CodeEditor file={openFile} />}
        <Tile title="Terminal">Terminal</Tile>
      </Splitter>
    </Splitter>
  );
};
