import { convertFileSrc} from "@tauri-apps/api/core";
import { videoPathIsValid } from "../App";

interface VideoViewProps {
    videoPath: string;
    onVideoPathClick: () => void;
}

function VideoView({ videoPath, onVideoPathClick }: VideoViewProps) {

    function RenderVideoElement() {
        if (videoPathIsValid(videoPath)) {
            return (
                <div className="selected-video-container">
                    <video key={videoPath} controls>
                        <source src={convertFileSrc(videoPath)}></source>
                    </video>
                    <canvas className="video-crop-canvas"></canvas>
                </div>
            );
        } else {
            return (
                <div onClick={onVideoPathClick} className="empty-video-container">
                    No video loaded. Click to open a video file.
                </div>
            );
        }
    }

    return (
        <div className="video-view">
            {RenderVideoElement()}
        </div>
    );
}



export default VideoView;