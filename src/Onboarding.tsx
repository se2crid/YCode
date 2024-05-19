import { useEffect, useMemo, useState } from "react";
import { invoke } from "@tauri-apps/api/tauri";
import { open } from "@tauri-apps/api/shell";
import "./Onboarding.css";
import { Button, Card, CardContent, Divider, Link, Typography } from "@mui/joy";
import RunCommand from "./components/RunCommand";

export default () => {
  useMemo(() => {
    invoke("has_theos").then((response) => {
      setHasTheos(response as boolean);
    });
  }, []);

  const [hasTheos, setHasTheos] = useState<boolean | null>(null);
  const [ready, setReady] = useState(false);
  const [updatingTheos, setUpdatingTheos] = useState(false);
  const [installingTheos, setInstallingTheos] = useState(false);

  // Listen for the update-theos-output event

  useEffect(() => {
    if (hasTheos !== null) {
      setReady(hasTheos);
    }
  }, [hasTheos]);

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
        <Button size="lg" disabled={!ready}>
          Open Project
        </Button>
        {!ready && (
          <Typography level="body-xs">
            Use the cards below to get setup before using YCode!
          </Typography>
        )}
      </div>
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
                  // invoke("update_theos").then(() => {
                  //   setUpdatingTheos(false);
                  // });
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
            <RunCommand
              title="Installing Theos..."
              command="install_theos"
              listener="install-theos-output"
              failedMessage="Failed to install theos"
              doneMessage="Theos has been installed! Please restart YCode."
              run={installingTheos}
              setRun={setInstallingTheos}
            />
          </div>
        </CardContent>
      </Card>
    </div>
  );
};
