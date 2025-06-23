import { CssVarsProvider, extendTheme } from "@mui/joy/styles";
import { Sheet } from "@mui/joy";
import "@fontsource/inter";
import Onboarding from "./pages/Onboarding";
import IDE from "./pages/IDE";
import Prefs from "./pages/Prefs";
import { BrowserRouter, Route, Routes, Navigate, Outlet } from "react-router";
import { StoreProvider, useStore } from "./utilities/StoreContext";
import { IDEProvider } from "./utilities/IDEContext";
import Splash from "./pages/Splash";
import { CommandProvider } from "./utilities/Command";
import { ToastProvider } from "react-toast-plus";

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

// Layout with IDE-related providers
const IDELayout = () => {
  const [appTheme] = useStore("appearance/theme", "light");
  return (
    <ToastProvider
      toastOptions={{ placement: "bottom-right" }}
      toastStyles={
        appTheme == "dark"
          ? { toastBgColor: "#333", toastTextColor: "#fff" }
          : {}
      }
    >
      <CommandProvider>
        <IDEProvider>
          <Outlet />
        </IDEProvider>
      </CommandProvider>
    </ToastProvider>
  );
};

const App = () => {
  return (
    <BrowserRouter>
      <CssVarsProvider defaultMode="system" theme={theme}>
        <StoreProvider>
          <Sheet
            onContextMenu={(e) => {
              e.preventDefault();
            }}
            sx={{
              width: "100%",
              height: "100%",
              overflow: "auto",
            }}
          >
            <Routes>
              <Route element={<IDELayout />}>
                <Route index element={<Onboarding />} />
                <Route path="/ide/:path" element={<IDE />} />
                <Route path="*" element={<Navigate to="/" replace />} />
              </Route>

              <Route path="/preferences/:page?" element={<Prefs />} />
              <Route path="/splashscreen" element={<Splash />} />
            </Routes>
          </Sheet>
        </StoreProvider>
      </CssVarsProvider>
    </BrowserRouter>
  );
};

export default App;
