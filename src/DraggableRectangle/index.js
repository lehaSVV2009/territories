import React from "react";
import { findDOMNode } from "react-dom";
import { DragSource } from "react-dnd";

import { StyledRectangle } from "./elements";

export const DND_TYPE = "Rectangle";

const rectangleSource = {
  beginDrag: ({ rows }) => ({ rows })
};

const collect = (connect, monitor) => ({
  connectDragSource: connect.dragSource(),
  isDragging: monitor.isDragging()
});

const DraggableRectangle = ({ isDragging, connectDragSource, rows }) => (
  <StyledRectangle
    rows={rows}
    isDragging={isDragging}
    ref={instance => connectDragSource(findDOMNode(instance))}
  />
);

export default DragSource(DND_TYPE, rectangleSource, collect)(
  DraggableRectangle
);
