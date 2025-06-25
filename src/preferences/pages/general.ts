import { createPreferencePage } from "../helpers";

export const generalPage = createPreferencePage("general", "General", [], {
  description: "General application settings",
  category: "general",
});
