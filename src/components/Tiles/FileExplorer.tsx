import {
  Accordion,
  AccordionDetails,
  AccordionGroup,
  AccordionSummary,
  Button,
} from "@mui/joy";
import "./FileExplorer.css";
import { useCallback, useEffect, useState } from "react";
import { fs, path } from "@tauri-apps/api";

interface FileItemProps {
  filePath: string;
  isDirectory: boolean;
  setOpenFile: (file: string) => void;
  openDefault?: boolean;
}

const FileItem: React.FC<FileItemProps> = ({
  filePath,
  isDirectory,
  setOpenFile,
  openDefault = false,
}) => {
  const handleOpenFile = useCallback(() => {
    setOpenFile(filePath);
  }, [filePath]);

  const [children, setChildren] = useState<
    {
      path: string;
      isDirectory: boolean;
    }[]
  >([]);
  const [name, setName] = useState("");
  const [expanded, setExpanded] = useState(openDefault);

  useEffect(() => {
    (async () => {
      setName(await path.basename(filePath));
      if (!isDirectory || !expanded) return;
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
  }, [filePath, expanded]);

  const handleAccordionChange = (
    _: React.SyntheticEvent,
    isExpanded: boolean
  ) => {
    setExpanded(isExpanded);
  };

  if (isDirectory) {
    return (
      <Accordion onChange={handleAccordionChange} defaultExpanded={openDefault}>
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
        onClick={handleOpenFile}
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
        openDefault={true}
      />
    </div>
  );
};
