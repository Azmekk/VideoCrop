import { convertFileSrc } from "@tauri-apps/api/core";
import type { VideoCropPoints } from "../Logic/Interfaces";
import { useEffect, useRef } from "react";
import { videoPathIsValid } from "../Logic/Utils";

interface VideoViewProps {
  videoPath: string;
  onVideoPathClick: () => void;
}

function VideoView(props: VideoViewProps) {
  const videoRef = useRef<HTMLVideoElement>(null);
  const canvasRef = useRef<HTMLCanvasElement>(null);

  useEffect(() => {
    function updateCanvasSize() {
      if (videoRef.current && canvasRef.current) {
        const rect = videoRef.current.getBoundingClientRect();
        canvasRef.current.width = rect.width;
        canvasRef.current.height = rect.height;
      }
    }

    updateCanvasSize();
    window.addEventListener("resize", updateCanvasSize);

    return () => {
      window.removeEventListener("resize", updateCanvasSize);
    };
  }, []);

  function RenderVideoElement() {
    if (videoPathIsValid(props.videoPath)) {
      return (
        <div className="video-container">
          <video ref={videoRef} key={props.videoPath} controls>
            <source src={convertFileSrc(props.videoPath)} />
          </video>
          <canvas
            ref={canvasRef}
            className="video-crop-canvas"
            width={videoRef.current?.videoWidth}
            height={videoRef.current?.videoHeight}
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
