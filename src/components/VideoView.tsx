import { convertFileSrc } from "@tauri-apps/api/core";
import { useContext, useEffect, useRef, useState } from "react";
import { getCanvasToVideoSizeDifference, videoPathIsValid } from "../Logic/Utils/Utils";
import { canvasLineDisplacementRef, clickedLineInfo, cropInputManuallyChangedInfo, CropPointsContext, CutSegmentContext } from "../Logic/GlobalContexts";
import type { VideoCropPoints, VideoInfo } from "../Logic/Interfaces/Interfaces";
import { HoveringOver } from "../Logic/Enums/Enums";
import { determineIfHoveringOverLine, updateCanvasLineDisplacement } from "../Logic/Utils/VideoCropUtils";

interface VideoViewProps {
  videoInfo: VideoInfo | undefined;
  videoPath: string;
  onVideoPathClick: () => void;
}

function VideoView(props: VideoViewProps) {
  const videoRef = useRef<HTMLVideoElement>(null);
  const canvasRef = useRef<HTMLCanvasElement>(null);

  const { cropPointPositions, setCropPointPositions, cropLinesUnlocked, /*setCropLinesUnlocked,*/ cropEnabled, resetCropPoints /*setResetCropPoints*/ } = useContext(CropPointsContext);

  const [currentlyHovering, setCurrentlyHovering] = useState<HoveringOver | undefined>(undefined);

  const cropLinesUnlockedRef = useRef(cropLinesUnlocked);
  const { sharedCutSegmentOptions } = useContext(CutSegmentContext);

  useEffect(() => {
    canvasLineDisplacementRef.bottom = 1;
    canvasLineDisplacementRef.left = 1;
    canvasLineDisplacementRef.right = 1;
    canvasLineDisplacementRef.top = 1;
    updateCanvasSize();
  }, [resetCropPoints, props.videoPath, props.videoInfo]);

  useEffect(() => {
    cropLinesUnlockedRef.current = cropLinesUnlocked;
    updateCanvasSize();
  }, [cropLinesUnlocked, cropEnabled]);

  useEffect(() => {
    if (!clickedLineInfo.clickedLine && videoRef.current) {
      const { widthDiff, heightDiff } = getCanvasToVideoSizeDifference(videoRef);

      canvasLineDisplacementRef.left = Math.round(cropPointPositions.starting_x_offset * widthDiff);
      canvasLineDisplacementRef.top = Math.round(cropPointPositions.starting_y_offset * heightDiff);
      canvasLineDisplacementRef.right = Math.max(0, Math.round((videoRef.current.videoWidth - (cropPointPositions.starting_x_offset + cropPointPositions.width)) * widthDiff));
      canvasLineDisplacementRef.bottom = Math.max(0, Math.round((videoRef.current.videoHeight - (cropPointPositions.starting_y_offset + cropPointPositions.height)) * heightDiff));
      redrawCanvas();
    }
  }, [cropInputManuallyChangedInfo.manuallyChanged]);

  const updateCropPositions = () => {
    if (cropLinesUnlocked && canvasRef.current) {
      const { widthDiff, heightDiff } = getCanvasToVideoSizeDifference(videoRef);

      const newCropPointPositions: VideoCropPoints = {
        starting_x_offset: Math.floor(canvasLineDisplacementRef.left / widthDiff / 2) * 2,
        starting_y_offset: Math.floor(canvasLineDisplacementRef.top / heightDiff / 2) * 2,
        width: Math.round((canvasRef.current.width - (canvasLineDisplacementRef.right + canvasLineDisplacementRef.left)) / widthDiff / 2) * 2,
        height: Math.round((canvasRef.current.height - (canvasLineDisplacementRef.bottom + canvasLineDisplacementRef.top)) / heightDiff / 2) * 2,
      };

      setCropPointPositions(newCropPointPositions);
    }
  };

  const onCanvasMouseMove = (e: React.MouseEvent<HTMLCanvasElement, MouseEvent>) => {
    if (!cropLinesUnlocked) return;

    if (!clickedLineInfo.clickedLine) {
      setCurrentlyHovering(determineIfHoveringOverLine(e, canvasLineDisplacementRef));
    } else {
      updateCanvasLineDisplacement(canvasLineDisplacementRef, clickedLineInfo.clickedLine, e);
      redrawCanvas();
      updateCropPositions();
    }
  };

  const redrawCanvas = () => {
    const canvas = canvasRef.current;
    if (!canvas) return;

    const ctx = canvas.getContext("2d");
    if (!ctx) return;

    const { left, right, top, bottom } = canvasLineDisplacementRef;

    ctx.strokeStyle = cropLinesUnlockedRef.current ? "red" : "gray";

    ctx.lineWidth = 2;

    ctx.clearRect(0, 0, canvas.width, canvas.height);
    ctx.beginPath();

    ctx.rect(0 + left, 0 + top, canvas.width - (right + left), canvas.height - (bottom + top));
    ctx.stroke();
  };

  const onCanvasMouseDown = () => {
    if (!cropLinesUnlocked) return;

    if (currentlyHovering) {
      clickedLineInfo.clickedLine = currentlyHovering;
      console.log("Clicked line: ", HoveringOver[clickedLineInfo.clickedLine]);
    }
  };

  const determineCursor = (currentlyHovering: HoveringOver | undefined): string | undefined => {
    if (!cropLinesUnlocked) return;

    switch (currentlyHovering) {
      case undefined:
        return "auto";
      case HoveringOver.Left:
      case HoveringOver.Right:
        return "ew-resize";
      case HoveringOver.Top:
      case HoveringOver.Bottom:
        return "ns-resize";
      case HoveringOver.TopLeftCorner:
      case HoveringOver.BottomRightCorner:
        return "nw-resize";
      case HoveringOver.TopRightCorner:
      case HoveringOver.BottomLeftCorner:
        return "ne-resize";
      default:
        return "auto";
    }
  };

  function updateCanvasSize() {
    if (videoRef.current && canvasRef.current) {
      const rect = videoRef.current.getBoundingClientRect();
      canvasRef.current.width = rect.width;
      canvasRef.current.height = rect.height;
      redrawCanvas();
    }
  }

  useEffect(() => {
    canvasLineDisplacementRef.bottom = 1;
    canvasLineDisplacementRef.left = 1;
    canvasLineDisplacementRef.right = 1;
    canvasLineDisplacementRef.top = 1;
    updateCanvasSize();

    if (videoRef.current) {
      videoRef.current.addEventListener("loadedmetadata", updateCanvasSize);
    }

    window.addEventListener("resize", updateCanvasSize);

    return () => {
      if (videoRef.current) {
        videoRef.current.removeEventListener("loadedmetadata", updateCanvasSize);
      }

      window.removeEventListener("resize", updateCanvasSize);
    };
  }, []);

  const setVideoTime = (time: number) => {
    if (videoRef.current) {
      videoRef.current.currentTime = time;
    }
  };

  useEffect(() => {
    setVideoTime(sharedCutSegmentOptions.startingSecond);
  }, [sharedCutSegmentOptions.startingSecond]);

  useEffect(() => {
    setVideoTime(sharedCutSegmentOptions.endingSecond);
  }, [sharedCutSegmentOptions.endingSecond]);

  useEffect(() => {
    canvasLineDisplacementRef.bottom = 1;
    canvasLineDisplacementRef.left = 1;
    canvasLineDisplacementRef.right = 1;
    canvasLineDisplacementRef.top = 1;
    updateCanvasSize();
    redrawCanvas();
  }, [props.videoPath]);

  function RenderVideoElement() {
    if (videoPathIsValid(props.videoPath)) {
      return (
        <div className="video-container">
          <video ref={videoRef} key={props.videoPath} controls={!cropLinesUnlocked}>
            <source src={convertFileSrc(props.videoPath)} />
          </video>
          <canvas
            style={{
              display: cropEnabled ? "" : "none",
              pointerEvents: cropLinesUnlocked ? "all" : "none",
              cursor: determineCursor(currentlyHovering),
            }}
            ref={canvasRef}
            className="video-crop-canvas"
            onMouseDown={onCanvasMouseDown}
            onMouseMove={onCanvasMouseMove}
            onMouseUp={() => {
              clickedLineInfo.clickedLine = undefined;
              console.log("Released line");
              console.log(canvasLineDisplacementRef);
            }}
            onMouseLeave={() => {
              clickedLineInfo.clickedLine = undefined;
            }}
          />
        </div>
      );
    }
    return (
      <div onClick={props.onVideoPathClick} onKeyDown={props.onVideoPathClick} className="empty-video-container">
        No video loaded. Click to open a video file or drag one.
      </div>
    );
  }

  return <div className="video-view">{RenderVideoElement()}</div>;
}

export default VideoView;
