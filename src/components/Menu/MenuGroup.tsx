import { List, ListDivider, ListItem, MenuItem, Typography } from "@mui/joy";

import { Fragment } from "react/jsx-runtime";

type MenuItem = {
  name: string;
  shortcut?: string;
  callback?: () => void;
  callbackName?: string;
};

type MenuGroup = {
  label: string;
  items: MenuItem[];
};

export type MenuBarData = {
  label: string;
  items: MenuGroup[];
}[];

export interface MenuGroupProps {
  groups: MenuGroup[];
  handleKeyDown: (event: React.KeyboardEvent) => void;
  resetMenuIndex: () => void;
  callbacks: Record<string, () => void>;
}

const renderShortcut = (text: string) => (
  <Typography level="body-sm" textColor="text.tertiary" ml="auto">
    {text}
  </Typography>
);

const MenuGroup: React.FC<MenuGroupProps> = ({
  groups,
  handleKeyDown,
  callbacks,
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
                    let callback;
                    if (item.callbackName !== undefined) {
                      callback = callbacks[item.callbackName];
                    } else {
                      callback = item.callback ?? (() => {});
                    }
                    callback();
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
