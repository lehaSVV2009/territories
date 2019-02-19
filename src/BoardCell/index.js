import React, { Component } from "react";
import * as GameUtils from "territories-core";

import {
  OccupiedCell,
  GreenOverlay,
  YellowOverlay,
  RedOverlay,
  TYPE_EMPTY,
  TYPE_PLAYER_1,
  TYPE_PLAYER_2
} from "./elements";

class BoardCell extends Component {
  state = {
    touchX: -1,
    touchY: -1,
    touchDraggedColumnIndex: -1,
    touchDraggedRowIndex: -1
  };

  handleCellMouseEnter = () => {
    const { rowIndex, columnIndex, onMouseEnter } = this.props;

    onMouseEnter({ rowIndex, columnIndex });
  };

  handleCellClick = () => {
    const { rowIndex, columnIndex, onClick } = this.props;

    onClick({ rowIndex, columnIndex });
  };

  handleCellTouchStart = ({ touches }) => {
    if (touches.length === 1) {
      this.setState({
        touchX: touches[0].clientX,
        touchY: touches[0].clientY
      });
    }
    this.handleCellMouseEnter();
  };

  handleCellTouchMove = ({ touches }) => {
    // Skip double touches
    if (touches.length < 1) {
      return;
    }

    // Skip touch move without touch start (probably impossible scenario)
    const { touchX, touchY } = this.state;
    if (touchX === -1 || touchY === -1) {
      return;
    }

    const { clientX, clientY } = touches[0];
    const { cellRadius } = this.props;
    const cellDiameter = cellRadius * 2;

    const differenceX = Math.floor((touchX - clientX) / cellDiameter);
    const differenceY = Math.floor((touchY - clientY) / cellDiameter);

    // Skip touch move near the same cell
    if (Math.abs(differenceX) === 0 && Math.abs(differenceY) === 0) {
      return;
    }

    const {
      columnIndex,
      rowIndex,
      boardWidth,
      boardHeight,
      onMouseEnter
    } = this.props;
    const newColumnIndex = columnIndex - differenceX;
    const newRowIndex = rowIndex - differenceY;

    // Skip too far touch move
    if (
      newColumnIndex < 0 ||
      newColumnIndex >= boardWidth ||
      newRowIndex < 0 ||
      newRowIndex >= boardHeight
    ) {
      return;
    }

    this.setState({
      touchDraggedColumnIndex: newColumnIndex,
      touchDraggedRowIndex: newRowIndex
    });
    onMouseEnter({
      columnIndex: newColumnIndex,
      rowIndex: newRowIndex
    });
  };

  handleCellTouchEnd = event => {
    if (event) {
      event.preventDefault();
    }

    const { touchDraggedColumnIndex, touchDraggedRowIndex } = this.state;
    const { onClick } = this.props;

    if (touchDraggedColumnIndex !== -1 || touchDraggedRowIndex !== -1) {
      onClick({
        rowIndex: touchDraggedRowIndex,
        columnIndex: touchDraggedColumnIndex
      });
    }
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

    const isCellInRectangle = GameUtils.inRectangle({
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
        GameUtils.inRectangle({
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
        cellRadius={cellRadius}
        type={
          GameUtils.isOccupiedByPlayerOneCell(value)
            ? TYPE_PLAYER_1
            : GameUtils.isOccupiedByPlayerTwoCell(value)
            ? TYPE_PLAYER_2
            : TYPE_EMPTY
        }
        onMouseEnter={this.handleCellMouseEnter}
        onClick={this.handleCellClick}
        onTouchStart={this.handleCellTouchStart}
        onTouchMove={this.handleCellTouchMove}
        onTouchEnd={this.handleCellTouchEnd}
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
