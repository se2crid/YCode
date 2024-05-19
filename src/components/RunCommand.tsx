// Create a simple component to display text in a console-style in a mui joy modal.

import { Modal, ModalClose, ModalDialog, Typography } from "@mui/joy";
import { invoke } from "@tauri-apps/api";
import { listen } from "@tauri-apps/api/event";
import { useEffect, useRef, useState } from "react";
import Convert from "ansi-to-html";
import "./RunCommand.css";

const convert = new Convert();

interface RunCommandProps {
  title: string;
  failedMessage?: string;
  doneMessage?: string;
  command: string;
  listener: string;
  run: boolean;
  setRun: (run: boolean) => void;
}

export default ({
  title,
  command,
  listener,
  run,
  setRun,
  failedMessage,
  doneMessage,
}: RunCommandProps) => {
  const [open, setOpen] = useState(false);
  const [body, setBody] = useState("");
  const [html, setHtml] = useState("");
  const [status, setStatus] = useState("none");

  const preRef = useRef<HTMLPreElement | null>(null);
  const listenerAdded = useRef(false);
  const hasRun = useRef(false);

  useEffect(() => {
    if (run && !hasRun.current) {
      setOpen(true);
      setStatus("running");
      invoke(command);
      hasRun.current = true;
    }
  }, [run]);

  useEffect(() => {
    if (!listenerAdded.current) {
      listen(listener, (event) => {
        let line = event.payload as string;
        if (line.includes("command.done")) {
          if (line.split(".")[2] !== "0") {
            setStatus("failed");
            return;
          }
          setStatus("done");
          return;
        }
        setBody((body) => body + line + "\n");
      });
      listenerAdded.current = true;
    }
  }, []);

  useEffect(() => {
    if (open) {
      if (body.startsWith("\n")) {
        setHtml(convert.toHtml(body.slice(1)));
      } else {
        setHtml(convert.toHtml(body));
      }
    }
  }, [body]);

  useEffect(() => {
    if (preRef.current) {
      const element = preRef.current;
      setTimeout(() => {
        element.scrollIntoView({ behavior: "smooth", block: "end" });
      }, 0);
    }
  }, [html]);

  return (
    <Modal
      open={open}
      onClose={
        status === "done" || status === "failed"
          ? () => {
              setOpen(false);
              setRun(false);
              setBody("");
              setHtml("");
              setStatus("none");
              hasRun.current = false;
            }
          : () => {}
      }
    >
      <ModalDialog>
        {(status === "done" || status === "failed") && <ModalClose />}
        <Typography level="h2">
          {status === "failed"
            ? failedMessage ?? "Failed"
            : status === "done"
            ? doneMessage ?? "Done"
            : title}
        </Typography>
        <div className="console">
          <pre ref={preRef} dangerouslySetInnerHTML={{ __html: html }}></pre>
        </div>
      </ModalDialog>
    </Modal>
  );
};
