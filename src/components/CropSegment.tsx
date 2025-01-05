import { useContext, useEffect, useState } from "react";
import { Button, Checkbox, InputNumber } from "antd";
import { cropInputManuallyChangedInfo, CropPointsContext } from "../Logic/GlobalContexts";
import { LockOutlined, UnlockOutlined } from "@ant-design/icons";
import type { VideoCropPoints, VideoInfo } from "../Logic/Interfaces";

interface CropSegmentProps {
  videoInfo: VideoInfo | undefined;
  disabled: boolean;
  onSegmentEnabledChanged?: (enabled: boolean) => void;
  onCropLinesLockStateChanged: (enabled: boolean) => void;
  onReset?: () => void;
  onChange: (x: VideoCropPoints) => void;
}

function CropSegment(props: CropSegmentProps) {
  const [segmentEnabled, setSegmentEnabled] = useState(false);
  const [cropLinesEnabled, setCropLinesEnabled] = useState(false);

  const { cropPointPositions, setCropPointPositions } = useContext(CropPointsContext);

  useEffect(() => {
    props.onChange(cropPointPositions);
  }, [cropPointPositions]);

  return (
    <div className={props.disabled ? "disabled" : ""}>
      <div style={{ marginBottom: "10px", display: "flex", gap: "10px", placeSelf: "end" }}>
        <Checkbox
          defaultChecked={false}
          onChange={(e) => {
            setSegmentEnabled(e.target.checked);
            props.onSegmentEnabledChanged?.(e.target.checked);
          }}
        />
        <div style={{ fontSize: "1.2em", fontWeight: "bold" }}>Crop</div>
      </div>

      <div className={segmentEnabled ? "" : "disabled"} style={{ display: "flex", gap: "5px", alignItems: "center", flexDirection: "column" }}>
        <div
          style={{
            display: "flex",
            gap: "5px",
            alignItems: "center",
            justifyContent: "space-between",
            placeSelf: "end",
            width: "100%",
          }}
        >
          <div style={{ marginRight: "5px" }}>Start X:</div>
          <InputNumber
            onChange={(e) => {
              setCropPointPositions({ ...cropPointPositions, starting_x_offset: e ?? 0 });
              cropInputManuallyChangedInfo.manuallyChanged++;
            }}
            style={{ maxWidth: "65px" }}
            value={cropPointPositions.starting_x_offset}
            max={(props.videoInfo?.width ?? 0) - cropPointPositions.width}
            min={0}
          />
        </div>

        <div
          style={{
            display: "flex",
            gap: "5px",
            alignItems: "center",
            justifyContent: "space-between",
            placeSelf: "end",
            width: "100%",
          }}
        >
          <div style={{ marginRight: "5px" }}>Start Y:</div>
          <InputNumber
            onChange={(e) => {
              setCropPointPositions({ ...cropPointPositions, starting_y_offset: e ?? 0 });
              cropInputManuallyChangedInfo.manuallyChanged++;
            }}
            style={{ maxWidth: "65px" }}
            value={cropPointPositions.starting_y_offset}
            max={(props.videoInfo?.height ?? 0) - cropPointPositions.height}
            min={0}
          />
        </div>

        <div
          style={{
            display: "flex",
            gap: "5px",
            alignItems: "center",
            justifyContent: "space-between",
            placeSelf: "end",
            width: "100%",
          }}
        >
          <div style={{ marginRight: "5px" }}>Width:</div>
          <InputNumber
            onChange={(e) => {
              setCropPointPositions({ ...cropPointPositions, width: e ?? 0 });
              cropInputManuallyChangedInfo.manuallyChanged++;
            }}
            style={{ maxWidth: "65px" }}
            value={cropPointPositions.width}
            min={segmentEnabled ? 50 : 0}
            max={(props.videoInfo?.width ?? 0) - cropPointPositions.starting_x_offset}
          />
        </div>

        <div
          style={{
            display: "flex",
            gap: "5px",
            alignItems: "center",
            justifyContent: "space-between",
            placeSelf: "end",
            width: "100%",
          }}
        >
          <div style={{ marginRight: "5px" }}>Height:</div>
          <InputNumber
            onChange={(e) => {
              setCropPointPositions({ ...cropPointPositions, height: e ?? 0 });
              cropInputManuallyChangedInfo.manuallyChanged++;
            }}
            style={{ maxWidth: "65px" }}
            value={cropPointPositions.height}
            min={segmentEnabled ? 50 : 0}
            max={(props.videoInfo?.height ?? 0) - cropPointPositions.starting_y_offset}
          />
        </div>

        <div style={{ display: "flex", gap: "5px" }}>
          <Button
            variant="outlined"
            color={cropLinesEnabled ? "danger" : "primary"}
            onClick={() => {
              const newCropLinesEnabled = !cropLinesEnabled;
              setCropLinesEnabled(newCropLinesEnabled);
              props.onCropLinesLockStateChanged(newCropLinesEnabled);
            }}
          >
            {cropLinesEnabled ? <UnlockOutlined /> : <LockOutlined />}
          </Button>
          <Button onClick={props.onReset} type="primary">
            Reset
          </Button>
        </div>
      </div>
    </div>
  );
}

export default CropSegment;
