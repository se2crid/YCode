import { CssVarsProvider } from "@mui/joy/styles";
import { Sheet } from "@mui/joy";
import "@fontsource/inter";
import Onboarding from "./Onboarding";

const App = () => {
  return (
    <CssVarsProvider defaultMode="dark">
      <Sheet
        sx={{
          width: "100%",
          height: "100%",
        }}
      >
        <Onboarding />
      </Sheet>
    </CssVarsProvider>
  );
};

export default App;
