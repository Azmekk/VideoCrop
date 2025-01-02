import { convertFileSrc } from "@tauri-apps/api/core";
import { type VideoCropPoints, videoPathIsValid } from "../Utils";
import { useEffect, useRef } from "react";

interface VideoViewProps {
	videoPath: string;
	onVideoPathClick: () => void;
	videoCropPoints: VideoCropPoints;
}

function VideoView({ videoPath, onVideoPathClick }: VideoViewProps) {
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
		if (videoPathIsValid(videoPath)) {
			return (
				<div className="video-container">
					<video ref={videoRef} key={videoPath} controls>
						<source src={convertFileSrc(videoPath)} />
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
				<div onClick={onVideoPathClick} onKeyDown={onVideoPathClick} className="empty-video-container">
					No video loaded. Click to open a video file.
				</div>
			);
	}

	return <div className="video-view">{RenderVideoElement()}</div>;
}

export default VideoView;
