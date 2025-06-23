import { Button } from "@mui/joy";
import { useCommandRunner } from "../utilities/Command";
import { useIDE } from "../utilities/IDEContext";

export interface CommandButtonProps {
  command: string;
  parameters?: Record<string, unknown>;
  label?: string;
  icon: React.ReactNode;
  variant?: "plain" | "outlined" | "soft" | "solid";
  sx?: React.CSSProperties;
  clearConsole?: boolean;
  validate?: () => boolean;
}

export default function CommandButton({
  command,
  parameters,
  label,
  icon,
  variant,
  sx = {},
  clearConsole = true,
  validate = () => true,
}: CommandButtonProps) {
  const { isRunningCommand, currentCommand, runCommand, cancelCommand } =
    useCommandRunner();
  const { setConsoleLines } = useIDE();

  return (
    <Button
      disabled={isRunningCommand && currentCommand !== command}
      startDecorator={label == "" || label == undefined ? undefined : icon}
      loading={isRunningCommand && currentCommand === command}
      variant={variant}
      size="md"
      sx={{
        marginRight: "var(--padding-md)",
        padding: "0 var(--padding-md)",
        ...sx,
      }}
      onClick={() => {
        if (!validate()) {
          return;
        }
        if (clearConsole) {
          setConsoleLines([]);
        }
        if (isRunningCommand) {
          if (currentCommand === command) {
            cancelCommand();
          }
          return;
        }
        runCommand(command, parameters);
      }}
    >
      {label == "" || label == undefined ? icon : label}
    </Button>
  );
}
