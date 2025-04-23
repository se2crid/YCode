import { CssVarsProvider, extendTheme } from "@mui/joy/styles";
import { Sheet } from "@mui/joy";
import "@fontsource/inter";
import Onboarding from "./pages/Onboarding";
import IDE from "./pages/IDE";
import Prefs from "./pages/Prefs";
import { BrowserRouter, Route, Routes } from "react-router";
import { StoreProvider } from "./utilities/StoreContext";
import { IDEProvider } from "./utilities/IDEContext";
import Splash from "./pages/Splash";

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
  return (
    <BrowserRouter>
      <CssVarsProvider defaultMode="system" theme={theme}>
        <StoreProvider>
          <IDEProvider>
            <Sheet
              sx={{
                width: "100%",
                height: "100%",
                overflow: "auto",
              }}
            >
              <Routes>
                <Route path="*" element={<Onboarding />} />
                <Route path="/ide/:path" element={<IDE />} />
                <Route path="/preferences/:page?" element={<Prefs />} />
                <Route path="/splashscreen" element={<Splash />} />
              </Routes>
            </Sheet>
          </IDEProvider>
        </StoreProvider>
      </CssVarsProvider>
    </BrowserRouter>
  );
};

export default App;
