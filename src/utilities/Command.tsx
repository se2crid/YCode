import { Mutex } from "async-mutex";
import { invoke } from "@tauri-apps/api";
import { useState } from "react";

const commandRunnerMutex = new Mutex();
let setIsRunningCommand: React.Dispatch<React.SetStateAction<boolean>>;
let setCurrentCommand: React.Dispatch<React.SetStateAction<string | null>>;

export function useCommandRunner() {
  const [isRunningCommand, _setIsRunningCommand] = useState(false);
  const [currentCommand, _setCurrentCommand] = useState<string | null>(null);
  setIsRunningCommand = _setIsRunningCommand; // Save the setter function
  setCurrentCommand = _setCurrentCommand; // Save the setter function

  return { isRunningCommand, currentCommand };
}

export async function runCommand(
  command: string,
  parameters?: Record<string, unknown>
) {
  const release = await commandRunnerMutex.acquire();
  setIsRunningCommand(true); // Update isRunningCommand
  setCurrentCommand(command); // Update currentCommand
  try {
    await invoke(command, parameters);
  } finally {
    release();
    setIsRunningCommand(false); // Update isRunningCommand
    setCurrentCommand(null); // Clear currentCommand
  }
}

export async function cancelCommand() {
  commandRunnerMutex.cancel();
  const release = await commandRunnerMutex.acquire();
  setIsRunningCommand(true); // Update isRunningCommand
  setCurrentCommand(null); // Clear currentCommand
  try {
    await invoke("cancel_command");
  } finally {
    release();
    setIsRunningCommand(false); // Update isRunningCommand
  }
}
