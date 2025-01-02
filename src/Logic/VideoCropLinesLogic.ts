import { HoveringOver } from "./Enums";
import type { VideoCropLineDisplacements } from "./Interfaces";

export const determineIfHoveringOverLine = (
  e: React.MouseEvent<HTMLCanvasElement, MouseEvent>,
  canvasLineDisplacementRef: VideoCropLineDisplacements,
): HoveringOver | undefined => {
  const canvasRect = e.currentTarget.getBoundingClientRect();
  const offsetX = e.clientX - canvasRect.left;
  const offsetY = e.clientY - canvasRect.top;

  console.log(offsetX, offsetY);
  const tolerance = 15;

  const { left, right, top, bottom } = canvasLineDisplacementRef;

  const isHoveringOverLeftLine = Math.abs(offsetX - left) < tolerance;
  const isHoveringOverRightLine = Math.abs(offsetX - (canvasRect.width - right)) < tolerance;
  const isHoveringOverTopLine = Math.abs(offsetY - top) < tolerance;
  const isHoveringOverBottomLine = Math.abs(offsetY - (canvasRect.height - bottom)) < tolerance;

  const isHoveringOverTopLeftCorner = isHoveringOverTopLine && isHoveringOverLeftLine;
  const isHoveringOverTopRightCorner = isHoveringOverTopLine && isHoveringOverRightLine;
  const isHoveringOverBottomLeftCorner = isHoveringOverBottomLine && isHoveringOverLeftLine;
  const isHoveringOverBottomRightCorner = isHoveringOverBottomLine && isHoveringOverRightLine;

  if (isHoveringOverTopLeftCorner) {
    console.log("Hovering over the top-left corner");
    return HoveringOver.TopLeftCorner;
  }
  if (isHoveringOverTopRightCorner) {
    console.log("Hovering over the top-right corner");
    return HoveringOver.TopRightCorner;
  }
  if (isHoveringOverBottomLeftCorner) {
    console.log("Hovering over the bottom-left corner");
    return HoveringOver.BottomLeftCorner;
  }
  if (isHoveringOverBottomRightCorner) {
    console.log("Hovering over the bottom-right corner");
    return HoveringOver.BottomRightCorner;
  }

  if (isHoveringOverLeftLine) {
    console.log("Hovering over the left line");
    return HoveringOver.Left;
  }
  if (isHoveringOverRightLine) {
    console.log("Hovering over the right line");
    return HoveringOver.Right;
  }
  if (isHoveringOverTopLine) {
    console.log("Hovering over the top line");
    return HoveringOver.Top;
  }
  if (isHoveringOverBottomLine) {
    console.log("Hovering over the bottom line");
    return HoveringOver.Bottom;
  }

  return undefined;
};

export const updateCanvasLineDisplacement = (
  canvasLineDisplacementRef: VideoCropLineDisplacements,
  clickedLine: HoveringOver,
  e: React.MouseEvent<HTMLCanvasElement, MouseEvent>,
) => {
  const minimumSeparation = 25;
  switch (clickedLine) {
    case undefined:
      return;
    case HoveringOver.Left: {
      const updatedLeftValue = e.clientX - e.currentTarget.getBoundingClientRect().left;

      if (updatedLeftValue + canvasLineDisplacementRef.right + minimumSeparation > e.currentTarget.width) {
        return;
      }
      canvasLineDisplacementRef.left = updatedLeftValue;
      break;
    }
    case HoveringOver.Right: {
      const updatedRightValue = e.currentTarget.getBoundingClientRect().right - e.clientX;

      if (updatedRightValue + canvasLineDisplacementRef.left + minimumSeparation > e.currentTarget.width) {
        return;
      }
      canvasLineDisplacementRef.right = updatedRightValue;
      break;
    }
    case HoveringOver.Top: {
      const updatedTopValue = e.clientY - e.currentTarget.getBoundingClientRect().top;

      if (updatedTopValue + canvasLineDisplacementRef.bottom + minimumSeparation > e.currentTarget.height) {
        return;
      }
      canvasLineDisplacementRef.top = updatedTopValue;
      break;
    }
    case HoveringOver.Bottom: {
      const updatedBottomValue = e.currentTarget.getBoundingClientRect().bottom - e.clientY;

      if (updatedBottomValue + canvasLineDisplacementRef.top + minimumSeparation > e.currentTarget.height) {
        return;
      }
      canvasLineDisplacementRef.bottom = updatedBottomValue;
      break;
    }
    case HoveringOver.TopLeftCorner: {
      const updatedLeftValue = e.clientX - e.currentTarget.getBoundingClientRect().left;
      const updatedTopValue = e.clientY - e.currentTarget.getBoundingClientRect().top;

      if (
        updatedLeftValue + canvasLineDisplacementRef.right + minimumSeparation > e.currentTarget.width ||
        updatedTopValue + canvasLineDisplacementRef.bottom + minimumSeparation > e.currentTarget.height
      ) {
        return;
      }
      canvasLineDisplacementRef.left = updatedLeftValue;
      canvasLineDisplacementRef.top = updatedTopValue;
      break;
    }
    case HoveringOver.TopRightCorner: {
      const updatedRightValue = e.currentTarget.getBoundingClientRect().right - e.clientX;
      const updatedTopValue = e.clientY - e.currentTarget.getBoundingClientRect().top;

      if (
        updatedRightValue + canvasLineDisplacementRef.left + minimumSeparation > e.currentTarget.width ||
        updatedTopValue + canvasLineDisplacementRef.bottom + minimumSeparation > e.currentTarget.height
      ) {
        return;
      }
      canvasLineDisplacementRef.right = updatedRightValue;
      canvasLineDisplacementRef.top = updatedTopValue;
      break;
    }
    case HoveringOver.BottomLeftCorner: {
      const updatedLeftValue = e.clientX - e.currentTarget.getBoundingClientRect().left;
      const updatedBottomValue = e.currentTarget.getBoundingClientRect().bottom - e.clientY;

      if (
        updatedLeftValue + canvasLineDisplacementRef.right + minimumSeparation > e.currentTarget.width ||
        updatedBottomValue + canvasLineDisplacementRef.top + minimumSeparation > e.currentTarget.height
      ) {
        return;
      }
      canvasLineDisplacementRef.left = updatedLeftValue;
      canvasLineDisplacementRef.bottom = updatedBottomValue;
      break;
    }
    case HoveringOver.BottomRightCorner: {
      const updatedRightValue = e.currentTarget.getBoundingClientRect().right - e.clientX;
      const updatedBottomValue = e.currentTarget.getBoundingClientRect().bottom - e.clientY;

      if (
        updatedRightValue + canvasLineDisplacementRef.left + minimumSeparation > e.currentTarget.width ||
        updatedBottomValue + canvasLineDisplacementRef.top + minimumSeparation > e.currentTarget.height
      ) {
        return;
      }
      canvasLineDisplacementRef.right = updatedRightValue;
      canvasLineDisplacementRef.bottom = updatedBottomValue;
      break;
    }
  }
};
