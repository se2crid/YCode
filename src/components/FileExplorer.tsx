import "./FileExplorer.css";

export interface FileExplorerProps {
  openFolder: string;
}
export default ({ openFolder }: FileExplorerProps) => {
  return (
    <div className={"file-explorer"}>
      <div>File Explorer Content: {openFolder}</div>
    </div>
  );
};
