// create a context to store a few state values about the system that are checked at startup
import React, {
  createContext,
  useCallback,
  useEffect,
  useMemo,
  useRef,
  useState,
} from "react";
import { invoke } from "@tauri-apps/api/core";
import { Window } from "@tauri-apps/api/window";
import { emit, listen } from "@tauri-apps/api/event";
import * as dialog from "@tauri-apps/plugin-dialog";
import { useToast } from "react-toast-plus";
import { useNavigate } from "react-router-dom";
import { Button, Input, Modal, ModalDialog, Typography } from "@mui/joy";

export interface IDEContextType {
  initialized: boolean;
  isWindows: boolean;
  hasWSL: boolean;
  hasTheos: boolean;
  devices: DeviceInfo[];
  openFolderDialog: () => void;
}

export type DeviceInfo = {
  name: string;
  id: number;
};

export const IDEContext = createContext<IDEContextType | null>(null);

export const IDEProvider: React.FC<{
  children: React.ReactNode;
}> = ({ children }) => {
  const [isWindows, setIsWindows] = useState<boolean>(false);
  const [hasWSL, setHasWSL] = useState<boolean>(false);
  const [hasTheos, setHasTheos] = useState<boolean>(false);
  const [initialized, setInitialized] = useState(false);
  const [devices, setDevices] = useState<DeviceInfo[]>([]);

  useEffect(() => {
    let initPromises: Promise<void>[] = [];
    initPromises.push(
      invoke("has_theos").then((response) => {
        setHasTheos(response as boolean);
      })
    );
    initPromises.push(
      invoke("has_wsl").then((response) => {
        setHasWSL(response as boolean);
      })
    );
    initPromises.push(
      invoke("is_windows").then((response) => {
        setIsWindows(response as boolean);
      })
    );

    Promise.all(initPromises)
      .then(() => {
        setInitialized(true);
      })
      .catch((error) => {
        console.error("Error initializing IDE context:", error);
        alert(
          "An error occurred while initializing the IDE context. Please check the console for details."
        );
      });
  }, []);

  useEffect(() => {
    if (initialized) {
      let changeWindows = async () => {
        let splash = await Window.getByLabel("splashscreen");
        let main = await Window.getByLabel("main");
        if (splash && main) {
          splash.close();
          await main.show();
          main.setFocus();
        }
      };
      changeWindows();
    }
  }, [initialized]);

  const listenerAdded = useRef(false);
  const listener2Added = useRef(false);
  const unlisten = useRef<() => void>(() => {});
  const unlisten2fa = useRef<() => void>(() => {});

  const { addToast } = useToast();

  const [tfaOpen, setTfaOpen] = useState(false);
  const tfaInput = useRef<HTMLInputElement | null>(null);

  useEffect(() => {
    if (!listenerAdded.current) {
      (async () => {
        const unlistenFn = await listen("idevices", (event) => {
          let devices = event.payload as DeviceInfo[];
          console.log("Received devices:", devices);
          setDevices(devices);
          if (devices.length === 0) {
            addToast.info("No devices found");
          }
        });
        unlisten.current = unlistenFn;
      })();
      listenerAdded.current = true;
    }
    return () => {
      unlisten.current();
    };
  }, []);

  useEffect(() => {
    if (!listener2Added.current) {
      (async () => {
        const unlistenFn = await listen("2fa-required", () => {
          console.log("2FA required, opening dialog");
          setTfaOpen(true);
        });
        unlisten2fa.current = unlistenFn;
      })();
      listener2Added.current = true;
    }
    return () => {
      unlisten2fa.current();
    };
  }, []);

  const navigate = useNavigate();

  const openFolderDialog = useCallback(async () => {
    const path = await dialog.open({
      directory: true,
      multiple: false,
    });
    if (path) {
      navigate("/ide/" + encodeURIComponent(path));
    }
  }, []);

  const contextValue = useMemo(
    () => ({
      isWindows,
      hasWSL,
      hasTheos,
      initialized,
      devices,
      openFolderDialog,
    }),
    [isWindows, hasWSL, hasTheos, initialized, devices]
  );

  return (
    <IDEContext.Provider value={contextValue}>
      {children}
      <Modal
        open={tfaOpen}
        onClose={() => {
          if (!tfaInput.current?.value) {
            addToast.error("Please enter a 2FA code");
            return;
          }
          emit("2fa-recieved", tfaInput.current?.value || "");
          setTfaOpen(false);
        }}
      >
        <ModalDialog>
          <Typography level="body-md">
            A two-factory authentication code has been sent, please enter it
            below.
          </Typography>
          <form
            onSubmit={() => {
              if (!tfaInput.current?.value) {
                addToast.error("Please enter a 2FA code");
                return;
              }
              emit("2fa-recieved", tfaInput.current?.value || "");
              setTfaOpen(false);
              tfaInput.current!.value = ""; // Clear the input after submission
              return false; // Prevent form submission
            }}
          >
            <Input type="number" slotProps={{ input: { ref: tfaInput } }} />
            <Button
              variant="soft"
              sx={{
                margin: "10px 0",
                width: "100%",
              }}
              type="submit"
            >
              Submit
            </Button>
          </form>
        </ModalDialog>
      </Modal>
    </IDEContext.Provider>
  );
};

export const useIDE = () => {
  const context = React.useContext(IDEContext);
  if (!context) {
    throw new Error("useIDEContext must be used within an IDEContextProvider");
  }
  return context;
};
