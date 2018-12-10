import React from "react";
import { findDOMNode } from "react-dom";
import { DragSource } from "react-dnd";

import { StyledBoard } from "./elements";

export const DND_TYPE = "Board";

const boardSource = {
  beginDrag: ({ size }) => ({ size })
};

const collect = (connect, monitor) => ({
  connectDragSource: connect.dragSource(),
  isDragging: monitor.isDragging()
});

const DraggableBoard = ({ isDragging, connectDragSource, data }) => (
  <StyledBoard
    data={data}
    isDragging={isDragging}
    ref={instance => connectDragSource(findDOMNode(instance))}
  />
);

export default DragSource(DND_TYPE, boardSource, collect)(DraggableBoard);
