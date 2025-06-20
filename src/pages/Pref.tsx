import { Button, Input, Option, Select, Typography } from "@mui/joy";
import { PrefSetting } from "./Prefs";
import { useStore } from "../utilities/StoreContext";
import { useEffect, useState } from "react";

export interface PrefParams {
  setting: PrefSetting;
  pageName: string;
  storeExists: boolean;
}

export default ({ setting, pageName, storeExists }: PrefParams) => {
  if (setting.type === "info" && typeof setting.defaultValue === "function") {
    const [info, setInfo] = useState("");

    useEffect(() => {
      const fetchInfo = async () => {
        const result = await setting.defaultValue();
        setInfo(result);
      };
      fetchInfo();
    }, [setting.defaultValue]);

    return (
      <Typography level="body-md" className="prefs-setting">
        {setting.name}: {info}
      </Typography>
    );
  }
  const [value, setValue] = useStore(
    (pageName + "/" + setting.name).toLowerCase(),
    setting.defaultValue || ""
  );
  return (
    <div className="prefs-setting">
      {setting.type !== "button" && (
        <Typography level="body-md">{setting.name}</Typography>
      )}
      {setting.type === "text" && (
        <Input
          type="text"
          disabled={!storeExists}
          defaultValue={value}
          onChange={(e) => {
            setValue(e.target.value);
            if (setting.onChange) {
              setting.onChange(e.target.value);
            }
          }}
        />
      )}
      {setting.type === "select" && (
        <Select
          value={value}
          size="sm"
          disabled={!storeExists}
          onChange={(_, newValue) => {
            setValue(newValue);
            if (setting.onChange) {
              setting.onChange(newValue);
            }
          }}
        >
          {setting.options?.map((option) => (
            <Option key={option.value} value={option.value}>
              {option.label}
            </Option>
          ))}
        </Select>
      )}
      {setting.type === "checkbox" && (
        <Input
          type="checkbox"
          defaultChecked={value === "true"}
          disabled={!storeExists}
          onChange={(e) => {
            setValue(e.target.checked ? "true" : "false");
            if (setting.onChange) {
              setting.onChange(e.target.checked);
            }
          }}
        />
      )}
      {setting.type === "button" && (
        <Button
          variant="soft"
          disabled={!storeExists}
          onClick={() => {
            if (setting.onChange) {
              setting.onChange("");
            }
          }}
          sx={{ marginTop: "var(--padding-md)" }}
        >
          {setting.name}
        </Button>
      )}
      {setting.type === "info" && (
        <Typography
          level="body-sm"
          sx={{ color: "var(--joy-palette-neutral-500)" }}
        >
          {value}
        </Typography>
      )}
    </div>
  );
};
