import { useEffect, useMemo, useState } from "react";
import Tile from "./Tile";
import { fs, path } from "@tauri-apps/api";
import { Button, Typography } from "@mui/joy";
import { Construction, PhonelinkSetup } from "@mui/icons-material";
import CommandButton from "../CommandButton";

export interface RunPanelProps {
  openFolder: string;
}
export default function RunPanel({ openFolder }: RunPanelProps) {
  const [hasMakefile, setHasMakefile] = useState(false);

  const buildIcon = useMemo(() => <Construction />, []);
  const buildParameters = useMemo(() => ({ folder: openFolder }), [openFolder]);

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
        <CommandButton
          command="build_theos"
          label="Build"
          icon={buildIcon}
          parameters={buildParameters}
        />
        <Button startDecorator={<PhonelinkSetup />} size="md" disabled={true}>
          Build & Install
        </Button>
      </div>
    </Tile>
  );
}
