import React from "react";
import { initiateVideoCropPoints } from "./Utils";
import type { VideoCropPoints } from "./Interfaces";

export const CropPointsContext = React.createContext({
    cropPointPositions: initiateVideoCropPoints(),
    setCropPointPositions: (_: VideoCropPoints) => {}
});