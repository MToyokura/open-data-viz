import { Alert, Autocomplete, Button, TextField } from "@mui/material";
import { useState } from "react";
import {
  useCommodityCandidates,
  useRegionCandidates,
} from "../hooks/useAutoselectorCandidates";

export const Selectors = (props: {
  onClickFetch(fileName: string): void;
  onClickReset: () => void;
  dataNotFound: boolean;
  onCloseAlert: () => void;
}) => {
  const [commodityCode, setCommodityCode] = useState("");
  const [regionCode, setRegionCode] = useState("");

  const commoditySelections = useCommodityCandidates();
  const regionSelections = useRegionCandidates();

  return (
    <div>
      <div
        style={{
          display: "flex",
          justifyContent: "center",
          gap: "1rem",
          flexWrap: "wrap",
        }}
      >
        <Autocomplete
          disablePortal
          options={
            commoditySelections.responseCommodityCandidates
              ? commoditySelections.responseCommodityCandidates
              : []
          }
          renderInput={(params) => {
            return <TextField {...params} label="品目" />;
          }}
          sx={{ width: "20rem" }}
          onChange={(event, value) => {
            if (value) {
              setCommodityCode(value.code);
            }
          }}
        />
        <Autocomplete
          disablePortal
          options={
            regionSelections.responseRegionCandidates
              ? regionSelections.responseRegionCandidates
              : []
          }
          renderInput={(params) => {
            return <TextField {...params} label="地域" />;
          }}
          sx={{ width: "20rem" }}
          onChange={(event, value) => {
            if (value) {
              setRegionCode(value.code);
            }
          }}
        />
      </div>
      {props.dataNotFound && (
        <div
          style={{
            display: "flex",
            justifyContent: "center",
            margin: "1rem",
          }}
        >
          <Alert severity="error" onClose={props.onCloseAlert}>
            該当データが見つかりませんでした。
          </Alert>
        </div>
      )}
      <div
        style={{
          display: "flex",
          justifyContent: "center",
          gap: "1rem",
          flexWrap: "wrap",
          margin: "1rem",
        }}
      >
        <Button
          style={{ width: "20em" }}
          variant="contained"
          disableElevation
          onClick={() => {
            if (commodityCode && regionCode) {
              props.onClickFetch(`0003421913_${commodityCode}_${regionCode}`);
            }
          }}
        >
          追加
        </Button>
        <Button
          variant="outlined"
          disableElevation
          onClick={() => props.onClickReset()}
        >
          リセット
        </Button>
      </div>
    </div>
  );
};
