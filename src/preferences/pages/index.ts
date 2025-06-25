import { preferenceRegistry } from "../registry";
import { PreferenceCategory } from "../types";

import { generalPage } from "./general";
import { appearancePage } from "./appearance";
import { appleIdPage } from "./appleId";
import { certificatesPage } from "./certificates";
import { appIdsPage } from "./appIds";
import { developerPage } from "./developer";

const generalCategory: PreferenceCategory = {
  id: "general",
  name: "General",
  pages: [generalPage, appearancePage],
};

const appleCategory: PreferenceCategory = {
  id: "apple",
  name: "Apple",
  pages: [appleIdPage, certificatesPage, appIdsPage],
};

const advancedCategory: PreferenceCategory = {
  id: "advanced",
  name: "Advanced",
  pages: [developerPage],
};

preferenceRegistry.registerCategory(generalCategory);
preferenceRegistry.registerCategory(appleCategory);
preferenceRegistry.registerCategory(advancedCategory);

export { preferenceRegistry };
