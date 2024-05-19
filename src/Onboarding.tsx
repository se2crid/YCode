import { useMemo, useState } from "react";
import { invoke } from "@tauri-apps/api/tauri";
import "./Onboarding.css";
import Card from "@mui/joy/Card";
import CardContent from "@mui/joy/CardContent";
import Typography from "@mui/joy/Typography";
import Divider from "@mui/joy/Divider";

export default () => {
  useMemo(() => {
    invoke("has_theos").then((response) => {
      setHasTheos(response as boolean);
    });
  }, []);
  const [hasTheos, setHasTheos] = useState<boolean | null>(null);
  const [loaded, setLoaded] = useState(false);

  return (
    <div className="onboarding">
      <h1>Welcome to YCode!</h1>
      <p>This is a simple IDE for iOS development with swift on linux.</p>
      <Card variant="soft">
        <Typography level="h3">Theos</Typography>
        <Typography level="body-sm">
          Theos is a cross-platform suite of tools for building and deploying
          software for iOS. It is the core of YCode.
        </Typography>
        <Divider />
        <CardContent>
          <Typography level="body-md">
            {hasTheos === null
              ? "Checking for Theos..."
              : hasTheos
              ? "Theos is already installed on your system!"
              : "Theos is not installed on your system."}
          </Typography>
        </CardContent>
      </Card>
    </div>
  );
};
