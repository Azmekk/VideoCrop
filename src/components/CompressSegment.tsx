import { Button, Checkbox, Dropdown, InputNumber, Radio, Space, type MenuProps } from "antd";
import { useEffect, useState } from "react";
import { DownOutlined } from "@ant-design/icons";
import type { VideoCompressionOptions, VideoEditOptions } from "../Logic/Interfaces";

interface CompressSegmentProps {
  disabled?: boolean;
  onChange: (x: VideoCompressionOptions, enabled: boolean) => void;
}
function CropSegment(props: CompressSegmentProps) {
  const [codecDropdownTitle, setCodecDropdownTitle] = useState("Codec: H.264");
  const [presetDropdownTitle, setPresetDropdownTitle] = useState("Medium (Default)");

  const [segmentEnabled, setSegmentEnabled] = useState(false);
  const [selectedCodec, setSelectedCodec] = useState("libx264");
  const [selectedPreset, setSelectedPreset] = useState("medium");

  const [selectedCRF, setSelectedCRF] = useState(29);
  const [selectedBitrate, setSelectedBitrate] = useState(5550);

  const [selectedQualityOption, setSelectedQualityOption] = useState(1);

  useEffect(() => {
    props.onChange(
      {
        codec: selectedCodec,
        preset: selectedPreset,
        crf: selectedCRF,
        bitrate: selectedBitrate,
        using_crf: selectedQualityOption === 1,
      },
      segmentEnabled,
    );
  }, [selectedCodec, selectedPreset, selectedCRF, selectedBitrate, selectedQualityOption, segmentEnabled]);

  const codecDropdownItems: { key: string; label: string }[] = [
    {
      key: "libx264",
      label: "H.264",
    },
    {
      key: "libx265",
      label: "H.265",
    },
    {
      key: "libsvtav1",
      label: "av1",
    },
  ];

  const libx26xPresets: { key: string; label: string }[] = [
    { key: "ultrafast", label: "Ultra Fast" },
    { key: "superfast", label: "Super Fast" },
    { key: "veryfast", label: "Very Fast" },
    { key: "faster", label: "Faster" },
    { key: "fast", label: "Fast" },
    { key: "medium", label: "Medium (Default)" },
    { key: "slow", label: "Slow" },
    { key: "slower", label: "Slower" },
    { key: "veryslow", label: "Very Slow" },
  ];

  const libaomPresets: { key: string; label: string }[] = [
    { key: "0", label: "0 (Fastest)" },
    { key: "1", label: "1" },
    { key: "2", label: "2" },
    { key: "3", label: "3" },
    { key: "4", label: "4" },
    { key: "5", label: "5" },
    { key: "6", label: "6" },
    { key: "7", label: "7" },
    { key: "8", label: "8 (Default)" },
    { key: "9", label: "9" },
    { key: "10", label: "10" },
    { key: "11", label: "11" },
    { key: "12", label: "12 (Slowest)" },
  ];

  const determinePreset = () => {
    if (selectedCodec === "libx264" || selectedCodec === "libx265") {
      return libx26xPresets;
    }

    if (selectedCodec === "libsvtav1") {
      return libaomPresets;
    }

    return [];
  };

  const getDefaultPreset = (selectedCodec: string) => {
    if (selectedCodec === "libx264" || selectedCodec === "libx265") {
      return "medium";
    }
    if (selectedCodec === "libsvtav1") {
      return "8";
    }
    return "";
  };

  const handleCodecDropdownMenuClick: MenuProps["onClick"] = (e) => {
    setCodecDropdownTitle(`Codec: ${codecDropdownItems.find((x) => x.key === e.key)?.label}` || "Select a codec");
    setSelectedCodec(e.key);
  };

  useEffect(() => {
    const defaultPreset = getDefaultPreset(selectedCodec);
    setPresetDropdownTitle(`Preset: ${determinePreset().find((x) => x.key === defaultPreset)?.label}` || "Select a preset");
    setSelectedPreset(defaultPreset);
  }, [selectedCodec]);

  const handlePresetDropdownMenuClick: MenuProps["onClick"] = (e) => {
    setPresetDropdownTitle(`Preset: ${determinePreset().find((x) => x.key === e.key)?.label}` || "Select a preset");
    setSelectedPreset(e.key);
  };

  const codecDropdownProps: MenuProps = {
    items: codecDropdownItems,
    onClick: handleCodecDropdownMenuClick,
    selectable: true,
  };

  const presetDropdownProps: MenuProps = {
    items: determinePreset(),
    onClick: handlePresetDropdownMenuClick,
    selectable: true,
  };

  return (
    <div className={props.disabled ? "disabled" : ""}>
      <div>
        <div style={{ marginBottom: "25px", display: "flex", gap: "10px" }}>
          <div style={{ fontSize: "1.2em", fontWeight: "bold" }}>Compression</div>
          <Checkbox defaultChecked={false} onChange={(e) => setSegmentEnabled(e.target.checked)} />
        </div>

        <div className={segmentEnabled ? "" : "disabled"} style={{ gap: "10px", display: "flex", flexDirection: "column" }}>
          <div>
            <Dropdown trigger={["click"]} menu={codecDropdownProps}>
              <Button type="primary">
                <Space>
                  {codecDropdownTitle}
                  <DownOutlined />
                </Space>
              </Button>
            </Dropdown>
          </div>

          <div style={{ marginBottom: "20px" }}>
            <Dropdown disabled={selectedCodec === ""} trigger={["click"]} menu={presetDropdownProps}>
              <Button>
                <Space>
                  {presetDropdownTitle}
                  <DownOutlined />
                </Space>
              </Button>
            </Dropdown>
          </div>

          <Radio.Group onChange={(e) => setSelectedQualityOption(e.target.value)} value={selectedQualityOption}>
            <Radio value={1}>CRF</Radio>
            <Radio value={2}>Bitrate</Radio>
          </Radio.Group>

          <div>
            {selectedQualityOption === 1 ? (
              <InputNumber
                style={{ maxWidth: "100px" }}
                min={0}
                max={51}
                addonAfter={"crf"}
                type="number"
                placeholder="CRF"
                defaultValue={29}
                value={selectedCRF}
                onChange={(e) => setSelectedCRF(e ?? 0)}
              />
            ) : (
              <InputNumber
                style={{ maxWidth: "200px" }}
                min={100}
                max={250000}
                addonAfter={"kbps"}
                type="number"
                placeholder="Bitrate"
                defaultValue={5500}
                value={selectedBitrate}
                onChange={(e) => setSelectedBitrate(e ?? 0)}
              />
            )}
          </div>
        </div>
      </div>
    </div>
  );
}

export default CropSegment;
