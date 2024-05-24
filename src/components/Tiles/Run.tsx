import { useEffect, useState } from "react";
import Tile from "./Tile";
import { fs, invoke, path } from "@tauri-apps/api";
import { Button, Typography } from "@mui/joy";
import { Construction, PhonelinkSetup } from "@mui/icons-material";

export interface RunPanelProps {
  openFolder: string;
}
export default function RunPanel({ openFolder }: RunPanelProps) {
  const [hasMakefile, setHasMakefile] = useState(false);

  useEffect(() => {
    (async () => {
      setHasMakefile(await fs.exists(await path.join(openFolder, "Makefile")));
    })();
  }, [openFolder]);

  if (!hasMakefile) {
    return (
      <Tile title="Run">
        <Typography level="body-sm" color="warning">
          This does not appear to be a theos project.
        </Typography>
      </Tile>
    );
  }

  return (
    <Tile title="Run">
      <div style={{ padding: "15px 5px" }}>
        <Button
          startDecorator={<Construction />}
          size="md"
          sx={{ marginRight: "10px" }}
          onClick={() => {
            invoke("build_theos", { folder: openFolder });
          }}
        >
          Build
        </Button>
        <Button startDecorator={<PhonelinkSetup />} size="md">
          Build & Install
        </Button>
      </div>
    </Tile>
  );
}
