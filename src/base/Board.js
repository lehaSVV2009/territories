import React, { Component } from "react";

import Table, { Cell, Row } from "./Table";

class Board extends Component {
  renderRow = ({
    row,
    rowIndex,
    rowRenderer,
    rowClassName,
    cellRenderer,
    cellClassName
  }) => {
    return rowRenderer ? (
      rowRenderer({
        row,
        rowIndex,
        rowClassName,
        cellRenderer,
        cellClassName
      })
    ) : (
      <Row key={`board-row-${rowIndex}`} className={rowClassName}>
        {row.map((cell, columnIndex) =>
          this.renderCell({
            row,
            rowIndex,
            columnIndex,
            cell,
            cellRenderer,
            cellClassName
          })
        )}
      </Row>
    );
  };

  renderCell = ({
    row,
    rowIndex,
    columnIndex,
    cell,
    cellRenderer,
    cellClassName
  }) => {
    return cellRenderer ? (
      cellRenderer({
        row,
        rowIndex,
        columnIndex,
        cell,
        cellClassName
      })
    ) : (
      <Cell key={`board-cell-${columnIndex}`} className={cellClassName} />
    );
  };

  render() {
    const {
      data,
      rowClassName,
      cellClassName,
      rowRenderer,
      cellRenderer,
      ...rest
    } = this.props;
    if (!Array.isArray(data)) {
      debugger;
      throw new Error(
        "'data' is a required argument in format [[ 0, 1 ], [0, 'any']]"
      );
    }
    return (
      <Table {...rest}>
        {data.map((row, rowIndex) =>
          this.renderRow({
            row,
            rowIndex,
            rowRenderer,
            rowClassName,
            cellRenderer,
            cellClassName
          })
        )}
      </Table>
    );
  }
}

export default Board;
