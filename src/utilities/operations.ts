export type Operation = {
  id: string;
  title: string;
  steps: OperationStep[];
};

export type OperationStep = {
  id: string;
  title: string;
};

export type OperationState = {
  current: Operation;
  completed: string[];
  started: string[];
  failed: {
    stepId: string;
    extraDetails: string;
  }[];
};

type OperationInfoUpdate = {
  updateType: "started" | "finished";
  stepId: string;
};

type OperationFailedUpdate = {
  updateType: "failed";
  stepId: string;
  extraDetails: string;
};

export type OperationUpdate = OperationInfoUpdate | OperationFailedUpdate;

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
    {
      id: "cleanup",
      title: "Clean Up",
    },
  ],
};
