import { prefectures } from "../constants";
import { MousePositionType } from "../types/MousePosition";

export const HoverLabel = (props: {
  mousePosition: MousePositionType;
  mouseOverPrefecture: (typeof prefectures)[number];
  prefectureAmount: number;
}) => {
  return (
    // ホバー時に数量を表示する
    // https://developer.mozilla.org/en-US/docs/Web/API/MouseEvent/pageX
    // https://stackoverflow.com/questions/22549505/make-css-tooltip-follow-cursor
    <div
      style={{
        position: "absolute",
        top: `${props.mousePosition.pageY - 10}px`,
        left: `${props.mousePosition.pageX + 30}px`,
        border: "solid black 1px",
        borderRadius: "10px",
        backgroundColor: "#fcfcfc",
        padding: "5px 8px",
      }}
    >
      <div
        style={{
          display: "flex",
          alignItems: "center",
          justifyContent: "center",
        }}
      >
        <div>{props.mouseOverPrefecture}：</div>
        <div style={{ marginLeft: "5px" }}>{props.prefectureAmount}</div>
        <div style={{ marginLeft: "5px" }}>t</div>
      </div>
    </div>
  );
};
