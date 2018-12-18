import React from "react";
import { findDOMNode } from "react-dom";
import { DropTarget } from "react-dnd";

import * as GameUtils from "../gameUtils";
import { DND_TYPE } from "../DraggableRectangle";
import {
  OccupiedCell,
  TYPE_EMPTY,
  TYPE_PLAYER_1,
  TYPE_PLAYER_2
} from "./elements";

const canDropRectangle = ({
  rowIndex,
  columnIndex,
  value,
  rows,
  rectangleHeight,
  rectangleWidth,
  currentPlayer
}) => {
  // Check if cell is not occupied
  if (!GameUtils.isEmptyCell(value)) {
    return false;
  }

  const isPlayer1 = GameUtils.isPlayer1(currentPlayer);
  const isPlayer2 = GameUtils.isPlayer2(currentPlayer);

  // Check if it is the 1st turn for player 1
  if (rowIndex === 0 && columnIndex === 0 && isPlayer1) {
    return true;
  }

  // Check if it is the 1st turn for player 2
  if (
    rowIndex === rows.length - 1 &&
    rows.length > 0 &&
    columnIndex === rows[0].length - 1 &&
    isPlayer2
  ) {
    return true;
  }

  return hasCapturedNeighbour({
    rows,
    rowIndex,
    columnIndex,
    isCapturedFunction: isPlayer1
      ? GameUtils.isOccupiedByPlayerOneCell
      : GameUtils.isOccupiedByPlayerTwoCell
  });
};

const hasCapturedNeighbour = ({
  rows,
  rowIndex,
  columnIndex,
  isCapturedFunction
}) => {
  if (rows.length === 0 || rows[0].length === 0) {
    return false;
  }
  const rowFirstIndex = 0;
  const columnFirstIndex = 0;
  const rowLastIndex = rows.length - 1;
  const columnLastIndex = rows[0].length - 1;

  // Check if left cell exists and is occupied by player1
  if (
    columnIndex - 1 >= columnFirstIndex &&
    isCapturedFunction(rows[rowIndex][columnIndex - 1])
  ) {
    return true;
  }
  // Check if right cell exists and is occupied by player1
  if (
    columnIndex + 1 <= columnLastIndex &&
    isCapturedFunction(rows[rowIndex][columnIndex + 1])
  ) {
    return true;
  }
  // Or check if top cell exists and is occupied by player1
  if (
    rowIndex - 1 >= rowFirstIndex &&
    isCapturedFunction(rows[rowIndex - 1][columnIndex])
  ) {
    return true;
  }
  // Check if bottom cell exists and is occupied by player1
  if (
    rowIndex + 1 <= rowLastIndex &&
    isCapturedFunction(rows[rowIndex + 1][columnIndex])
  ) {
    return true;
  }
  return false;
};

const cellTarget = {
  canDrop: (props, monitor) => {
    const { rowIndex, columnIndex, value, rows, currentPlayer } = props;
    // will be useful for future
    const { rows: rectangleRows } = monitor.getItem();
    return canDropRectangle({
      rowIndex,
      columnIndex,
      value,
      rows,
      currentPlayer,
      rectangleHeight: rectangleRows.length,
      rectangleWidth: rectangleRows[0].length
    });
  },

  drop: (props, monitor) => {
    const { rowIndex, columnIndex } = props;
    const { rows: rectangleRows } = monitor.getItem();
    props.onDropCell({
      rowIndex,
      columnIndex,
      rectangleHeight: rectangleRows.length,
      rectangleWidth: rectangleRows[0].length
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
    <OccupiedCell
      type={
        GameUtils.isOccupiedByPlayerOneCell(value)
          ? TYPE_PLAYER_1
          : GameUtils.isOccupiedByPlayerTwoCell(value)
          ? TYPE_PLAYER_2
          : TYPE_EMPTY
      }
      className={cellClassName}
      ref={instance => connectDropTarget(findDOMNode(instance))}
    >
      {isOver && !canDrop && renderOverlay("red")}
      {!isOver && canDrop && renderOverlay("yellow")}
      {isOver && canDrop && renderOverlay("green")}
    </OccupiedCell>
  );
};

export default DropTarget(DND_TYPE, cellTarget, collect)(DropzoneCell);
