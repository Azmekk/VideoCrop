import { Button, Dropdown, Space, type MenuProps } from "antd";
import { useState } from "react";
import { DownOutlined } from '@ant-design/icons';

interface CompressSegmentProps {
    disabled?: boolean;
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
        onClick: handleDropdownMenuClick,
        selectable: true,
    }

    return (
        <div style={{ opacity: props.disabled ? 0.5 : 1, pointerEvents: props.disabled ? 'none' : 'auto', }} className="edit-segment">
            <div>
                <p style={{ fontSize: "1.5em", fontWeight: "bold" }}>Compression</p>
                <Dropdown menu={codecDropdownProps}>
                    <Button>
                        <Space>
                            {dropdownTitle}
                            <DownOutlined />
                        </Space>
                    </Button>
                </Dropdown>
            </div>

        </div>
    );
}

export default CropSegment;
