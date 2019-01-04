import React, { Component } from "react";

import * as GameUtils from "../gameUtils";
import {
  OccupiedCell,
  GreenOverlay,
  YellowOverlay,
  RedOverlay,
  TYPE_EMPTY,
  TYPE_PLAYER_1,
  TYPE_PLAYER_2
} from "./elements";

const inRectangle = ({
  rowIndex,
  columnIndex,
  selectedRowIndex,
  selectedColumnIndex,
  rectangleHeight,
  rectangleWidth
}) => {
  return (
    selectedRowIndex !== -1 &&
    selectedColumnIndex !== -1 &&
    rowIndex >= selectedRowIndex &&
    rowIndex < selectedRowIndex + rectangleHeight &&
    columnIndex >= selectedColumnIndex &&
    columnIndex < selectedColumnIndex + rectangleWidth
  );
};

class BoardCell extends Component {
  handleCellMouseEnter = () => {
    const { rowIndex, columnIndex, onMouseEnter } = this.props;

    onMouseEnter({ rowIndex, columnIndex });
  };

  handleCellClick = () => {
    const { rowIndex, columnIndex, onClick } = this.props;

    onClick({ rowIndex, columnIndex });
  };

  render() {
    const {
      canDrop,
      value,
      rowIndex,
      columnIndex,
      cellRadius,
      selectedRowIndex,
      selectedColumnIndex,
      rectangleWidth,
      rectangleHeight,
      potentiallyOccupiedCells
    } = this.props;

    const isCellInRectangle = inRectangle({
      rowIndex,
      columnIndex,
      selectedRowIndex,
      selectedColumnIndex,
      rectangleWidth,
      rectangleHeight
    });

    let isPotentiallyOccupied = false;
    if (!isCellInRectangle) {
      isPotentiallyOccupied = potentiallyOccupiedCells.some(cell =>
        inRectangle({
          rowIndex,
          columnIndex,
          selectedRowIndex: cell.rowIndex,
          selectedColumnIndex: cell.columnIndex,
          rectangleWidth,
          rectangleHeight
        })
      );
    }

    return (
      <OccupiedCell
        disabled={false} // add disable logic when not a turn or rectangle doesn't fit
        cellRadius={cellRadius}
        type={
          GameUtils.isOccupiedByPlayerOneCell(value)
            ? TYPE_PLAYER_1
            : GameUtils.isOccupiedByPlayerTwoCell(value)
            ? TYPE_PLAYER_2
            : TYPE_EMPTY
        }
        onMouseEnter={this.handleCellMouseEnter}
        onTouchStart={this.handleCellMouseEnter}
        onTouchMove={this.handleCellMouseEnter}
        onClick={this.handleCellClick}
        onTouchCancel={this.handleCellClick}
      >
        {isCellInRectangle && (canDrop ? <GreenOverlay /> : <RedOverlay />)}
        {GameUtils.isEmptyCell(value) && isPotentiallyOccupied && (
          <YellowOverlay />
        )}
      </OccupiedCell>
    );
  }
}

export default BoardCell;
