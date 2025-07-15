import {
  Accordion,
  AccordionDetails,
  AccordionSummary,
  Divider,
  Modal,
  ModalClose,
  ModalDialog,
  Typography,
} from "@mui/joy";
import { OperationState } from "../utilities/operations";
import "./OperationView.css";
import { SuccessIcon, ErrorIcon, StyledLoadingIcon } from "react-toast-plus";
import { PanoramaFishEye } from "@mui/icons-material";

export default ({
  operationState,
  closeMenu,
}: {
  operationState: OperationState;
  closeMenu: () => void;
}) => {
  const operation = operationState.current;
  const failed = operationState.failed.length > 0;
  const done =
    failed || operationState.completed.length == operation.steps.length;

  return (
    <Modal
      open={true}
      onClose={() => {
        if (done) closeMenu();
      }}
    >
      <ModalDialog>
        {done && <ModalClose />}
        <div>
          <Typography level="h3">{operation?.title}</Typography>
          <Typography level="body-lg">
            {done
              ? failed
                ? "Operation failed. Please see steps for details."
                : "Operation completed!"
              : "Please wait..."}
          </Typography>
        </div>
        <Divider />
        <div className="operation-content">
          {operation.steps.map((step) => {
            let failed = operationState.failed.find((f) => f.stepId == step.id);
            let completed = operationState.completed.includes(step.id);
            let started = operationState.started.includes(step.id);
            let notStarted = !failed && !completed && !started;
            return (
              <div className="operation-step">
                <div className="operation-step-icon">
                  {failed && <ErrorIcon />}
                  {!failed && completed && <SuccessIcon />}
                  {!failed && !completed && started && <StyledLoadingIcon />}
                  {notStarted && (
                    <PanoramaFishEye
                      sx={{
                        width: "100%",
                        height: "100%",
                        color: "neutral.500",
                      }}
                    />
                  )}
                </div>

                <div className="operation-step-internal">
                  <Typography
                    textColor={notStarted ? "neutral.500" : undefined}
                  >
                    {step.title}
                  </Typography>
                  {failed && (
                    <Accordion sx={{ marginTop: 0 }}>
                      <AccordionSummary>
                        <Typography level="body-sm">Show Details</Typography>
                      </AccordionSummary>
                      <AccordionDetails>
                        <pre className="operation-extra-details">
                          {failed.extraDetails}
                        </pre>
                      </AccordionDetails>
                    </Accordion>
                  )}
                </div>
              </div>
            );
          })}
        </div>
      </ModalDialog>
    </Modal>
  );
};
