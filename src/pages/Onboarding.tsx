import { useEffect, useState } from "react";
import { open } from "@tauri-apps/plugin-shell";
import "./Onboarding.css";
import { Button, Card, CardContent, Divider, Link, Typography } from "@mui/joy";
import RunCommand from "../components/RunCommand";
import { useIDE } from "../utilities/IDEContext";
import logo from "../assets/logo.png";

export interface OnboardingProps {}

export default ({}: OnboardingProps) => {
  const { hasTheos, hasWSL, isWindows, openFolderDialog } = useIDE();
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
      <div className="onboarding-header">
        <img src={logo} alt="YCode Logo" className="onboarding-logo" />
        <div>
          <Typography level="h1">Welcome to YCode!</Typography>
          <Typography level="body-sm">
            IDE for iOS Development on Windows and Linux
          </Typography>
        </div>
      </div>
      <div className="onboarding-buttons">
        <Button
          size="lg"
          // disabled={!ready}
          disabled
          className={!hasTheos ? "disabled-button" : ""}
        >
          Create New (Coming Soon)
        </Button>
        <Button size="lg" disabled={!ready} onClick={openFolderDialog}>
          Open Project
        </Button>
      </div>

      <Typography level="body-sm">
        {ready
          ? "Use the cards below to manage your YCode setup"
          : "Use the cards below to get setup before using YCode!"}
      </Typography>
      <div className="onboarding-cards">
        {isWindows && (
          <Card variant="soft">
            <Typography level="h3">WSL</Typography>
            <Typography level="body-sm">
              Windows Subsystem for Linux (WSL) is required to use YCode on
              Windows. Learn more about WSL on{" "}
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
            Theos is a cross-platform suite of tools for building software for
            iOS. It is the core of YCode. Learn more about theos at{" "}
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
                If you manually installed theos without installing
                https://github.com/kabiroberai/swift-toolchain-linux/, please
                delete $THEOS/toolchain and press reinstall.
              </Typography>
            )}
            <div
              style={{
                marginTop: "var(--padding-md)",
                display: "flex",
                gap: "var(--padding-md)",
              }}
            >
              {hasTheos === true && (
                <Button
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
                  isWindows === true
                    ? "install_theos_windows"
                    : "install_theos_linux"
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
    </div>
  );
};
