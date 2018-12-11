import React from "react";
import { findDOMNode } from "react-dom";
import { DropTarget } from "react-dnd";

import {
  isEmptyCell,
  isCapturedByPlayerOneCell,
  isCapturedByPlayerTwoCell
} from "../gameUtils";
import { DND_TYPE } from "../DraggableRectangle";
import {
  CapturedCell,
  TYPE_EMPTY,
  TYPE_PLAYER_1,
  TYPE_PLAYER_2
} from "./elements";

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
  value,
  cellClassName,
  connectDropTarget,
  isOver,
  canDrop
}) => {
  return (
    <CapturedCell
      type={
        isCapturedByPlayerOneCell(value)
          ? TYPE_PLAYER_1
          : isCapturedByPlayerTwoCell(value)
          ? TYPE_PLAYER_2
          : TYPE_EMPTY
      }
      className={cellClassName}
      ref={instance => connectDropTarget(findDOMNode(instance))}
    >
      {isOver && !canDrop && renderOverlay("red")}
      {!isOver && canDrop && renderOverlay("yellow")}
      {isOver && canDrop && renderOverlay("green")}
    </CapturedCell>
  );
};

export default DropTarget(DND_TYPE, cellTarget, collect)(DropzoneCell);
