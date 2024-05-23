import { CssVarsProvider } from "@mui/joy/styles";
import { Sheet } from "@mui/joy";
import "@fontsource/inter";
import Onboarding from "./Onboarding";
import { useCallback, useState } from "react";
import IDE from "./IDE";

const App = () => {
  const [openFolder, setOpenFolder] = useState<string | null>(null);
  const [page, setPage] = useState("onboarding");

  const openProject = useCallback((path: string) => {
    setOpenFolder(path);
    setPage("ide");
  }, []);

  return (
    <CssVarsProvider defaultMode="system">
      <Sheet
        sx={{
          width: "100%",
          height: "100%",
          overflow: "auto",
        }}
      >
        {page === "onboarding" && <Onboarding openProject={openProject} />}
        {page === "ide" && <IDE openFolder={openFolder!} />}
      </Sheet>
    </CssVarsProvider>
  );
};

export default App;
