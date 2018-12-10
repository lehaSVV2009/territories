import React from "react";
import { findDOMNode } from "react-dom";
import { DropTarget } from "react-dnd";

import { isEmptyCell } from "../Game";
import { DND_TYPE } from "../DraggableRectangle";
import { Cell } from "../base/Table";

const canDropRectangle = ({ value }) => {
  return isEmptyCell(value);
};

const cellTarget = {
  canDrop: (props, monitor) => {
    const { value } = props;
    // will be useful for future
    // const { rows: rectangleRows } = monitor.getItem();
    return canDropRectangle({ value });
  },

  drop: (props, monitor) => {
    const { rowIndex, columnIndex } = props;
    const { rows } = monitor.getItem();
    props.onDropCell({
      rowIndex,
      columnIndex,
      rectangleHeight: rows.length,
      rectangleWidth: rows[0].length
    });
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
  connectDropTarget,
  isOver,
  canDrop
}) => {
  return (
    <Cell
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
