import { createPreferencePage } from "../helpers";
import { getAllWindows } from "@tauri-apps/api/window";

export const appearancePage = createPreferencePage(
  "appearance",
  "Appearance",
  [
    {
      id: "theme",
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
  {
    description: "Customize the look and feel of the application",
    category: "general",
  }
);
