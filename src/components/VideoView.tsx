import { convertFileSrc } from "@tauri-apps/api/core";
import { useContext, useEffect, useRef, useState } from "react";
import { videoPathIsValid } from "../Logic/Utils";
import { CropPointsContext } from "../Logic/GlobalContexts";
import type { VideoCropPoints } from "../Logic/Interfaces";

interface VideoViewProps {
  videoPath: string;
  onVideoPathClick: () => void;
  resizerEnabled: boolean;
  reset: number;
}

enum HoveringOver {
  Left = 1,
  Right = 2,
  Top = 3,
  Bottom = 4,
  TopLeftCorner = 5,
  TopRightCorner = 6,
  BottomLeftCorner = 7,
  BottomRightCorner = 8,
}

function VideoView(props: VideoViewProps) {
  const videoRef = useRef<HTMLVideoElement>(null);
  const canvasRef = useRef<HTMLCanvasElement>(null);

  //const { cropPointPositions, setCropPointPositions } = useContext(CropPointsContext);

  const [currentlyHovering, setCurrentlyHovering] = useState<HoveringOver | undefined>(undefined);

  const canvasLineDisplacementRef = useRef({
    left: 0,
    right: 0,
    top: 0,
    bottom: 0,
  });

  let clickedLine: HoveringOver | undefined = undefined;

  useEffect(() => {
    canvasLineDisplacementRef.current.bottom = 0;
    canvasLineDisplacementRef.current.left = 0;
    canvasLineDisplacementRef.current.right = 0;
    canvasLineDisplacementRef.current.top = 0;
    redrawCanvas();
  }, [props.reset]);

  const determineIfHoveringOverLine = (
    e: React.MouseEvent<HTMLCanvasElement, MouseEvent>,
  ): HoveringOver | undefined => {
    if (!props.resizerEnabled) return;

    const canvasRect = e.currentTarget.getBoundingClientRect();
    const offsetX = e.clientX - canvasRect.left;
    const offsetY = e.clientY - canvasRect.top;

    console.log(offsetX, offsetY);
    const tolerance = 15;

    const { left, right, top, bottom } = canvasLineDisplacementRef.current;

    const isHoveringOverLeftLine = Math.abs(offsetX - left) < tolerance;
    const isHoveringOverRightLine = Math.abs(offsetX - (canvasRect.width - right)) < tolerance;
    const isHoveringOverTopLine = Math.abs(offsetY - top) < tolerance;
    const isHoveringOverBottomLine = Math.abs(offsetY - (canvasRect.height - bottom)) < tolerance;

    const isHoveringOverTopLeftCorner = isHoveringOverTopLine && isHoveringOverLeftLine;
    const isHoveringOverTopRightCorner = isHoveringOverTopLine && isHoveringOverRightLine;
    const isHoveringOverBottomLeftCorner = isHoveringOverBottomLine && isHoveringOverLeftLine;
    const isHoveringOverBottomRightCorner = isHoveringOverBottomLine && isHoveringOverRightLine;

    if (isHoveringOverTopLeftCorner) {
      console.log("Hovering over the top-left corner");
      return HoveringOver.TopLeftCorner;
    }
    if (isHoveringOverTopRightCorner) {
      console.log("Hovering over the top-right corner");
      return HoveringOver.TopRightCorner;
    }
    if (isHoveringOverBottomLeftCorner) {
      console.log("Hovering over the bottom-left corner");
      return HoveringOver.BottomLeftCorner;
    }
    if (isHoveringOverBottomRightCorner) {
      console.log("Hovering over the bottom-right corner");
      return HoveringOver.BottomRightCorner;
    }

    if (isHoveringOverLeftLine) {
      console.log("Hovering over the left line");
      return HoveringOver.Left;
    }
    if (isHoveringOverRightLine) {
      console.log("Hovering over the right line");
      return HoveringOver.Right;
    }
    if (isHoveringOverTopLine) {
      console.log("Hovering over the top line");
      return HoveringOver.Top;
    }
    if (isHoveringOverBottomLine) {
      console.log("Hovering over the bottom line");
      return HoveringOver.Bottom;
    }

    return undefined;
  };

  const onCanvasMouseMove = (e: React.MouseEvent<HTMLCanvasElement, MouseEvent>) => {
    if (!props.resizerEnabled) return;

    if (!clickedLine) {
      setCurrentlyHovering(determineIfHoveringOverLine(e));
    } else {
      const minimumSeparation = 25;
      switch (clickedLine) {
        case undefined:
          return;
        case HoveringOver.Left: {
          const updatedLeftValue = e.clientX - e.currentTarget.getBoundingClientRect().left;

          if (updatedLeftValue + canvasLineDisplacementRef.current.right + minimumSeparation > e.currentTarget.width) {
            return;
          }
          canvasLineDisplacementRef.current.left = updatedLeftValue;
          break;
        }
        case HoveringOver.Right: {
          const updatedRightValue = e.currentTarget.getBoundingClientRect().right - e.clientX;

          if (updatedRightValue + canvasLineDisplacementRef.current.left + minimumSeparation > e.currentTarget.width) {
            return;
          }
          canvasLineDisplacementRef.current.right = updatedRightValue;
          break;
        }
        case HoveringOver.Top: {
          const updatedTopValue = e.clientY - e.currentTarget.getBoundingClientRect().top;

          if (updatedTopValue + canvasLineDisplacementRef.current.bottom + minimumSeparation > e.currentTarget.height) {
            return;
          }
          canvasLineDisplacementRef.current.top = updatedTopValue;
          break;
        }
        case HoveringOver.Bottom: {
          const updatedBottomValue = e.currentTarget.getBoundingClientRect().bottom - e.clientY;

          if (updatedBottomValue + canvasLineDisplacementRef.current.top + minimumSeparation > e.currentTarget.height) {
            return;
          }
          canvasLineDisplacementRef.current.bottom = updatedBottomValue;
          break;
        }
        case HoveringOver.TopLeftCorner: {
          const updatedLeftValue = e.clientX - e.currentTarget.getBoundingClientRect().left;
          const updatedTopValue = e.clientY - e.currentTarget.getBoundingClientRect().top;

          if (
            updatedLeftValue + canvasLineDisplacementRef.current.right + minimumSeparation > e.currentTarget.width ||
            updatedTopValue + canvasLineDisplacementRef.current.bottom + minimumSeparation > e.currentTarget.height
          ) {
            return;
          }
          canvasLineDisplacementRef.current.left = updatedLeftValue;
          canvasLineDisplacementRef.current.top = updatedTopValue;
          break;
        }
        case HoveringOver.TopRightCorner: {
          const updatedRightValue = e.currentTarget.getBoundingClientRect().right - e.clientX;
          const updatedTopValue = e.clientY - e.currentTarget.getBoundingClientRect().top;

          if (
            updatedRightValue + canvasLineDisplacementRef.current.left + minimumSeparation > e.currentTarget.width ||
            updatedTopValue + canvasLineDisplacementRef.current.bottom + minimumSeparation > e.currentTarget.height
          ) {
            return;
          }
          canvasLineDisplacementRef.current.right = updatedRightValue;
          canvasLineDisplacementRef.current.top = updatedTopValue;
          break;
        }
        case HoveringOver.BottomLeftCorner: {
          const updatedLeftValue = e.clientX - e.currentTarget.getBoundingClientRect().left;
          const updatedBottomValue = e.currentTarget.getBoundingClientRect().bottom - e.clientY;

          if (
            updatedLeftValue + canvasLineDisplacementRef.current.right + minimumSeparation > e.currentTarget.width ||
            updatedBottomValue + canvasLineDisplacementRef.current.top + minimumSeparation > e.currentTarget.height
          ) {
            return;
          }
          canvasLineDisplacementRef.current.left = updatedLeftValue;
          canvasLineDisplacementRef.current.bottom = updatedBottomValue;
          break;
        }
        case HoveringOver.BottomRightCorner: {
          const updatedRightValue = e.currentTarget.getBoundingClientRect().right - e.clientX;
          const updatedBottomValue = e.currentTarget.getBoundingClientRect().bottom - e.clientY;

          if (
            updatedRightValue + canvasLineDisplacementRef.current.left + minimumSeparation > e.currentTarget.width ||
            updatedBottomValue + canvasLineDisplacementRef.current.top + minimumSeparation > e.currentTarget.height
          ) {
            return;
          }
          canvasLineDisplacementRef.current.right = updatedRightValue;
          canvasLineDisplacementRef.current.bottom = updatedBottomValue;
          break;
        }
      }
      redrawCanvas();
    }
  };

  const redrawCanvas = () => {
    const canvas = canvasRef.current;
    if (!canvas) return;

    const ctx = canvas.getContext("2d");
    if (!ctx) return;

    const { left, right, top, bottom } = canvasLineDisplacementRef.current;

    ctx.clearRect(0, 0, canvas.width, canvas.height);
    ctx.beginPath();

    ctx.rect(0 + left, 0 + top, canvas.width - (right + left), canvas.height - (bottom + top));
    ctx.stroke();
  };

  const onCanvasMouseDown = () => {
    if (!props.resizerEnabled) return;

    if (currentlyHovering) {
      clickedLine = currentlyHovering;
      console.log("Clicked line: ", HoveringOver[clickedLine]);
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

  const drawInitialLines = (canvas: HTMLCanvasElement) => {
    const ctx = canvas.getContext("2d");
    if (!ctx) {
      return;
    }

    const width = canvas.width;
    const height = canvas.height;

    ctx.strokeStyle = "red";
    ctx.lineWidth = 2;

    ctx.rect(0, 0, width, height);
    ctx.stroke();
  };

  function updateCanvasSize() {
    if (videoRef.current && canvasRef.current) {
      const rect = videoRef.current.getBoundingClientRect();
      canvasRef.current.width = rect.width;
      canvasRef.current.height = rect.height;
      drawInitialLines(canvasRef.current);
    }
  }

  useEffect(() => {
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
              display: props.resizerEnabled ? "" : "none",
              pointerEvents: props.resizerEnabled ? "all" : "none",
              cursor: determineCursor(currentlyHovering),
            }}
            ref={canvasRef}
            className="video-crop-canvas"
            onMouseDown={onCanvasMouseDown}
            onMouseMove={onCanvasMouseMove}
            onMouseUp={() => {
              clickedLine = undefined;
              console.log("Released line");
              console.log(canvasLineDisplacementRef);
            }}
            onMouseLeave={() => {
              clickedLine = undefined;
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
