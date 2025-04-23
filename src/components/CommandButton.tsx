import { Button } from "@mui/joy";
import { useCommandRunner } from "../utilities/Command";

export interface CommandButtonProps {
  command: string;
  parameters?: Record<string, unknown>;
  label?: string;
  icon: React.ReactNode;
  variant?: "plain" | "outlined" | "soft" | "solid";
  sx?: React.CSSProperties;
}

export default function CommandButton({
  command,
  parameters,
  label,
  icon,
  variant,
  sx = {},
}: CommandButtonProps) {
  const { isRunningCommand, currentCommand, runCommand, cancelCommand } =
    useCommandRunner();

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
