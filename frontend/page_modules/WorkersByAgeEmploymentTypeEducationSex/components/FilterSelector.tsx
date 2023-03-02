import {
  FormControl,
  InputLabel,
  MenuItem,
  Select,
  SelectChangeEvent,
} from "@mui/material";

export const FilterSelector = (props: {
  label: string;
  currentValue: string; // TODO: generics で型渡せる？
  selectOptionValues: string[];
  onChange: (event: SelectChangeEvent) => void;
}) => {
  return (
    <FormControl>
      <InputLabel>{props.label}</InputLabel>
      <Select
        sx={{ width: "10rem" }}
        value={props.currentValue}
        onChange={props.onChange}
      >
        {props.selectOptionValues.map((item) => {
          return (
            <MenuItem key={item} value={item}>
              {item}
            </MenuItem>
          );
        })}
      </Select>
    </FormControl>
  );
};
