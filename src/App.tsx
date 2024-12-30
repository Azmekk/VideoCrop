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

  async function OnStart(){
    checkFfmpegAndFfprobe();
  }
  useEffect(() => {OnStart()}, []);

  return (
    <main className="app-container">
      <div className="video-view-container">
        <VideoView />
      </div>
      
    </main>
  );
}

export default App;
