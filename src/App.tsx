import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import "./App.css";
import VideoView from "./components/VideoView";

function App() {
  const [greetMsg, setGreetMsg] = useState("");
  const [name, setName] = useState("");

  const [ffmpegExsts, setFfmpegExsts] = useState(false);

  async function greet() {
    setGreetMsg(await invoke("greet", { name }));
  }

  async function checkFfmpegAndFfprobe() {
    setFfmpegExsts(await invoke("check_ffmpeg_and_ffprobe"));
  }

  async function OnStart() {
    checkFfmpegAndFfprobe();
  }
  useEffect(() => { OnStart() }, []);

  if (ffmpegExsts) {
    return (
      <div className="ffmpeg-not-downloaded">
        Ffmpeg and Ffprobe were not located. Please download them and add them to path.
      </div>
    )
  }
  else {
    return (
      <main className="app-container">
        <div className="video-view-container">
          <VideoView />
        </div>

      </main>
    );
  }

}

export default App;
