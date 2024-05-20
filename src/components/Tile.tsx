import { Typography } from "@mui/joy";
import "./Tile.css";

export interface TileProps {
  title?: string;
  children: React.ReactNode;
}

export default ({ title, children }: TileProps) => {
  return (
    <div className={"tile"}>
      {title != null && (
        <Typography level="body-xs" className={"tile-title"}>
          {title}
        </Typography>
      )}
      <div className={"tile-content"}>{children}</div>
    </div>
  );
};
