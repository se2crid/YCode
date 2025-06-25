import { useParams } from "react-router";
import "./Prefs.css";

import { Divider, Typography, Link } from "@mui/joy";
import { Link as RouterLink } from "react-router-dom";
import { Fragment, useContext, useEffect } from "react";
import { StoreContext } from "../utilities/StoreContext";
import { preferenceRegistry } from "./pages";
import PreferenceItemRenderer from "./PreferenceItemRenderer";

export default function Preferences() {
  const { page } = useParams<"page">();
  const { store } = useContext(StoreContext);
  const storeExists = store !== null && store !== undefined;

  const categories = preferenceRegistry.getAllCategories();
  const currentPage = page ? preferenceRegistry.getPage(page) : null;

  // Call onLoad when page changes
  useEffect(() => {
    if (currentPage?.onLoad) {
      currentPage.onLoad();
    }
  }, [currentPage]);

  return (
    <div className="prefs-container">
      <div className="prefs-sidebar-container">
        <div className="prefs-sidebar">
          {categories.map((category, categoryIndex) => (
            <Fragment key={category.id}>
              <Typography
                level="title-sm"
                sx={{
                  padding: "var(--padding-xs) 0",
                  color: "var(--joy-palette-neutral-600)",
                  textTransform: "uppercase",
                  fontSize: "0.75rem",
                  fontWeight: 600,
                }}
              >
                {category.name}
              </Typography>
              {category.pages.map((p) => (
                <Link
                  level="body-sm"
                  className="prefs-sidebar-item"
                  key={p.id}
                  component={RouterLink}
                  to={`/preferences/${p.id}`}
                  sx={{
                    textDecoration: page === p.id ? "underline" : "none",
                    color: "inherit",
                    padding: "var(--padding-xs)",
                    marginTop: "2px",
                    fontWeight: page === p.id ? 600 : 400,
                  }}
                >
                  {p.name}
                </Link>
              ))}
              {categoryIndex !== categories.length - 1 && (
                <Divider orientation="horizontal" />
              )}
            </Fragment>
          ))}
        </div>
        <Divider orientation="vertical" />
      </div>

      <div className="prefs-content">
        {currentPage ? (
          <>
            <div className="prefs-page-header">
              <Typography level="title-lg">{currentPage.name}</Typography>
              {currentPage.description && (
                <Typography level="body-sm">
                  {currentPage.description}
                </Typography>
              )}
            </div>

            <div className="prefs-page-content">
              {currentPage.customComponent ? (
                <currentPage.customComponent />
              ) : (
                currentPage.items?.map((item) => (
                  <PreferenceItemRenderer
                    key={item.id}
                    item={item}
                    storeExists={storeExists}
                    pageName={currentPage.id}
                  />
                ))
              )}
            </div>
          </>
        ) : page ? (
          <Typography level="h2">Page Not Found</Typography>
        ) : (
          <div className="prefs-welcome">
            <Typography level="title-lg">Preferences</Typography>
            <Typography level="body-md" sx={{ marginTop: "var(--padding-sm)" }}>
              Select a category from the sidebar to configure your preferences.
            </Typography>
          </div>
        )}
      </div>
    </div>
  );
}
