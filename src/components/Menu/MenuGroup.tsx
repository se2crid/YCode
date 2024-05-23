import { List, ListDivider, ListItem, MenuItem, Typography } from "@mui/joy";

import { Fragment } from "react/jsx-runtime";

type MenuItem = {
  name: string;
  shortcut?: string;
  callback: () => void;
};

type MenuGroup = {
  label: string;
  items: MenuItem[];
};

export interface MenuGroupProps {
  groups: MenuGroup[];
  handleKeyDown: (event: React.KeyboardEvent) => void;
  resetMenuIndex: () => void;
}

const renderShortcut = (text: string) => (
  <Typography level="body-sm" textColor="text.tertiary" ml="auto">
    {text}
  </Typography>
);

const MenuGroup: React.FC<MenuGroupProps> = ({
  groups,
  handleKeyDown,
  resetMenuIndex,
}) => {
  return (
    <>
      {groups.map((group, groupIndex) => (
        <Fragment key={groupIndex}>
          <ListItem nested>
            <List aria-label={group.label}>
              {group.items.map((item, itemIndex) => (
                <MenuItem
                  key={itemIndex}
                  onClick={() => {
                    resetMenuIndex();
                    item.callback();
                  }}
                  onKeyDown={handleKeyDown}
                >
                  {item.name} {item.shortcut && renderShortcut(item.shortcut)}
                </MenuItem>
              ))}
            </List>
          </ListItem>
          {groupIndex < groups.length - 1 && <ListDivider />}
        </Fragment>
      ))}
    </>
  );
};

export default MenuGroup;
