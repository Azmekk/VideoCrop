import { convertFileSrc, invoke } from "@tauri-apps/api/core";
import { useEffect, useRef, useState } from "react";

function VideoView() {
    const [videoIsLoaded, setVideoIsLoaded] = useState(false);
    const [videoPath, setVideoPath] = useState("");
    async function get_video_path() {
        let path: string = await invoke("open_video");
        if (path !== "") {
            setVideoPath(path);
            setVideoIsLoaded(true);
        }
    }

    function RenderVideoElement() {
        if (videoIsLoaded) {
            return (
                <div className="selected-video-container">
                    <video controls>
                        <source src={convertFileSrc(videoPath)}></source>
                    </video>
                    <canvas className="video-crop-canvas">

                    </canvas>
                </div>
            );
        } else {
            return (
                <div onClick={async () => await get_video_path()} className="empty-video-container">
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