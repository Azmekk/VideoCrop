import { convertFileSrc } from "@tauri-apps/api/core";
import { useContext, useEffect, useRef, useState } from "react";
import { videoPathIsValid } from "../Logic/Utils";
import {
  canvasLineDisplacementRef,
  clickedLineInfo,
  cropInputManuallyChangedInfo,
  CropPointsContext,
} from "../Logic/GlobalContexts";
import type { VideoCropPoints, VideoInfo } from "../Logic/Interfaces";
import { HoveringOver } from "../Logic/Enums";
import { determineIfHoveringOverLine, updateCanvasLineDisplacement } from "../Logic/VideoCropLinesLogic";

interface VideoViewProps {
  videoInfo: VideoInfo | undefined;
  videoPath: string;
  onVideoPathClick: () => void;
  resizerEnabled: boolean;
  reset: number;
  cropEnabled: boolean;
}

function VideoView(props: VideoViewProps) {
  const videoRef = useRef<HTMLVideoElement>(null);
  const canvasRef = useRef<HTMLCanvasElement>(null);

  const { cropPointPositions, setCropPointPositions } = useContext(CropPointsContext);

  const [currentlyHovering, setCurrentlyHovering] = useState<HoveringOver | undefined>(undefined);

  useEffect(() => {
    canvasLineDisplacementRef.bottom = 1;
    canvasLineDisplacementRef.left = 1;
    canvasLineDisplacementRef.right = 1;
    canvasLineDisplacementRef.top = 1;
    redrawCanvas();
  }, [props.reset]);

  useEffect(() => {
    redrawCanvas();
  }, [props.resizerEnabled]);

  useEffect(() => {
    if (!clickedLineInfo.clickedLine && videoRef.current) {
      const { widthDiff, heightDiff } = getCanvasToVideoSizeDifference();

      canvasLineDisplacementRef.left = Math.round(cropPointPositions.startingXOffset * widthDiff);
      canvasLineDisplacementRef.top = Math.round(cropPointPositions.startingYOffset * heightDiff);
      canvasLineDisplacementRef.right = Math.max(
        0,
        Math.round(
          (videoRef.current.videoWidth - (cropPointPositions.startingXOffset + cropPointPositions.width)) * widthDiff,
        ),
      );
      canvasLineDisplacementRef.bottom = Math.max(
        0,
        Math.round(
          (videoRef.current.videoHeight - (cropPointPositions.startingYOffset + cropPointPositions.height)) *
            heightDiff,
        ),
      );
      redrawCanvas();
    }
  }, [cropInputManuallyChangedInfo.manuallyChanged]);

  const updateCropPositions = () => {
    if (props.resizerEnabled && canvasRef.current) {
      const { widthDiff, heightDiff } = getCanvasToVideoSizeDifference();

      const newCropPointPositions: VideoCropPoints = {
        startingXOffset: Math.round(canvasLineDisplacementRef.left / widthDiff / 2) * 2,
        startingYOffset: Math.round(canvasLineDisplacementRef.top / heightDiff / 2) * 2,
        width:
          Math.round(
            (canvasRef.current.width - (canvasLineDisplacementRef.right + canvasLineDisplacementRef.left)) /
              widthDiff /
              2,
          ) * 2,
        height:
          Math.round(
            (canvasRef.current.height - (canvasLineDisplacementRef.bottom + canvasLineDisplacementRef.top)) /
              heightDiff /
              2,
          ) * 2,
      };

      setCropPointPositions(newCropPointPositions);
    }
  };

  const getCanvasToVideoSizeDifference = () => {
    if (!videoRef.current) return { widthDiff: 0, heightDiff: 0 };

    const videoWidth = videoRef.current.videoWidth;
    const videoHeight = videoRef.current.videoHeight;

    const rect = videoRef.current.getBoundingClientRect();
    const elementWidth = rect.width;
    const elementHeight = rect.height;

    return {
      widthDiff: elementWidth / videoWidth,
      heightDiff: elementHeight / videoHeight,
    };
  };

  const getVideoToCanvasSizeDifference = () => {
    if (!videoRef.current) return { widthDiff: 0, heightDiff: 0 };

    const videoWidth = videoRef.current.videoWidth;
    const videoHeight = videoRef.current.videoHeight;
    const rect = videoRef.current.getBoundingClientRect();
    const elementWidth = rect.width;
    const elementHeight = rect.height;

    return {
      widthDiff: videoWidth / elementWidth,
      heightDiff: videoHeight / elementHeight,
    };
  };

  const onCanvasMouseMove = (e: React.MouseEvent<HTMLCanvasElement, MouseEvent>) => {
    if (!props.resizerEnabled) return;

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

    ctx.strokeStyle = props.resizerEnabled ? "red" : "gray";
    ctx.lineWidth = 2;

    ctx.clearRect(0, 0, canvas.width, canvas.height);
    ctx.beginPath();

    ctx.rect(0 + left, 0 + top, canvas.width - (right + left), canvas.height - (bottom + top));
    ctx.stroke();
  };

  const onCanvasMouseDown = () => {
    if (!props.resizerEnabled) return;

    if (currentlyHovering) {
      clickedLineInfo.clickedLine = currentlyHovering;
      console.log("Clicked line: ", HoveringOver[clickedLineInfo.clickedLine]);
    }
  };

  const determineCursor = (currentlyHovering: HoveringOver | undefined): string | undefined => {
    if (!props.resizerEnabled) return;

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

    const handleLoadedMetadata = () => {
      updateCanvasSize();
    };

    const videoElement = videoRef.current;
    if (videoElement) {
      videoElement.addEventListener("loadedmetadata", handleLoadedMetadata);
    }

    window.addEventListener("resize", updateCanvasSize);

    return () => {
      if (videoElement) {
        videoElement.removeEventListener("loadedmetadata", handleLoadedMetadata);
      }
      window.removeEventListener("resize", updateCanvasSize);
    };
  }, [props.videoPath]);

  function RenderVideoElement() {
    if (videoPathIsValid(props.videoPath)) {
      return (
        <div className="video-container">
          <video ref={videoRef} key={props.videoPath} controls={!props.resizerEnabled}>
            <source src={convertFileSrc(props.videoPath)} />
          </video>
          <canvas
            style={{
              display: props.cropEnabled ? "" : "none",
              pointerEvents: props.resizerEnabled ? "all" : "none",
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
        No video loaded. Click to open a video file.
      </div>
    );
  }

  return <div className="video-view">{RenderVideoElement()}</div>;
}

export default VideoView;
