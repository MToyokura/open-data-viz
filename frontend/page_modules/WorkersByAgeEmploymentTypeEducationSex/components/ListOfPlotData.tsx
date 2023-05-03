import CloseIcon from "@mui/icons-material/Close";
import { PlotData } from "../types";

import {
  Table,
  TableBody,
  TableCell,
  TableContainer,
  TableHead,
  TableRow,
} from "@mui/material";

export const ListOfPlotData = (props: {
  plotData: PlotData[];
  onClickDeletePlotData: (shownRowdata: PlotData) => void;
}) => {
  return (
    <div>
      <div
        style={{
          display: "flex",
          justifyContent: "center",
          margin: "3rem 1rem 1rem 1rem",
        }}
      >
        表示されているデータ
      </div>
      <div
        style={{
          display: "flex",
          justifyContent: "center",
        }}
      >
        <div
          style={{
            border: "1px solid rgba(0, 0, 0, 0.2) ",
            padding: "1rem",
          }}
        >
          <TableContainer>
            <Table>
              <TableHead>
                <TableRow>
                  <TableCell style={{ fontWeight: "bold" }}>
                    {"年齢階級"}
                  </TableCell>
                  <TableCell style={{ fontWeight: "bold" }}>
                    {"雇用形態"}
                  </TableCell>
                  <TableCell style={{ fontWeight: "bold" }}>{"教育"}</TableCell>
                  <TableCell style={{ fontWeight: "bold" }}>{"性別"}</TableCell>
                  <TableCell style={{ fontWeight: "bold" }}></TableCell>
                </TableRow>
              </TableHead>
              <TableBody>
                {props.plotData.map((shownRowdata) => {
                  return (
                    <TableRow key={shownRowdata.index}>
                      <TableCell>
                        {shownRowdata["年齢階級(詳細集計）"]}
                      </TableCell>
                      <TableCell>{shownRowdata["雇用形態"]}</TableCell>
                      <TableCell>{shownRowdata["教育"]}</TableCell>
                      <TableCell>{shownRowdata["性別"]}</TableCell>
                      <TableCell>
                        <div
                          style={{
                            display: "flex",
                            justifyContent: "center",
                            alignItems: "center",
                            width: "5rem",
                          }}
                        >
                          <CloseIcon
                            sx={{ color: "rgba(0, 0, 0, 0.5)" }}
                            className="icon_button"
                            onClick={() => {
                              props.onClickDeletePlotData(shownRowdata);
                            }}
                          />
                        </div>
                      </TableCell>
                    </TableRow>
                  );
                })}
              </TableBody>
            </Table>
          </TableContainer>
        </div>
      </div>
    </div>
  );
};
