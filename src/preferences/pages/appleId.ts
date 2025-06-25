import { createPreferencePage, createItems } from "../helpers";
import { invoke } from "@tauri-apps/api/core";

export const appleIdPage = createPreferencePage(
  "apple-id",
  "Apple ID",
  [
    createItems.select(
      "anisette-server",
      "Anisette Server",
      [
        { label: "Sidestore (.io)", value: "ani.sidestore.io" },
        { label: "Sidestore (.app)", value: "ani.sidestore.app" },
      ],
      "Select an anisette server to use for Apple ID authentication.",
      "ani.sidestore.io"
    ),
    {
      id: "apple-id-email",
      name: "Apple ID",
      description: "The apple ID email you are currently logged in with.",
      type: "info",
      defaultValue: async () => {
        const appleId = await invoke<string>("get_apple_email");
        return appleId || "Not logged in";
      },
    },
    createItems.button(
      "reset-anisette",
      "Reset Anisette",
      "Remove all anisette data (will require 2fa again)",
      async () => {
        await invoke("reset_anisette");
      }
    ),
    createItems.button(
      "reset-credentials",
      "Reset Saved Credentials",
      "Remove saved Apple ID credentials",
      async () => {
        await invoke("delete_stored_credentials");
      }
    ),
  ],
  {
    description: "Manage your Apple ID authentication",
    category: "general",
  }
);
