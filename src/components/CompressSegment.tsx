import { Button, Dropdown, Space, type MenuProps } from "antd";
import { useState } from "react";

interface CompressSegmentProps {
    segmentOpen: boolean;
}
function CropSegment(props: CompressSegmentProps) {
    const [dropdownTitle, setDropdownTitle] = useState("Select a codec");
    const codecDropdownItems: { key: string, label: string }[] = [
        {
            key: "libx264",
            label: "H.264"
        },
        {
            key: "libx265",
            label: "H.265"
        },
        {
            key: "libsvtav1",
            label: "av1"
        }
    ]

    const handleDropdownMenuClick: MenuProps['onClick'] = (e) => {
        setDropdownTitle("Codec: " + codecDropdownItems.find(x => x.key === e.key)?.label || "Select a codec");
    };
    
    const codecDropdownProps: MenuProps = {
        items: codecDropdownItems,
        onClick: handleDropdownMenuClick
    }

    return (
        <div className="edit-segment">
            {props.segmentOpen && (
                <div>
                <p style={{ fontSize: "1.5em", fontWeight: "bold" }}>Compression</p>
                <Dropdown menu={codecDropdownProps}>
                    <Button>
                        <Space>
                            {dropdownTitle}
                        </Space>
                    </Button>
                </Dropdown>
                </div>
            )}
        </div>
    );
}

export default CropSegment;
