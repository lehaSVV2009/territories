import React, { Component } from "react";

import BoardCell from "../BoardCell";
import * as GameUtils from "../gameUtils";
import Rectangle from "../libs/territories-ui/Rectangle";

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
    rowIndex === rows.length - rectangleHeight &&
    rows.length > 0 &&
    columnIndex === rows[0].length - rectangleWidth &&
    isPlayer2
  ) {
    return true;
  }

  if (
    isOccupiedOrOutborderRectangle({
      rows,
      rowIndex,
      columnIndex,
      rectangleHeight,
      rectangleWidth
    })
  ) {
    return false;
  }

  return hasOccupiedNeighbour({
    rows,
    rowIndex,
    columnIndex,
    rectangleHeight,
    rectangleWidth,
    isOccupiedFunction: isPlayer1
      ? GameUtils.isOccupiedByPlayerOneCell
      : GameUtils.isOccupiedByPlayerTwoCell
  });
};

const isOccupiedOrOutborderRectangle = ({
  rows,
  rowIndex,
  columnIndex,
  rectangleHeight,
  rectangleWidth
}) => {
  // TODO probably try to replace with JS counting for
  for (let i = 0; i < rectangleHeight; i++) {
    const currentRowIndex = rowIndex + i;
    // Check if rectangle is not out of bottom border
    if (currentRowIndex >= rows.length) {
      return true;
    }
    for (let j = 0; j < rectangleWidth; j++) {
      const currentColumnIndex = columnIndex + j;
      // Check if rectangle is not out of right border
      if (currentColumnIndex >= rows[currentRowIndex].length) {
        return true;
      }
      // Check if all the cells are not already occupied
      if (!GameUtils.isEmptyCell(rows[currentRowIndex][currentColumnIndex])) {
        return true;
      }
    }
  }

  return false;
};

const hasOccupiedNeighbour = ({
  rows,
  rowIndex,
  columnIndex,
  rectangleHeight,
  rectangleWidth,
  isOccupiedFunction
}) => {
  if (rows.length === 0 || rows[0].length === 0) {
    return false;
  }
  const rowFirstIndex = 0;
  const columnFirstIndex = 0;
  const rowLastIndex = rows.length - 1;
  const columnLastIndex = rows[0].length - 1;

  // Check if any cell exists on the LEFT of rectangle
  // and occupied by current player
  const leftNeighbourColumnIndex = columnIndex - 1;
  if (leftNeighbourColumnIndex >= columnFirstIndex) {
    for (let index = 0; index < rectangleHeight; ++index) {
      if (
        isOccupiedFunction(rows[rowIndex + index][leftNeighbourColumnIndex])
      ) {
        return true;
      }
    }
  }

  // Check if any cell exists on the RIGHT of rectangle
  // and is occupied by current player
  const rightNeighbourColumnIndex = columnIndex + rectangleWidth;
  if (rightNeighbourColumnIndex <= columnLastIndex) {
    for (let index = 0; index < rectangleHeight; ++index) {
      if (
        isOccupiedFunction(rows[rowIndex + index][rightNeighbourColumnIndex])
      ) {
        return true;
      }
    }
  }

  // Check if any cell exists on the TOP of rectangle
  // and is occupied by current player
  const topNeighbourRowIndex = rowIndex - 1;
  if (topNeighbourRowIndex >= rowFirstIndex) {
    for (let index = 0; index < rectangleWidth; ++index) {
      if (isOccupiedFunction(rows[topNeighbourRowIndex][columnIndex + index])) {
        return true;
      }
    }
  }

  // Check if any cell exists on the BOTTOM of rectangle
  // and is occupied by current player
  const bottomNeighbourRowIndex = rowIndex + rectangleHeight;
  if (bottomNeighbourRowIndex <= rowLastIndex) {
    for (let index = 0; index < rectangleWidth; ++index) {
      if (
        isOccupiedFunction(rows[bottomNeighbourRowIndex][columnIndex + index])
      ) {
        return true;
      }
    }
  }

  return false;
};

class Board extends Component {
  state = {
    selectedRowIndex: -1,
    selectedColumnIndex: -1,
    canDrop: false,
    potentiallyOccupiedCells: []
  };

  componentDidUpdate(prevProps) {
    const { currentPlayer, rectangleHeight, rectangleWidth, rows } = this.props;
    if (
      prevProps.rectangleHeight !== rectangleHeight ||
      prevProps.rectangleWidth !== rectangleWidth ||
      prevProps.currentPlayer !== currentPlayer
    ) {
      const potentiallyOccupiedCells = [];
      rows.forEach((row, rowIndex) => {
        row.forEach((column, columnIndex) => {
          if (
            canDropRectangle({
              rowIndex,
              columnIndex,
              value: rows[rowIndex][columnIndex],
              rows,
              rectangleHeight,
              rectangleWidth,
              currentPlayer
            })
          ) {
            potentiallyOccupiedCells.push({ rowIndex, columnIndex });
          }
        });
      });
      this.setState({ potentiallyOccupiedCells });
    }
  }

  handleCellMouseEnter = ({ rowIndex, columnIndex }) => {
    const { rows, rectangleHeight, rectangleWidth, currentPlayer } = this.props;
    const canDrop = canDropRectangle({
      rowIndex,
      columnIndex,
      value: rows[rowIndex][columnIndex],
      rows,
      rectangleHeight,
      rectangleWidth,
      currentPlayer
    });
    this.setState({
      selectedRowIndex: rowIndex,
      selectedColumnIndex: columnIndex,
      canDrop
    });
  };

  handleCellMouseLeave = () => {
    this.setState({ selectedRowIndex: -1, selectedColumnIndex: -1 });
  };

  handleDropRectangle = ({ rowIndex, columnIndex }) => {
    const { canDrop } = this.state;
    const { onDropRectangle, rectangleHeight, rectangleWidth } = this.props;

    if (canDrop) {
      onDropRectangle({
        rowIndex,
        columnIndex,
        rectangleHeight,
        rectangleWidth
      });
    }

    this.handleCellMouseLeave();
  };

  render() {
    const { rows, cellRadius, rectangleWidth, rectangleHeight } = this.props;
    const {
      selectedRowIndex,
      selectedColumnIndex,
      canDrop,
      potentiallyOccupiedCells
    } = this.state;
    return (
      <Rectangle
        rows={rows}
        cellRenderer={({ value, rowIndex, columnIndex }) => (
          <BoardCell
            key={`board-${rowIndex}-${columnIndex}`}
            canDrop={canDrop}
            value={value}
            rowIndex={rowIndex}
            columnIndex={columnIndex}
            selectedRowIndex={selectedRowIndex}
            selectedColumnIndex={selectedColumnIndex}
            rectangleHeight={rectangleHeight}
            rectangleWidth={rectangleWidth}
            cellRadius={cellRadius}
            potentiallyOccupiedCells={potentiallyOccupiedCells}
            onMouseEnter={this.handleCellMouseEnter}
            onMouseLeave={this.handleCellMouseLeave}
            onClick={this.handleDropRectangle}
          />
        )}
      />
    );
  }
}

export default Board;
