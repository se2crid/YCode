import { preferenceRegistry } from "../registry";
import { PreferenceCategory } from "../types";

import { generalPage } from "./general";
import { appearancePage } from "./appearance";
import { appleIdPage } from "./appleId";
import { certificatesPage } from "./certificates";
import { editorPage } from "./editor";

const generalCategory: PreferenceCategory = {
  id: "general",
  name: "General",
  pages: [generalPage, appearancePage],
};

const editorCategory: PreferenceCategory = {
  id: "editor",
  name: "Editor",
  pages: [editorPage],
};

const appleCategory: PreferenceCategory = {
  id: "apple",
  name: "Apple",
  pages: [appleIdPage, certificatesPage],
};

preferenceRegistry.registerCategory(generalCategory);
preferenceRegistry.registerCategory(editorCategory);
preferenceRegistry.registerCategory(appleCategory);

export { preferenceRegistry };
