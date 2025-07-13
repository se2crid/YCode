export type Operation = {
  id: string;
  title: string;
  steps: OperationStep[];
};

export type OperationStep = {
  id: string;
  title: string;
};

export const installSdkOperation: Operation = {
  id: "install_sdk",
  title: "Installing Darwin SDK",
  steps: [
    {
      id: "create_stage",
      title: "Create Stage",
    },
    {
      id: "extract_xip",
      title: "Extract Xcode.xip",
    },
    {
      id: "install_toolset",
      title: "Install toolset",
    },
  ],
};
