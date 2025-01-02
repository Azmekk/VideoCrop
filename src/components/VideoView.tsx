import { convertFileSrc } from "@tauri-apps/api/core";
import { useContext, useEffect, useRef } from "react";
import { videoPathIsValid } from "../Logic/Utils";
import { CropPointsContext } from "../Logic/GlobalContexts";

interface VideoViewProps {
  videoPath: string;
  onVideoPathClick: () => void;
  enabled: boolean;
}

function VideoView(props: VideoViewProps) {
  const videoRef = useRef<HTMLVideoElement>(null);
  const canvasRef = useRef<HTMLCanvasElement>(null);

  //const { cropPointPositions, setCropPointPositions } = useContext(CropPointsContext);

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
            style={{ display: props.enabled ? "" : "none" }}
            ref={canvasRef}
            className="video-crop-canvas"
            onMouseDown={determineClickedCanvasLine}
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
