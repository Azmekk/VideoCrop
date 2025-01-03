import { Button } from "antd";
import type { VideoEditOptions } from "../Logic/Interfaces";
import { videoPathIsValid } from "../Logic/Utils";

const videoPathSelection = (props: { videoEditOptions: VideoEditOptions; videoPath: string; onClick: () => void }) => {
  return (
    <div style={{ marginBottom: "20px", display: "flex", flexDirection: "column", gap: "10px" }}>
      <Button
        disabled={!videoPathIsValid(props.videoPath)}
        size="large"
        color={props.videoEditOptions.output_video_path === "" ? "danger" : "primary"}
        onClick={props.onClick}
        variant={props.videoEditOptions.output_video_path === "" ? "solid" : "dashed"}
      >
        {props.videoEditOptions.output_video_path === "" ? "Select output path" : "Change output path"}
      </Button>
      {props.videoEditOptions.output_video_path !== "" && (
        <div>{props.videoEditOptions.output_video_path.length > 25 ? ` ...${props.videoEditOptions.output_video_path.slice(-25)}` : props.videoEditOptions.output_video_path}</div>
      )}
    </div>
  );
};

export default videoPathSelection;
