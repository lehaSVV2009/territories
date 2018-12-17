import React, { Component } from "react";

import Table, { Cell, Row } from "./Table";

class Board extends Component {
  renderRow = ({
    rows,
    row,
    rowIndex,
    rowRenderer,
    rowClassName,
    cellRenderer,
    cellClassName
  }) => {
    return rowRenderer ? (
      rowRenderer({
        rows,
        value: row,
        rowIndex,
        rowClassName,
        cellRenderer,
        cellClassName
      })
    ) : (
      <Row key={`board-row-${rowIndex}`} className={rowClassName}>
        {row.map((cell, columnIndex) =>
          this.renderCell({
            rows,
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
    rows,
    row,
    rowIndex,
    columnIndex,
    cell,
    cellRenderer,
    cellClassName
  }) => {
    return cellRenderer ? (
      cellRenderer({
        rows,
        row,
        rowIndex,
        columnIndex,
        value: cell,
        cellClassName
      })
    ) : (
      <Cell key={`board-cell-${columnIndex}`} className={cellClassName} />
    );
  };

  render() {
    const {
      rows,
      rowClassName,
      cellClassName,
      rowRenderer,
      cellRenderer,
      ...rest
    } = this.props;
    if (!Array.isArray(rows)) {
      throw new Error(
        "'rows' is a required argument in format [[ 0, 1 ], [0, 'any']]"
      );
    }
    return (
      <Table {...rest}>
        {rows.map((row, rowIndex) =>
          this.renderRow({
            rows,
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
