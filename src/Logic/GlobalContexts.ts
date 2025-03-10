import React from "react";
import { initiateVideoCropPoints } from "./Utils/Utils";
import type { SharedCutSegmentOptions, SharedCutSegmentOptionsContext, VideoCropPoints } from "./Interfaces/Interfaces";
import type { HoveringOver } from "./Enums/Enums";

export const CropPointsContext = React.createContext({
  cropPointPositions: initiateVideoCropPoints(),
  setCropPointPositions: (_: VideoCropPoints) => {},
  cropLinesUnlocked: false,
  setCropLinesUnlocked: (_: boolean) => {},
  cropEnabled: false,
  resetCropPoints: 0,
  setResetCropPoints: (_: number) => {},
});

export const CutSegmentContext = React.createContext<SharedCutSegmentOptionsContext>({
  setSharedCutSegmentOptions: (_: SharedCutSegmentOptions) => {},
  sharedCutSegmentOptions: {
    startingSecond: 0,
    endingSecond: 0,
  },
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
