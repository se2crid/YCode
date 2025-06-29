import { useEffect, useMemo, useState } from "react";
import { open } from "@tauri-apps/plugin-shell";
import "./Onboarding.css";
import {
  Button,
  Card,
  CardContent,
  Divider,
  FormControl,
  Link,
  Radio,
  RadioGroup,
  Typography,
} from "@mui/joy";
import { Toolchain, useIDE } from "../utilities/IDEContext";
import logo from "../assets/logo.png";
import { useNavigate } from "react-router";
import { openUrl } from "@tauri-apps/plugin-opener";

export interface OnboardingProps {}

export default ({}: OnboardingProps) => {
  const {
    selectedToolchain,
    setSelectedToolchain,
    toolchains,
    scanToolchains,
    hasWSL,
    isWindows,
    openFolderDialog,
    locateToolchain,
  } = useIDE();
  const [ready, setReady] = useState(false);
  const navigate = useNavigate();

  useEffect(() => {
    if (toolchains !== null && isWindows !== null && hasWSL !== null) {
      setReady(selectedToolchain !== null && (isWindows ? hasWSL : true));
    } else {
      setReady(false);
    }
  }, [selectedToolchain, toolchains, hasWSL, isWindows]);

  let allToolchains = useMemo(() => {
    let all: Toolchain[] = [];
    if (toolchains !== null && toolchains.toolchains) {
      all = [...toolchains.toolchains];
    }
    if (
      selectedToolchain &&
      !all.some(
        (t) => stringifyToolchain(t) === stringifyToolchain(selectedToolchain)
      )
    ) {
      all.push(selectedToolchain);
    }
    return all;
  }, [selectedToolchain, toolchains]);

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
      <div>
        <Typography level="h3" className="onboarding-version" color="warning">
          ⚠️ Early Access Version ⚠️
        </Typography>
        <Typography level="body-md">
          This is an early access version of YCode. Expect bugs. Please report
          any issues you find on{" "}
          <Link
            href="#"
            onClick={(e) => {
              e.preventDefault();
              open("https://github.com/nab138/ycode/issues");
            }}
          >
            github
          </Link>
          .
        </Typography>
      </div>
      <div className="onboarding-buttons">
        <Button
          size="lg"
          disabled={!ready}
          className={!ready ? "disabled-button" : ""}
          onClick={() => {
            if (ready) {
              navigate("/new");
            }
          }}
        >
          Create New
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
                        openUrl(
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
          <Typography level="h3">Swift</Typography>
          <Typography level="body-sm">
            You will need a Swift toolchain to use YCode. It is recommended to
            install it using swiftly, but you can also install it manually.
          </Typography>
          <Divider />
          <CardContent>
            <Typography level="body-sm">
              {toolchains === null
                ? "Checking for Swift..."
                : toolchains.swiftlyInstalled
                ? `Swiftly Detected: ${toolchains.swiftlyVersion}`
                : "YCode was unable to detect Swiftly."}
            </Typography>
            {toolchains !== null && allToolchains.length === 0 && (
              <Typography
                level="body-md"
                style={{ marginBottom: "var(--padding-md)" }}
                color="warning"
              >
                No Swift toolchains found. You can install one using "
                <span
                  style={{
                    fontFamily: "monospace",
                  }}
                >
                  swiftly install latest
                </span>
                " or manually.
              </Typography>
            )}
            {toolchains !== null && allToolchains.length > 0 && (
              <>
                <Typography level="body-md">Select a toolchain:</Typography>
                <RadioGroup
                  value={stringifyToolchain(selectedToolchain)}
                  sx={{
                    marginTop: "var(--padding-xs)",
                  }}
                >
                  {allToolchains.map((toolchain) => (
                    <FormControl sx={{ marginBottom: "var(--padding-md)" }}>
                      <Radio
                        key={
                          toolchain.path +
                          toolchain.version +
                          toolchain.isSwiftly
                        }
                        label={toolchain.version}
                        value={stringifyToolchain(toolchain)}
                        variant="outlined"
                        overlay
                        onChange={() => setSelectedToolchain(toolchain)}
                      />
                      <div
                        style={{
                          display: "flex",
                          alignItems: "center",
                          gap: "var(--padding-xs)",
                        }}
                      >
                        <Typography level="body-sm">
                          {toolchain.path}
                        </Typography>
                        <Typography level="body-sm" color="primary">
                          {toolchain.isSwiftly
                            ? "(Swiftly)"
                            : "(Manually Installed)"}
                        </Typography>
                      </div>
                    </FormControl>
                  ))}
                </RadioGroup>
              </>
            )}
            <div
              style={{
                display: "flex",
                gap: "var(--padding-md)",
              }}
            >
              {
                <Button variant="soft" onClick={locateToolchain}>
                  Locate Existing Toolchain
                </Button>
              }
              {toolchains?.swiftlyInstalled === false &&
                selectedToolchain === null && (
                  <Button
                    variant="soft"
                    onClick={() => {
                      openUrl("https://swift.org/install/");
                    }}
                  >
                    Download Swift
                  </Button>
                )}
              {
                <Button
                  variant="soft"
                  onClick={() => {
                    scanToolchains();
                  }}
                >
                  Scan Again
                </Button>
              }
            </div>
          </CardContent>
        </Card>
      </div>
    </div>
  );
};

function stringifyToolchain(toolchain: Toolchain | null): string | null {
  if (!toolchain) return null;
  return `${toolchain.path}:${toolchain.version}:${toolchain.isSwiftly}`;
}
