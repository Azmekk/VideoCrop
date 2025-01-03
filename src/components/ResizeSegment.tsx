import { Checkbox, Input, InputNumber } from "antd";
import type { VideoCropPoints, VideoInfo } from "../Logic/Interfaces";
import { useEffect, useState } from "react";

interface ResizeSegmentProps {
  videoInfo: VideoInfo | undefined;
  disabled: boolean;
  videoNotCropped: boolean;
  onChange: (x: { width: number; height: number }, enabled: boolean) => void;
}
function CropSegment(props: ResizeSegmentProps) {
  const [segmentEnabled, setSegmentEnabled] = useState(false);
  const [videoDimensions, setVideoDimensions] = useState({
    width: props.videoInfo?.width ?? 0,
    height: props.videoInfo?.height ?? 0,
  });

  useEffect(() => {
    props.onChange(videoDimensions, segmentEnabled && props.videoNotCropped);
  }, [videoDimensions, segmentEnabled, props.videoNotCropped]);

  useEffect(() => {
    setVideoDimensions({
      width: props.videoInfo?.width ?? 0,
      height: props.videoInfo?.height ?? 0,
    });
  }, [props.videoInfo]);

  return (
    <div className={props.disabled || !props.videoNotCropped ? "disabled" : ""}>
      <div style={{ display: "flex", gap: "10px" }}>
        <div style={{ fontSize: "1.2em", fontWeight: "bold" }}>Resize</div>
        <Checkbox defaultChecked={false} onChange={(e) => setSegmentEnabled(e.target.checked)} />
      </div>

      {!props.videoNotCropped && segmentEnabled && (
        <div style={{ color: "red", fontSize: "0.8em" }}>Cannot resize if crop is enabled.</div>
      )}

      <div
        className={segmentEnabled ? "" : "disabled"}
        style={{ display: "flex", gap: "5px", flexDirection: "column", marginTop: "25px" }}
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
          <InputNumber
            key={props.videoInfo?.width}
            defaultValue={props.videoInfo?.width}
            placeholder="1920"
            onChange={(value) => {
              setVideoDimensions({ width: value ?? 0, height: videoDimensions.height });
            }}
          />
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
            onChange={(value) => {
              setVideoDimensions({ width: videoDimensions.width, height: value ?? 0 });
            }}
          />
        </div>
      </div>
    </div>
  );
}

export default CropSegment;
