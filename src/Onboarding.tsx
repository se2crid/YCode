import { useEffect, useMemo, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-shell";
import "./Onboarding.css";
import { Button, Card, CardContent, Divider, Link, Typography } from "@mui/joy";
import RunCommand from "./components/RunCommand";
import * as dialog from "@tauri-apps/plugin-dialog";

export interface OnboardingProps {
  openProject: (path: string) => void;
}

export default ({ openProject }: OnboardingProps) => {
  useMemo(() => {
    invoke("has_theos").then((response) => {
      setHasTheos(response as boolean);
    });
    invoke("has_wsl").then((response) => {
      setHasWSL(response as boolean);
    });
    invoke("is_windows").then((response) => {
      setIsWindows(response as boolean);
    });
  }, []);

  const [hasTheos, setHasTheos] = useState<boolean | null>(null);
  const [isWindows, setIsWindows] = useState<boolean | null>(null);
  const [hasWSL, setHasWSL] = useState<boolean | null>(null);
  const [ready, setReady] = useState(false);
  const [updatingTheos, setUpdatingTheos] = useState(false);
  const [installingTheos, setInstallingTheos] = useState(false);

  useEffect(() => {
    if (hasTheos !== null && isWindows !== null && hasWSL !== null) {
      setReady(hasTheos && (isWindows ? hasWSL : true));
    } else {
      setReady(false);
    }
  }, [hasTheos, hasWSL, isWindows]);

  return (
    <div className="onboarding">
      <div className="header">
        <Typography level="h1">Welcome to YCode!</Typography>
        <Typography level="body-sm">
          A simple IDE for iOS development with swift on linux.
        </Typography>
      </div>
      <div className="buttons" style={{ marginBottom: "20px" }}>
        <Button
          size="lg"
          disabled={!ready}
          sx={{
            marginRight: "10px",
          }}
        >
          Create New
        </Button>
        <Button
          size="lg"
          disabled={!ready}
          onClick={async () => {
            const path = await dialog.open({
              directory: true,
              multiple: false,
            });
            if (path) {
              openProject(path);
            }
          }}
        >
          Open Project
        </Button>
        {!ready && (
          <Typography level="body-xs">
            Use the cards below to get setup before using YCode!
          </Typography>
        )}
      </div>
      {isWindows === true && (
        <Card variant="soft" sx={{ marginBottom: "10px" }}>
          <Typography level="h3">WSL</Typography>
          <Typography level="body-sm">
            Windows subsystem for linux (WSL) is required to use YCode on
            windows. Learn more about WSL on{" "}
            <Link
              href="#"
              onClick={(e) => {
                e.preventDefault();
                open("https://learn.microsoft.com/en-us/windows/wsl/");
              }}
            >
              microsoft.com
            </Link>
            .
          </Typography>
          <Divider />
          <CardContent>
            <Typography level="body-md">
              {hasWSL === null ? (
                "Checking for wsl..."
              ) : hasWSL ? (
                "WSL is already installed on your system!"
              ) : (
                <>
                  WSL is not installed on your system. Please follow the guide
                  on{" "}
                  <Link
                    href="#"
                    onClick={(e) => {
                      e.preventDefault();
                      open(
                        "https://learn.microsoft.com/en-us/windows/wsl/install"
                      );
                    }}
                  >
                    microsoft.com
                  </Link>
                  .
                </>
              )}
            </Typography>
          </CardContent>
        </Card>
      )}
      <Card variant="soft">
        <Typography level="h3">Theos</Typography>
        <Typography level="body-sm">
          Theos is a cross-platform suite of tools for building and deploying
          software for iOS. It is the core of YCode. Learn more about theos at{" "}
          <Link
            href="#"
            onClick={(e) => {
              e.preventDefault();
              open("https://theos.dev");
            }}
          >
            theos.dev
          </Link>
          .
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
          {hasTheos === true && (
            <Typography level="body-xs">
              If you manually installed theos without manually installing
              https://github.com/kabiroberai/swift-toolchain-linux/, please
              delete $THEOS/toolchain and press reinstall.
            </Typography>
          )}
          <div>
            {hasTheos === true && (
              <Button
                sx={{
                  margin: "10px 0",
                }}
                variant="soft"
                onClick={() => {
                  setUpdatingTheos(true);
                }}
              >
                Update Theos
              </Button>
            )}
            {hasTheos === true && (
              <RunCommand
                title="Updating Theos..."
                command="update_theos"
                listener="update-theos-output"
                failedMessage="Failed to update Theos, you can try manually running $THEOS/bin/update-theos"
                doneMessage="Theos is up-to-date!"
                run={updatingTheos}
                setRun={setUpdatingTheos}
              />
            )}
            {hasTheos !== null && (!isWindows || hasWSL) && (
              <Button
                sx={{
                  margin: hasTheos ? "10px" : "0",
                }}
                variant="soft"
                onClick={() => {
                  setInstallingTheos(true);
                }}
              >
                {hasTheos ? "Reinstall Theos" : "Install Theos"}
              </Button>
            )}
            <RunCommand
              title="Installing Theos..."
              command={
                isWindows === true ? "install_theos_windows" : "install_theos"
              }
              listener="install-theos-output"
              failedMessage="Failed to install theos"
              doneMessage="Theos has been installed! Please restart YCode."
              run={installingTheos}
              setRun={setInstallingTheos}
              askPassword={isWindows === true}
            />
          </div>
        </CardContent>
      </Card>
    </div>
  );
};
