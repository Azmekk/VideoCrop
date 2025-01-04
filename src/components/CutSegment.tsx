import { Checkbox, Input, Slider, type SliderSingleProps } from "antd";
import { useEffect, useState } from "react";
import type { VideoCutOptions } from "../Logic/Interfaces";

interface CutSegmentProps {
  videoPath: string;
  videoDuration: string;
  onChange: (x: VideoCutOptions, enabled: boolean) => void;
}

interface VideoDuration {
  hours: number;
  minutes: number;
  seconds: number;
  microseconds: number;
}

function CutSegment(props: CutSegmentProps) {
  const [totalSeconds, setTotalSeconds] = useState(0);
  const [_, setVideoDuration] = useState<VideoDuration>({
    hours: 0,
    minutes: 0,
    seconds: 0,
    microseconds: 0,
  });

  const [startingSecond, setStartingSecond] = useState(0);
  const [endingSecond, setEndingSecond] = useState(0);

  const [startingInputError, setStartingInputError] = useState(false);
  const [endingInputError, setEndingInputError] = useState(false);

  const [segmentEnabled, setSegmentEnabled] = useState(false);

  useEffect(() => {
    const vidDuration = parseVideoDuration(props.videoDuration);
    setVideoDuration(vidDuration);

    const totalSecs = convertToSeconds(vidDuration);
    setTotalSeconds(totalSecs);

    setStartingSecond(0);
    setEndingSecond(totalSecs);
  }, [props.videoDuration]);

  useEffect(() => {
    props.onChange(
      {
        starting_time_string: videoDurationToString(convertFromSeconds(startingSecond)),
        end_time_string: videoDurationToString(convertFromSeconds(endingSecond)),
      },
      segmentEnabled,
    );
  }, [startingSecond, endingSecond, segmentEnabled]);

  const parseVideoDuration = (duration: string) => {
    const [hours, minutes, secs] = duration.split(":");
    let [seconds, microseconds] = ["00", "000000"];
    if (secs.includes(".")) {
      [seconds, microseconds] = secs.split(".");
      microseconds = microseconds.padEnd(6, "0");
    } else {
      seconds = secs;
    }

    return {
      hours: Number.parseInt(hours, 10),
      minutes: Number.parseInt(minutes, 10),
      seconds: Number.parseInt(seconds, 10),
      microseconds: Number.parseInt(microseconds, 10),
    };
  };

  const videoDurationToString = (videoDuration: VideoDuration): string => {
    const { hours, minutes, seconds, microseconds } = videoDuration;
    const milliseconds = Math.round(microseconds / 1000);
    return `${String(hours).padStart(2, "0")}:${String(minutes).padStart(2, "0")}:${String(seconds).padStart(2, "0")}.${String(milliseconds).padStart(3, "0")}`;
  };

  const convertToSeconds = (videoDuration: VideoDuration): number => {
    const { hours, minutes, seconds, microseconds: milliseconds } = videoDuration;
    return hours * 3600 + minutes * 60 + seconds + milliseconds / 1000000;
  };

  const convertFromSeconds = (totalSeconds: number): VideoDuration => {
    let seconds = totalSeconds;

    const hours = Math.floor(seconds / 3600);
    seconds %= 3600;
    const minutes = Math.floor(seconds / 60);
    seconds %= 60;
    const microseconds = Math.round((seconds % 1) * 1000000);
    seconds = Math.floor(seconds);

    return {
      hours,
      minutes,
      seconds,
      microseconds,
    };
  };

  const handleSliderInput = (e: number[]) => {
    setStartingSecond(e[0] < 0 ? 0 : e[0]);
    setEndingSecond(e[1] > totalSeconds ? totalSeconds : e[1]);
  };

  const handleDurationInput = (durationString: string, starting: boolean) => {
    if (!/^\d{2}:\d{2}:\d{2}/.test(durationString)) {
      return;
    }

    const duration = parseVideoDuration(durationString);
    const secs = convertToSeconds(duration);

    if (starting) {
      if (secs < 0) {
        setStartingInputError(true);
        setStartingSecond(0);
      } else {
        setStartingInputError(false);
        setStartingSecond(secs);
      }
    } else {
      if (secs > totalSeconds) {
        setEndingInputError(true);
        setEndingSecond(totalSeconds);
      } else {
        setEndingInputError(false);
        setEndingSecond(secs);
      }
    }
  };

  const formatter: NonNullable<SliderSingleProps["tooltip"]>["formatter"] = (value) => `${videoDurationToString(convertFromSeconds(Number(value)))}`;

  return (
    <div style={{ padding: "20px" }}>
      <div style={{ marginBottom: "10px", display: "flex", gap: "10px" }}>
        <div style={{ fontSize: "1.2em", fontWeight: "bold" }}>Cut</div>
        <Checkbox defaultChecked={false} onChange={(e) => setSegmentEnabled(e.target.checked)} />
      </div>

      <div className={segmentEnabled ? "" : "disabled"}>
        <Slider tooltip={{ formatter, placement: "left" }} range max={totalSeconds} value={[startingSecond, endingSecond]} step={0.05} onChange={handleSliderInput} />
        <div style={{ display: "flex", justifyContent: "space-between" }}>
          <div>
            <label htmlFor="startTimeInput">Start time:</label>
            <p style={{ cursor: "default" }}>{videoDurationToString(convertFromSeconds(startingSecond))}</p>
            <Input
              id="startTimeInput"
              status={startingInputError ? "error" : ""}
              placeholder="00:00:00.000"
              onChange={(e) => handleDurationInput(e.target.value, true)}
              onBlur={() => setStartingInputError(false)}
              onFocus={(e) => handleDurationInput(e.target.value, true)}
            />
          </div>
          <div>
            <label htmlFor="endingTimeInput">End time:</label>
            <p style={{ cursor: "default" }}>{videoDurationToString(convertFromSeconds(endingSecond))}</p>
            <Input
              id="endingTimeInput"
              status={endingInputError ? "error" : ""}
              placeholder="00:00:00.000"
              onChange={(e) => handleDurationInput(e.target.value, false)}
              onBlur={() => setEndingInputError(false)}
              onFocus={(e) => handleDurationInput(e.target.value, false)}
            />
          </div>
        </div>
      </div>
    </div>
  );
}

export default CutSegment;
