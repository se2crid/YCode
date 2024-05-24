import { Button } from "@mui/joy";
import {
  cancelCommand,
  runCommand,
  useCommandRunner,
} from "../utilities/Command";

export interface CommandButtonProps {
  command: string;
  parameters?: Record<string, unknown>;
  label: string;
  icon: React.ReactNode;
}

export default function CommandButton({
  command,
  parameters,
  label,
  icon,
}: CommandButtonProps) {
  const { isRunningCommand, currentCommand } = useCommandRunner();

  return (
    <Button
      disabled={isRunningCommand && currentCommand !== command}
      startDecorator={icon}
      loading={isRunningCommand && currentCommand === command}
      size="md"
      sx={{ marginRight: "10px" }}
      onClick={() => {
        if (isRunningCommand) {
          if (currentCommand === command) {
            cancelCommand();
          }
          return;
        }
        runCommand(command, parameters);
      }}
    >
      {label}
    </Button>
  );
}
