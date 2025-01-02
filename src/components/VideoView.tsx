import { convertFileSrc } from "@tauri-apps/api/core";
import { useContext, useEffect, useRef } from "react";
import { videoPathIsValid } from "../Logic/Utils";
import { CropPointsContext } from "../Logic/GlobalContexts";
import type { VideoCropPoints } from "../Logic/Interfaces";

interface VideoViewProps {
  videoPath: string;
  onVideoPathClick: () => void;
  enabled: boolean;
}

function VideoView(props: VideoViewProps) {
  const videoRef = useRef<HTMLVideoElement>(null);
  const canvasRef = useRef<HTMLCanvasElement>(null);

  const canvasLineDisplacement = {
    left: 0,
    right: 0,
    top: 0,
    bottom: 0,
  };

  //const { cropPointPositions, setCropPointPositions } = useContext(CropPointsContext);

  const determineIfHoveringOverLine = (e: React.MouseEvent<HTMLCanvasElement, MouseEvent>) => {
    if (!props.enabled) return;

    const canvasRect = e.currentTarget.getBoundingClientRect();
    const offsetX = e.clientX - canvasRect.left;
    const offsetY = e.clientY - canvasRect.top;

    console.log(offsetX, offsetY);
    const tolerance = 15;

    const { left, right, top, bottom } = canvasLineDisplacement;

    const isHoveringOverLeftLine = Math.abs(offsetX - left) < tolerance;
    const isHoveringOverRightLine = Math.abs(offsetX - (canvasRect.width - right)) < tolerance;
    const isHoveringOverTopLine = Math.abs(offsetY - top) < tolerance;
    const isHoveringOverBottomLine = Math.abs(offsetY - (canvasRect.height - bottom)) < tolerance;

    if (isHoveringOverLeftLine) {
      console.log("Hovering over the left line");
    }
    if (isHoveringOverRightLine) {
      console.log("Hovering over the right line");
    }
    if (isHoveringOverTopLine) {
      console.log("Hovering over the top line");
    }
    if (isHoveringOverBottomLine) {
      console.log("Hovering over the bottom line");
    }
    if (!isHoveringOverLeftLine && !isHoveringOverRightLine && !isHoveringOverTopLine && !isHoveringOverBottomLine) {
      console.log("Not hovering over any line");
    }

    const isHoveringOverTopLeftCorner = isHoveringOverTopLine && isHoveringOverLeftLine;
    const isHoveringOverTopRightCorner = isHoveringOverTopLine && isHoveringOverRightLine;
    const isHoveringOverBottomLeftCorner = isHoveringOverBottomLine && isHoveringOverLeftLine;
    const isHoveringOverBottomRightCorner = isHoveringOverBottomLine && isHoveringOverRightLine;

    if (isHoveringOverTopLeftCorner) {
      console.log("Hovering over the top-left corner");
    }
    if (isHoveringOverTopRightCorner) {
      console.log("Hovering over the top-right corner");
    }
    if (isHoveringOverBottomLeftCorner) {
      console.log("Hovering over the bottom-left corner");
    }
    if (isHoveringOverBottomRightCorner) {
      console.log("Hovering over the bottom-right corner");
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
    ctx.lineWidth = 5;

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

  const determineClickedCanvasLine = (e: React.MouseEvent<HTMLCanvasElement, MouseEvent>) => {};

  function RenderVideoElement() {
    if (videoPathIsValid(props.videoPath)) {
      return (
        <div className="video-container">
          <video ref={videoRef} key={props.videoPath} controls>
            <source src={convertFileSrc(props.videoPath)} />
          </video>
          <canvas
            style={{ display: props.enabled ? "" : "none", pointerEvents: props.enabled ? "all" : "none" }}
            ref={canvasRef}
            className="video-crop-canvas"
            onMouseDown={determineClickedCanvasLine}
            onMouseMove={determineIfHoveringOverLine}
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
