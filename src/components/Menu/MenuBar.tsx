import Menu from "@mui/joy/Menu";
import List from "@mui/joy/List";
import ListItem from "@mui/joy/ListItem";
import { useEffect, useRef, useState } from "react";
import MenuBarButton from "./MenuBarButton";
import MenuGroup, { MenuBarData } from "./MenuGroup";
import { Shortcut, acceleratorPresssed } from "../../Shortcut";

const bar = [
  {
    label: "File",
    items: [
      {
        label: "New",
        items: [
          {
            name: "New File...",
            shortcut: "Ctrl+N",
            callback: () => {
              console.log("New File!");
            },
          },
          {
            name: "New Project...",
            callback: () => {
              console.log("New Project!");
            },
          },
        ],
      },
      {
        label: "Open",
        items: [
          {
            name: "Open File...",
            shortcut: "Ctrl+O",
            callback: () => {
              console.log("Open File!");
            },
          },
          {
            name: "Open Workspace...",
            callback: () => {
              console.log("Open Workspace!");
            },
          },
        ],
      },
      {
        label: "Save",
        items: [
          {
            name: "Save",
            shortcut: "Ctrl+S",
            callbackName: "save",
          },
          {
            name: "Save As...",
            shortcut: "Ctrl+Shift+S",
            callback: () => {
              console.log("Save As!");
            },
          },
        ],
      },
    ],
  },
  {
    label: "Edit",
    items: [
      {
        label: "Timeline",
        items: [
          {
            name: "Undo",
            shortcut: "Ctrl+Z",
            callback: () => {
              console.log("Undo!");
            },
          },
          {
            name: "Redo",
            shortcut: "Ctrl+Shift+Z",
            callback: () => {
              console.log("Redo!");
            },
          },
        ],
      },
    ],
  },
  {
    label: "View",
    items: [],
  },
  {
    label: "Help",
    items: [],
  },
] as MenuBarData;

export interface MenuBarProps {
  callbacks: Record<string, () => void>;
}
export default function MenuBar({ callbacks }: MenuBarProps) {
  const menus = useRef<Array<HTMLButtonElement>>([]);
  const [menuIndex, setMenuIndex] = useState<null | number>(null);

  useEffect(() => {
    const items: { shortcut: Shortcut; callback: () => void }[] = [];

    for (const menu of bar) {
      for (const group of menu.items) {
        for (const item of group.items) {
          if (item.shortcut) {
            const shortcut = Shortcut.fromString(item.shortcut);
            let callback;
            if (item.callbackName !== undefined) {
              callback = callbacks[item.callbackName];
            } else {
              callback = item.callback ?? (() => {});
            }
            items.push({
              shortcut,
              callback,
            });
          }
        }
      }
    }

    const handleGlobalKeyDown = (event: KeyboardEvent) => {
      if (!acceleratorPresssed(event)) return;

      for (const item of items) {
        if (item.shortcut.pressed(event)) {
          event.preventDefault();
          item.callback();
        }
      }
    };

    document.addEventListener("keydown", handleGlobalKeyDown);

    return () => {
      document.removeEventListener("keydown", handleGlobalKeyDown);
    };
  }, [bar, callbacks]);

  const openNextMenu = () => {
    if (typeof menuIndex === "number") {
      if (menuIndex === menus.current.length - 1) {
        setMenuIndex(0);
      } else {
        setMenuIndex(menuIndex + 1);
      }
    }
  };

  const openPreviousMenu = () => {
    if (typeof menuIndex === "number") {
      if (menuIndex === 0) {
        setMenuIndex(menus.current.length - 1);
      } else {
        setMenuIndex(menuIndex - 1);
      }
    }
  };

  const handleKeyDown = (event: React.KeyboardEvent) => {
    if (event.key === "ArrowRight") {
      openNextMenu();
    }
    if (event.key === "ArrowLeft") {
      openPreviousMenu();
    }
  };

  const createHandleButtonKeyDown =
    (index: number) => (event: React.KeyboardEvent) => {
      if (event.key === "ArrowRight") {
        if (index === menus.current.length - 1) {
          menus.current[0]?.focus();
        } else {
          menus.current[index + 1]?.focus();
        }
      }
      if (event.key === "ArrowLeft") {
        if (index === 0) {
          menus.current[menus.current.length]?.focus();
        } else {
          menus.current[index - 1]?.focus();
        }
      }
    };

  return (
    <List
      size="sm"
      orientation="horizontal"
      aria-label="YCode menu bar"
      role="menubar"
      data-joy-color-scheme="dark"
      sx={{
        bgcolor: "background.body",
        borderRadius: "4px",
        width: "100%",
      }}
    >
      {bar &&
        bar.map((menu, index) => (
          <ListItem key={index}>
            <MenuBarButton
              open={menuIndex === index}
              onOpen={() => {
                setMenuIndex((prevMenuIndex) =>
                  prevMenuIndex === null ? index : null
                );
              }}
              onKeyDown={createHandleButtonKeyDown(1)}
              onMouseEnter={() => {
                if (typeof menuIndex === "number") {
                  setMenuIndex(index);
                }
              }}
              ref={(instance) => {
                menus.current[index] = instance!;
              }}
              menu={
                <Menu
                  size="sm"
                  onClose={() => {
                    menus.current[index]?.focus();
                  }}
                >
                  <MenuGroup
                    handleKeyDown={handleKeyDown}
                    resetMenuIndex={() => setMenuIndex(null)}
                    groups={menu.items}
                    callbacks={callbacks}
                  />
                </Menu>
              }
            >
              {menu.label}
            </MenuBarButton>
          </ListItem>
        ))}
    </List>
  );
}
