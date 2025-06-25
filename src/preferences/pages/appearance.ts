import { createPreferencePage, createItems } from "../helpers";
import { getAllWindows } from "@tauri-apps/api/window";

export const appearancePage = createPreferencePage(
  "appearance",
  "Appearance",
  [
    createItems.select(
      "theme",
      "Theme",
      [
        { label: "Light", value: "light" },
        { label: "Dark", value: "dark" },
      ],
      "Select the theme for the application.",
      "light",
      async (value) => {
        let windows = await getAllWindows();
        for (const win of windows) {
          await win.setTheme(value as "light" | "dark");
        }
      }
    ),
  ],
  {
    description: "Customize the look and feel of the application",
    category: "general",
  }
);
