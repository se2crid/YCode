import { createPreferencePage } from "../helpers";

export const editorPage = createPreferencePage("editor", "Editor", [], {
  description: "Configure editor settings and behavior",
  category: "editor",
});
