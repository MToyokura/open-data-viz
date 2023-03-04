import {
  Add,
  KeyboardArrowLeft,
  KeyboardArrowRight,
  Remove,
} from "@mui/icons-material";
import { Slider } from "@mui/material";
import Autocomplete from "@mui/material/Autocomplete";
import TextField from "@mui/material/TextField";
import { vegetableSelect, yearMonthLabel } from "../constants";
import { QueryParams } from "../types/QueryParams";

import { Dispatch, SetStateAction } from "react";
import { AmountPerPrefectureOfMonth } from "../types/AmountPerPrefectureOfMonth";

export const ControlPanel = (props: {
  queryParams: QueryParams;
  setQueryParams: Dispatch<SetStateAction<QueryParams>>;
  yearMonth: AmountPerPrefectureOfMonth[];
}) => {
  return (
    <div style={{ width: "100%" }}>
      <div style={{ display: "flex", justifyContent: "center" }}>
        {yearMonthLabel[props.queryParams.yearMonthIndex]}
      </div>
      <div style={{ display: "flex", justifyContent: "center" }}>
        <div style={{ display: "flex", alignItems: "center", width: "20rem" }}>
          <KeyboardArrowLeft
            className="icon_button"
            onClick={() => {
              if (props.queryParams.yearMonthIndex > 0) {
                props.setQueryParams({
                  ...props.queryParams,
                  yearMonthIndex: props.queryParams.yearMonthIndex - 1,
                });
              }
            }}
          />
          <Slider
            style={{ margin: "0rem 1rem" }}
            min={0}
            max={props.yearMonth.length - 1}
            value={props.queryParams.yearMonthIndex}
            onChange={(event, newValue) => {
              props.setQueryParams({
                ...props.queryParams,
                yearMonthIndex: newValue as number,
              });
            }}
          />
          <KeyboardArrowRight
            className="icon_button"
            onClick={() => {
              if (
                props.queryParams.yearMonthIndex <
                props.yearMonth.length - 1
              ) {
                props.setQueryParams({
                  ...props.queryParams,
                  yearMonthIndex: props.queryParams.yearMonthIndex + 1,
                });
              }
            }}
          />
        </div>
      </div>
      <div
        style={{
          display: "flex",
          justifyContent: "center",
          margin: "1rem",
        }}
      >
        <div style={{ display: "flex", alignItems: "center" }}>
          <Remove
            className="icon_button"
            onClick={() => {
              if (props.queryParams.circleSize > 1) {
                props.setQueryParams({
                  ...props.queryParams,
                  circleSize: props.queryParams.circleSize - 1,
                });
              }
            }}
          />
          <TextField
            label="半径"
            size="small"
            style={{ margin: "0rem 1rem" }}
            // https://mui.com/material-ui/react-text-field/#type-quot-number-quot
            inputProps={{ inputMode: "numeric", pattern: "[0-9]*" }}
            sx={{ width: "4rem" }}
            value={props.queryParams.circleSize}
            onChange={(event) => {
              const value = Number(event.target.value);
              if (value > 0) {
                props.setQueryParams({
                  ...props.queryParams,
                  circleSize: value,
                });
              }
            }}
          />
          <Add
            className="icon_button"
            onClick={() => {
              props.setQueryParams({
                ...props.queryParams,
                circleSize: props.queryParams.circleSize + 1,
              });
            }}
          />
        </div>
      </div>
      <div style={{ display: "flex", justifyContent: "center" }}>
        <Autocomplete
          size="small"
          disablePortal
          id="combo-box-demo"
          options={vegetableSelect}
          sx={{ width: "12rem" }}
          renderInput={(params) => <TextField {...params} label="品目" />}
          value={props.queryParams.vegetable}
          onChange={(event, value) => {
            if (value) {
              props.setQueryParams({ ...props.queryParams, vegetable: value });
            }
          }}
        />
      </div>
    </div>
  );
};
