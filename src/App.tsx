import Splitter, { SplitDirection } from "@devbookhq/splitter";
import CodeEditor from "./components/CodeEditor";
import Tile from "./components/Tile";
import FileExplorer from "./components/FileExplorer";

const App = () => {
  return (
    <Splitter direction={SplitDirection.Horizontal} initialSizes={[20, 80]}>
      <Tile title="File Explorer">
        <FileExplorer />
      </Tile>
      <Splitter direction={SplitDirection.Vertical} initialSizes={[70, 30]}>
        <CodeEditor />
        <Tile title="Terminal">Terminal</Tile>
      </Splitter>
    </Splitter>
  );
};

export default App;
