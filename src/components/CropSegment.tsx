import { useContext, useEffect, useState } from "react";
import type { VideoCropPoints, VideoInfo } from "../Logic/Interfaces";
import { Checkbox, InputNumber } from "antd";
import { CropPointsContext } from "../Logic/GlobalContexts";

interface CropSegmentProps {
  disabled: boolean;
  onSegmentEnabledChanged?: (enabled: boolean) => void;
}

enum CropPointInput {
  TopLeftX = "topLeftX",
  TopLeftY = "topLeftY",
  TopRightX = "topRightX",
  TopRightY = "topRightY",
  BottomLeftX = "bottomLeftX",
  BottomLeftY = "bottomLeftY",
  BottomRightX = "bottomRightX",
  BottomRightY = "bottomRightY",
}

function CropSegment(props: CropSegmentProps) {
  const [segmentEnabled, setSegmentEnabled] = useState(false);

  const { cropPointPositions, setCropPointPositions } = useContext(CropPointsContext);

  const handleCropPointChange = (point: CropPointInput, value: number) => {
    const newCropPoints = { ...cropPointPositions };

    switch (point) {
      case CropPointInput.TopLeftX:
        newCropPoints.topLeft.x = value;
        break;
      case CropPointInput.TopLeftY:
        newCropPoints.topLeft.y = value;
        break;
      case CropPointInput.TopRightX:
        newCropPoints.topRight.x = value;
        break;
      case CropPointInput.TopRightY:
        newCropPoints.topRight.y = value;
        break;
      case CropPointInput.BottomLeftX:
        newCropPoints.bottomLeft.x = value;
        break;
      case CropPointInput.BottomLeftY:
        newCropPoints.bottomLeft.y = value;
        break;
      case CropPointInput.BottomRightX:
        newCropPoints.bottomRight.x = value;
        break;
      case CropPointInput.BottomRightY:
        newCropPoints.bottomRight.y = value;
        break;
      default:
        break;
    }

    setCropPointPositions(newCropPoints);
  };

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

      <div
        className={segmentEnabled ? "" : "disabled"}
        style={{ display: "flex", gap: "5px", alignItems: "center", flexDirection: "column" }}
      >
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
          <div style={{ marginRight: "5px" }}>TL:</div>

          <div style={{ display: "flex", gap: "5px", alignItems: "center" }}>
            <div>x:</div>
            <InputNumber
              onChange={(e) => {
                handleCropPointChange(CropPointInput.TopLeftX, e ?? 0);
              }}
              style={{ maxWidth: "65px" }}
              value={cropPointPositions.topLeft.x}
            />
            <div>y:</div>
            <InputNumber
              onChange={(e) => {
                handleCropPointChange(CropPointInput.TopLeftY, e ?? 0);
              }}
              style={{ maxWidth: "65px" }}
              value={cropPointPositions.topLeft.y}
            />
          </div>
        </div>

        <div style={{ display: "flex", gap: "5px", alignItems: "center" }}>
          <div style={{ marginRight: "5px" }}>TR:</div>

          <div style={{ display: "flex", gap: "5px", alignItems: "center" }}>
            <div>x:</div>
            <InputNumber
              onChange={(e) => {
                handleCropPointChange(CropPointInput.TopRightX, e ?? 0);
              }}
              style={{ maxWidth: "65px" }}
              value={cropPointPositions?.topRight.x}
            />
            <div>y:</div>
            <InputNumber
              onChange={(e) => {
                handleCropPointChange(CropPointInput.TopRightY, e ?? 0);
              }}
              style={{ maxWidth: "65px" }}
              value={cropPointPositions?.topRight.y}
            />
          </div>
        </div>

        <div style={{ display: "flex", gap: "5px", alignItems: "center" }}>
          <div style={{ marginRight: "5px" }}>BL:</div>

          <div style={{ display: "flex", gap: "5px", alignItems: "center" }}>
            <div>x:</div>
            <InputNumber
              onChange={(e) => {
                handleCropPointChange(CropPointInput.BottomLeftX, e ?? 0);
              }}
              style={{ maxWidth: "65px" }}
              value={cropPointPositions?.bottomLeft.x}
            />
            <div>y:</div>
            <InputNumber
              onChange={(e) => {
                handleCropPointChange(CropPointInput.BottomLeftY, e ?? 0);
              }}
              style={{ maxWidth: "65px" }}
              value={cropPointPositions?.bottomLeft.y}
            />
          </div>
        </div>

        <div style={{ display: "flex", gap: "5px", alignItems: "center" }}>
          <div style={{ marginRight: "5px" }}>BR:</div>

          <div style={{ display: "flex", gap: "5px", alignItems: "center" }}>
            <div>x:</div>
            <InputNumber
              onChange={(e) => {
                handleCropPointChange(CropPointInput.BottomRightX, e ?? 0);
              }}
              style={{ maxWidth: "65px" }}
              value={cropPointPositions?.bottomRight.x}
            />
            <div>y:</div>
            <InputNumber
              onChange={(e) => {
                handleCropPointChange(CropPointInput.BottomRightY, e ?? 0);
              }}
              style={{ maxWidth: "65px" }}
              value={cropPointPositions?.bottomRight.y}
            />
          </div>
        </div>
      </div>
    </div>
  );
}

export default CropSegment;
