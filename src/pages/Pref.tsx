import { Typography } from "@mui/joy";
import { PrefSetting } from "./Prefs";
import { useStore } from "../utilities/StoreContext";

export interface PrefParams {
  setting: PrefSetting;
  pageName: string;
  storeExists: boolean;
}

export default ({ setting, pageName, storeExists }: PrefParams) => {
  const [value, setValue] = useStore(
    (pageName + "/" + setting.name).toLowerCase(),
    setting.defaultValue || ""
  );
  return (
    <div className="prefs-setting">
      <Typography level="body-sm">{setting.name}</Typography>
      {setting.type === "text" && (
        <input
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
        <select
          value={value}
          disabled={!storeExists}
          onChange={(e) => {
            setValue(e.target.value);
            if (setting.onChange) {
              setting.onChange(e.target.value);
            }
          }}
        >
          {setting.options?.map((option) => (
            <option key={option.value} value={option.value}>
              {option.label}
            </option>
          ))}
        </select>
      )}
      {setting.type === "checkbox" && (
        <input
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
    </div>
  );
};
