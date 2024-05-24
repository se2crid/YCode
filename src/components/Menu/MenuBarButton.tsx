import {
  Dropdown,
  DropdownProps,
  ListItemButton,
  MenuButton,
  Theme,
  menuItemClasses,
  typographyClasses,
} from "@mui/joy";
import { cloneElement, forwardRef } from "react";

type MenuBarButtonProps = Pick<DropdownProps, "children" | "open"> & {
  onOpen: DropdownProps["onOpenChange"];
  onKeyDown: React.KeyboardEventHandler;
  menu: JSX.Element;
  onMouseEnter: React.MouseEventHandler;
};

export default forwardRef(
  (
    { children, menu, open, onOpen, onKeyDown, ...props }: MenuBarButtonProps,
    ref: React.ForwardedRef<HTMLButtonElement>
  ) => {
    return (
      <Dropdown open={open} onOpenChange={onOpen}>
        <MenuButton
          {...props}
          slots={{ root: ListItemButton }}
          ref={ref}
          role="menuitem"
          variant={open ? "soft" : "plain"}
        >
          {children}
        </MenuButton>
        {cloneElement(menu, {
          slotProps: {
            listbox: {
              id: `toolbar-example-menu-${children}`,
              "aria-label": children,
            },
          },
          placement: "bottom-start",
          disablePortal: false,
          variant: "soft",
          sx: (theme: Theme) => ({
            overflowX: "hidden",
            width: 288,
            boxShadow: "0 2px 8px 0px rgba(0 0 0 / 0.38)",
            "--List-padding": "var(--ListDivider-gap)",
            "--ListItem-minHeight": "32px",
            [`&& .${menuItemClasses.root}`]: {
              transition: "none",
              "&:hover": {
                ...theme.variants.solid.primary,
                [`& .${typographyClasses.root}`]: {
                  color: "inherit",
                },
              },
            },
          }),
        })}
      </Dropdown>
    );
  }
);
