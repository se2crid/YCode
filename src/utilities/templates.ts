import swiftui from "../assets/swiftui.png";
import uikit from "../assets/uikit.png";

export interface TemplateField {
  type: string;
  label: string;
  default: string;
}

export interface Template {
  name: string;
  description: string;
  id: string;
  image?: string;
  version: string;
  fields: {
    [key: string]: TemplateField;
  };
}

const defaultFields = {
  projectName: {
    type: "text",
    label: "Project Name",
    default: "MyProject",
  },
  bundleId: {
    type: "text",
    label: "Bundle Identifier",
    default: "com.example.myproject",
  },
};

export const templates: Template[] = [
  {
    name: "Basic SwiftUI Application",
    description: "A barebones SwiftUI template to get you started quickly",
    image: swiftui,
    id: "swiftui",
    version: "1.0.0",
    fields: {
      ...defaultFields,
    },
  },
  {
    name: "Basic UIKit Application",
    description: "A barebones UIKit template to get you started quickly",
    image: uikit,
    id: "uikit",
    version: "1.0.0",
    fields: {
      ...defaultFields,
    },
  },
];
