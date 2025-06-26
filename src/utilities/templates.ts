import swiftui from "../assets/swiftui.png";

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

export const templates: Template[] = [
  {
    name: "Basic SwiftUI Application",
    description:
      "A barebones SwiftUI application template to get you started quickly",
    image: swiftui,
    id: "swiftui",
    version: "1.0.0",
    fields: {
      projectName: {
        type: "text",
        label: "Project Name",
        default: "MySwiftUIApp",
      },
      bundleId: {
        type: "text",
        label: "Bundle Identifier",
        default: "com.example.myswiftuiapp",
      },
      projectDescription: {
        type: "text",
        label: "Project Description",
        default: "A basic SwiftUI application",
      },
      author: {
        type: "text",
        label: "Author",
        default: "Your Name",
      },
    },
  },
];
