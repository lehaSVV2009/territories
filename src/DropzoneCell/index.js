import React from "react";
import { findDOMNode } from "react-dom";
import { DropTarget } from "react-dnd";

import { DND_TYPE } from "../DraggableBoard";
import { Cell } from "../base/Table";

const cellTarget = {
  canDrop: props => {
    return true;
  },

  drop: props => {
    console.log(props.size, props.x, props.y);
  }
};

const collect = (connect, monitor) => ({
  connectDropTarget: connect.dropTarget(),
  isOver: monitor.isOver(),
  canDrop: monitor.canDrop()
});

const renderOverlay = color => {
  return (
    <div
      style={{
        position: "absolute",
        top: 0,
        left: 0,
        height: "100%",
        width: "100%",
        zIndex: 1,
        opacity: 0.5,
        backgroundColor: color
      }}
    />
  );
};

const DropzoneCell = ({
  cellClassName,
  columnIndex,
  connectDropTarget,
  isOver,
  canDrop
}) => {
  return (
    <Cell
      key={`board-cell-${columnIndex}`}
      className={cellClassName}
      style={{
        position: "relative"
      }}
      ref={instance => connectDropTarget(findDOMNode(instance))}
    >
      {isOver && !canDrop && renderOverlay("red")}
      {!isOver && canDrop && renderOverlay("yellow")}
      {isOver && canDrop && renderOverlay("green")}
    </Cell>
  );
};

export default DropTarget(DND_TYPE, cellTarget, collect)(DropzoneCell);
