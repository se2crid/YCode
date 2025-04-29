import { useParams } from "react-router";
import "./Prefs.css";

import { Divider, Typography, Link } from "@mui/joy";
import { Link as RouterLink } from "react-router-dom";
import { Fragment, useContext } from "react";
import { StoreContext } from "../utilities/StoreContext";
import Pref from "./Pref";
import { getAllWindows } from "@tauri-apps/api/window";
import { invoke } from "@tauri-apps/api/core";
export type PrefSetting = {
  name: string;
  description: string;
  type: "text" | "select" | "checkbox" | "button";
  options?: Array<{ label: string; value: string }>;
  defaultValue?: any;
  onChange?: (value: any) => void;
};

export type PrefPage = {
  name: string;
  settings?: PrefSetting[];
};
const prefs: PrefPage[][] = [
  [
    {
      name: "General",
    },
    {
      name: "Appearance",
      settings: [
        {
          name: "Theme",
          description: "Select the theme for the application.",
          type: "select",
          options: [
            { label: "Light", value: "light" },
            { label: "Dark", value: "dark" },
          ],
          defaultValue: "light",
          onChange: async (value) => {
            let windows = await getAllWindows();
            for (const win of windows) {
              await win.setTheme(value as "light" | "dark");
            }
          },
        },
      ],
    },
    {
      name: "Apple ID",
      settings: [
        {
          name: "Anisette Server",
          description:
            "Select an anisette server to use for Apple ID authentication.",
          type: "select",
          options: [
            { label: "Sidestore (.io)", value: "ani.sidestore.io" },
            { label: "Sidestore (.app)", value: "ani.sidestore.app" },
          ],
          defaultValue: "ani.sidestore.io",
        },
        {
          name: "Reset Anisette",
          description: "Remove all anisette data (will require 2fa again)",
          type: "button",
          onChange: async () => {
            invoke("reset_anisette");
          },
        },
        {
          name: "Reset Saved Credentials",
          description: "Remove saved Apple ID credentials",
          type: "button",
          onChange: async () => {
            invoke("delete_stored_credentials");
          },
        },
      ],
    },
  ],
  [
    {
      name: "Editor",
    },
  ],
];

export default () => {
  const { page } = useParams<"page">();
  const { store } = useContext(StoreContext);
  const storeExists = store !== null && store !== undefined;
  return (
    <div className="prefs-container">
      <div className="prefs-sidebar-container">
        <div className="prefs-sidebar">
          {prefs.map((pref, index) => (
            <Fragment key={index}>
              {pref.map((p) => (
                <Link
                  level="body-sm"
                  className="prefs-sidebar-item"
                  key={p.name + index}
                  component={RouterLink}
                  to={`/preferences/${p.name}`}
                  sx={{ textDecoration: "none", color: "inherit" }}
                >
                  {p.name}
                </Link>
              ))}
              {index !== prefs.length - 1 && (
                <Divider orientation="horizontal" />
              )}
            </Fragment>
          ))}
        </div>
        <Divider orientation="vertical" />
      </div>
      <div className="prefs-content">
        {page &&
          (prefs.flat().find((p) => p.name === page) ? (
            <>
              <Typography level="title-lg">{page}</Typography>
              {prefs
                .flat()
                .find((p) => p.name === page)
                ?.settings?.map((setting) => (
                  <Pref
                    key={setting.name + page}
                    storeExists={storeExists}
                    pageName={page}
                    setting={setting}
                  />
                ))}
            </>
          ) : (
            <Typography level="h2">Page Not Found</Typography>
          ))}
      </div>
    </div>
  );
};
