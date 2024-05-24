import { CssVarsProvider, extendTheme } from "@mui/joy/styles";
import { Sheet } from "@mui/joy";
import "@fontsource/inter";
import Onboarding from "./Onboarding";
import { useCallback, useState } from "react";
import IDE from "./IDE";

declare module "@mui/joy/IconButton" {
  interface IconButtonPropsSizeOverrides {
    xs: true;
  }
}

const theme = extendTheme({
  components: {
    JoyIconButton: {
      styleOverrides: {
        root: ({ ownerState, theme }) => ({
          ...(ownerState.size === "xs" && {
            "--Icon-fontSize": "1rem",
            "--Button-gap": "0.25rem",
            minHeight: "var(--Button-minHeight, 1.5rem)",
            fontSize: theme.vars.fontSize.xs,
            paddingBlock: "2px",
            paddingInline: "0.25rem",
          }),
        }),
      },
    },
  },
});

const App = () => {
  const [openFolder, setOpenFolder] = useState<string | null>(null);
  const [page, setPage] = useState("onboarding");

  const openProject = useCallback((path: string) => {
    setOpenFolder(path);
    setPage("ide");
  }, []);

  return (
    <CssVarsProvider defaultMode="system" theme={theme}>
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
