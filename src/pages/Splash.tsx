import { CircularProgress } from "@mui/joy";
import "./Splash.css";

export default () => {
  return (
    <div className="splash">
      <CircularProgress />
      <div className="splash-text">Loading YCode...</div>
    </div>
  );
};
