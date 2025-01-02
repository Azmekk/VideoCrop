import React from "react";
import { initiateVideoCropPoints } from "./Utils";
import type { VideoCropPoints } from "./Interfaces";
import type { HoveringOver } from "./Enums";

export const CropPointsContext = React.createContext({
  cropPointPositions: initiateVideoCropPoints(),
  setCropPointPositions: (_: VideoCropPoints) => {},
});

export const canvasLineDisplacementRef = {
  left: 0,
  right: 0,
  top: 0,
  bottom: 0,
};

export const clickedLineInfo = {
  clickedLine: undefined as HoveringOver | undefined,
};

export const cropInputManuallyChangedInfo = {
  manuallyChanged: 0,
};
