import "./Tile.css";

export interface TileProps {
  title: string;
  children: React.ReactNode;
}

export default ({ title, children }: TileProps) => {
  return (
    <div className={"tile"}>
      <div className={"tile-title"}>{title}</div>
      <div className={"tile-content"}>{children}</div>
    </div>
  );
};
