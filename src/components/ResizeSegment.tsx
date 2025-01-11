import { Checkbox, InputNumber } from "antd";
import type { VideoInfo } from "../Logic/Interfaces/Interfaces";
import { useEffect, useState } from "react";
import { LockOutlined, UnlockFilled, UnlockOutlined } from "@ant-design/icons";

interface ResizeSegmentProps {
  videoInfo: VideoInfo | undefined;
  disabled: boolean;
  videoNotCropped: boolean;
  onChange: (x: { width: number; height: number }, enabled: boolean) => void;
}
function ResizeSegment(props: ResizeSegmentProps) {
  const [segmentEnabled, setSegmentEnabled] = useState(false);
  const [videoDimensions, setVideoDimensions] = useState({
    width: props.videoInfo?.width ?? 0,
    height: props.videoInfo?.height ?? 0,
  });

  const [resizeRatioLocked, setResizeRatioLocked] = useState(true);

  useEffect(() => {
    props.onChange(videoDimensions, segmentEnabled && props.videoNotCropped);
  }, [videoDimensions, segmentEnabled, props.videoNotCropped]);

  useEffect(() => {
    setVideoDimensions({
      width: props.videoInfo?.width ?? 0,
      height: props.videoInfo?.height ?? 0,
    });
  }, [props.videoInfo]);

  function adjustWidth(width: number) {
    if (resizeRatioLocked && props.videoInfo) {
      let newHeight = (width / props.videoInfo?.aspect_ratio_width) * props.videoInfo?.aspect_ratio_height;
      newHeight = Math.round(newHeight) % 2 === 0 ? Math.round(newHeight) : Math.round(newHeight) + 1;

      setVideoDimensions({ width, height: newHeight });
    } else {
      setVideoDimensions({ ...videoDimensions, width });
    }
  }

  function adjustHeight(height: number) {
    if (resizeRatioLocked && props.videoInfo) {
      let newWidth = (height / props.videoInfo?.aspect_ratio_height) * props.videoInfo?.aspect_ratio_width;
      newWidth = Math.round(newWidth) % 2 === 0 ? Math.round(newWidth) : Math.round(newWidth) + 1;

      setVideoDimensions({ width: newWidth, height });
    } else {
      setVideoDimensions({ ...videoDimensions, height });
    }
  }

  return (
    <div className={props.disabled || !props.videoNotCropped ? "disabled" : ""}>
      <div style={{ display: "flex", gap: "10px" }}>
        <div style={{ fontSize: "1.2em", fontWeight: "bold" }}>Resize</div>
        <Checkbox defaultChecked={false} onChange={(e) => setSegmentEnabled(e.target.checked)} />
      </div>

      {!props.videoNotCropped && segmentEnabled && <div style={{ color: "red", fontSize: "0.8em" }}>Cannot resize if crop is enabled.</div>}

      <div className={segmentEnabled ? "" : "disabled"} style={{ display: "flex", gap: "10px", alignItems: "center", height: "100%", marginTop: "25px" }}>
        <div style={{ display: "flex", gap: "5px", flexDirection: "column" }}>
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
              value={videoDimensions.width}
              placeholder="1920"
              onChange={(value) => {
                adjustWidth(value ?? 0);
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
              value={videoDimensions.height}
              title="height"
              placeholder="1080"
              onChange={(value) => {
                adjustHeight(value ?? 0);
              }}
            />
          </div>
        </div>
        <div style={{ fontSize: "1.1em", cursor: "pointer", height: "100%", display: "flex", alignItems: "center" }}>
          {resizeRatioLocked ? <LockOutlined onClick={() => setResizeRatioLocked(false)} /> : <UnlockOutlined onClick={() => setResizeRatioLocked(true)} />}
        </div>
      </div>
    </div>
  );
}

export default ResizeSegment;
