import React, { Component } from "react";
import * as GameUtils from "territories-core";

import BoardCell from "../BoardCell";
import Rectangle from "../libs/territories-ui/Rectangle";

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
      const potentiallyOccupiedCells = GameUtils.findPotentiallyOccupiedRectangles(
        {
          currentPlayer,
          rectangleHeight,
          rectangleWidth,
          rows
        }
      );
      this.setState({ potentiallyOccupiedCells });
    }
  }

  handleCellMouseEnter = ({ rowIndex, columnIndex }) => {
    const {
      rows,
      rectangleHeight,
      rectangleWidth,
      currentPlayer,
      disabled
    } = this.props;

    if (disabled) {
      return;
    }

    const canDrop = GameUtils.canDropRectangle({
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

  handleBoardMouseLeave = () => {
    this.setState({ selectedRowIndex: -1, selectedColumnIndex: -1 });
  };

  handleDropRectangle = ({ rowIndex, columnIndex }) => {
    const { disabled } = this.props;
    const { canDrop } = this.state;
    const { onDropRectangle, rectangleHeight, rectangleWidth } = this.props;

    if (!disabled && canDrop) {
      onDropRectangle({
        rowIndex,
        columnIndex,
        rectangleHeight,
        rectangleWidth
      });
    }

    this.handleBoardMouseLeave();
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
            boardHeight={rows.length}
            boardWidth={rows.length > 0 ? rows[0].length : 0}
            onMouseEnter={this.handleCellMouseEnter}
            onClick={this.handleDropRectangle}
          />
        )}
        onMouseLeave={this.handleBoardMouseLeave}
      />
    );
  }
}

export default Board;
