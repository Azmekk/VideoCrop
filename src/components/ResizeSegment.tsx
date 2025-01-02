import { Checkbox, Input, InputNumber } from "antd";
import type { VideoCropPoints, VideoInfo } from "../Logic/Interfaces";
import { useState } from "react";

interface ResizeSegmentProps {
  videoInfo: VideoInfo | undefined;
  disabled: boolean;
}
function CropSegment(props: ResizeSegmentProps) {
  const [segmentEnabled, setSegmentEnabled] = useState(false);

  return (
    <div className={props.disabled ? "disabled" : ""}>
      <div style={{ marginBottom: "25px", display: "flex", gap: "10px" }}>
        <div style={{ fontSize: "1.2em", fontWeight: "bold" }}>Resize</div>
        <Checkbox defaultChecked={false} onChange={(e) => setSegmentEnabled(e.target.checked)} />
      </div>

      <div
        className={segmentEnabled ? "" : "disabled"}
        style={{ display: "flex", gap: "5px", flexDirection: "column" }}
      >
        <div
          style={{
            display: "flex",
            gap: "5px",
            alignItems: "center",
            justifyContent: "space-between",
            width: "150px",
          }}
        >
          <div>Width:</div>
          <InputNumber key={props.videoInfo?.width} defaultValue={props.videoInfo?.width} placeholder="1920" />
        </div>
        <div
          style={{
            display: "flex",
            gap: "5px",
            alignItems: "center",
            justifyContent: "space-between",
            width: "150px",
          }}
        >
          <div>Height:</div>
          <InputNumber
            key={props.videoInfo?.height}
            defaultValue={props.videoInfo?.height}
            title="height"
            placeholder="1080"
          />
        </div>
      </div>
    </div>
  );
}

export default CropSegment;
