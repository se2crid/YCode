import { createItems, createPreferencePage } from "../helpers";

export const developerPage = createPreferencePage(
  "developer",
  "Developer",
  [
    createItems.checkbox(
      "delete-app-ids",
      "Allow deleting app IDs",
      "Reveal the delete button in the app ID page (note: they will still count towards your limit!)"
    ),
  ],
  {
    description: "Internal developer settings",
    category: "advanced",
  }
);
