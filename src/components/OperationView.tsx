import {
  Button,
  Input,
  Modal,
  ModalClose,
  ModalDialog,
  Typography,
} from "@mui/joy";
import { listen } from "@tauri-apps/api/event";
import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Operation } from "../utilities/operations";

interface OperationViewProps {
  operation_info: { operation: Operation; params: { [key: string]: any } };
}

export default ({ operation_info }: OperationViewProps) => {
  const operation = operation_info.operation;
  const params = operation_info.params;
  const id = useMemo<string>(() => operation.id + "_operation", [operation]);
  const [open, setOpen] = useState(true);

  const listenerAdded = useRef(false);
  const unlisten = useRef<() => void>(() => {});

  useEffect(() => {
    if (!listenerAdded.current) {
      (async () => {
        const unlistenFn = await listen(id, (event) => {});
        unlisten.current = unlistenFn;
      })();
      listenerAdded.current = true;
    }
    return () => {
      unlisten.current();
    };
  }, []);

  const startOperation = useCallback(async () => {
    invoke(id);
  }, []);

  return (
    <Modal
      open={open}
      onClose={() => {
        setOpen(false);
      }}
    >
      <ModalDialog>
        <ModalClose />
        <Typography level="h3">Can I put my balls in your jaws?</Typography>
      </ModalDialog>
    </Modal>
  );
};
