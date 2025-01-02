import {
	Button,
	Checkbox,
	Dropdown,
	InputNumber,
	Radio,
	Space,
	type MenuProps,
} from "antd";
import { useEffect, useState } from "react";
import { DownOutlined } from "@ant-design/icons";

interface CompressSegmentProps {
	disabled?: boolean;
}
function CropSegment(props: CompressSegmentProps) {
	const [codecDropdownTitle, setCodecDropdownTitle] =
		useState("Select a codec");
	const [presetDropdownTitle, setPresetDropdownTitle] =
		useState("Select a preset");

	const [segmentEnabled, setSegmentEnabled] = useState(false);
	const [selectedCodec, setSelectedCodec] = useState("");
	const [selectedPreset, setSelectedPreset] = useState("");

	const [selectedCRF, setSelectedCRF] = useState(0);
	const [selectedBitrate, setSelectedBitrate] = useState(0);

	const [selectedQualityOption, setSelectedQualityOption] = useState(1);

	useEffect(() => {
		setSelectedCRF(27);
		setSelectedBitrate(5500);
	}, []);

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
		setCodecDropdownTitle(
			`Codec: ${codecDropdownItems.find((x) => x.key === e.key)?.label}` ||
				"Select a codec",
		);
		setSelectedCodec(e.key);

		const defaultPreset = getDefaultPreset(e.key);
		setPresetDropdownTitle(
			`Preset: ${determinePreset().find((x) => x.key === defaultPreset)?.label}` ||
				"Select a preset",
		);
		setSelectedPreset(defaultPreset);
	};

	const handlePresetDropdownMenuClick: MenuProps["onClick"] = (e) => {
		setPresetDropdownTitle(
			`Preset: ${determinePreset().find((x) => x.key === e.key)?.label}` ||
				"Select a preset",
		);
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
		<div
			style={{
				opacity: props.disabled ? 0.5 : 1,
				pointerEvents: props.disabled ? "none" : "auto",
			}}
		>
			<div>
				<div style={{ marginBottom: "25px", display: "flex", gap: "10px" }}>
					<div style={{ fontSize: "1.2em", fontWeight: "bold" }}>
						Compression
					</div>
					<Checkbox
						defaultChecked={false}
						onChange={(e) => setSegmentEnabled(e.target.checked)}
					/>
				</div>

				<div
					className={segmentEnabled ? "" : "disabled"}
					style={{ gap: "10px", display: "flex", flexDirection: "column" }}
				>
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
						<Dropdown
							disabled={selectedCodec === ""}
							trigger={["click"]}
							menu={presetDropdownProps}
						>
							<Button>
								<Space>
									{presetDropdownTitle}
									<DownOutlined />
								</Space>
							</Button>
						</Dropdown>
					</div>

					<Radio.Group
						onChange={(e) => setSelectedQualityOption(e.target.value)}
						value={selectedQualityOption}
					>
						<Radio value={1}>CRF</Radio>
						<Radio value={2}>Bitrate</Radio>
					</Radio.Group>

					<div>
						{selectedQualityOption === 1 ? (
							<InputNumber
								type="number"
								placeholder="CRF"
								defaultValue={27}
								onChange={(e) => setSelectedCRF(e ?? 0)}
							/>
						) : (
							<InputNumber
								addonAfter={"kbps"}
								type="number"
								placeholder="Bitrate"
								defaultValue={5500}
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
