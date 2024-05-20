import { CssVarsProvider } from "@mui/joy/styles";
import { Sheet } from "@mui/joy";
import "@fontsource/inter";
import Onboarding from "./Onboarding";
import { useState } from "react";
import IDE from "./IDE";

const App = () => {
  const [openFolder, setOpenFolder] = useState<string | null>(null);
  const [page, setPage] = useState("onboarding");

  return (
    <CssVarsProvider defaultMode="system">
      <Sheet
        sx={{
          width: "100%",
          height: "100%",
        }}
      >
        {page === "onboarding" && (
          <Onboarding
            openProject={(path) => {
              setOpenFolder(path);
              setPage("ide");
            }}
          />
        )}
        {page === "ide" && <IDE openFolder={openFolder!} />}
      </Sheet>
    </CssVarsProvider>
  );
};

export default App;
