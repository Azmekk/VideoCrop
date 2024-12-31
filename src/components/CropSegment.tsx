import { VideoCropPoints } from "../Utils";

interface CropSegmentProps{
    videoCropPoints: VideoCropPoints;
    segmentOpen: boolean;
}
function CropSegment(props: CropSegmentProps){
    return (
        <div className="edit-segment">
        </div>
    );
}

export default CropSegment;