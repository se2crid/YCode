import {
  Accordion,
  AccordionDetails,
  AccordionGroup,
  AccordionSummary,
  Button,
} from "@mui/joy";
import "./FileExplorer.css";
import { useEffect, useState } from "react";
import { fs, path } from "@tauri-apps/api";

interface FileItemProps {
  filePath: string;
  isDirectory: boolean;
  setOpenFile: (file: string) => void;
}

const FileItem: React.FC<FileItemProps> = ({
  filePath,
  isDirectory,
  setOpenFile,
}) => {
  const [children, setChildren] = useState<
    {
      path: string;
      isDirectory: boolean;
    }[]
  >([]);
  const [name, setName] = useState("");

  useEffect(() => {
    (async () => {
      setName(await path.basename(filePath));
      if (!isDirectory) return;
      try {
        const files = await fs.readDir(filePath);
        setChildren(
          files.map((file) => {
            return {
              path: file.path,
              isDirectory: file.children !== undefined,
            };
          })
        );
      } catch (error) {
        console.error("Failed to read file:", filePath, error);
      }
    })();
  }, [filePath]);

  if (isDirectory) {
    return (
      <Accordion>
        <AccordionSummary>{name}</AccordionSummary>
        <AccordionDetails>
          <AccordionGroup
            size="sm"
            sx={{
              borderLeft: "1px solid var(--joy-palette-neutral-800, #171A1C)",
            }}
            disableDivider={true}
          >
            {children.map((child) => (
              <FileItem
                key={child.path}
                filePath={child.path}
                isDirectory={child.isDirectory}
                setOpenFile={setOpenFile}
              />
            ))}
          </AccordionGroup>
        </AccordionDetails>
      </Accordion>
    );
  } else {
    return (
      <Button
        size="sm"
        onClick={() => setOpenFile(filePath)}
        variant="plain"
        sx={{ justifyContent: "flex-start" }}
      >
        {name}
      </Button>
    );
  }
};

export interface FileExplorerProps {
  openFolder: string;
  setOpenFile: (file: string) => void;
}
export default ({ openFolder, setOpenFile }: FileExplorerProps) => {
  return (
    <div className={"file-explorer"}>
      <FileItem
        filePath={openFolder}
        isDirectory={true}
        setOpenFile={setOpenFile}
      />
    </div>
  );
};
