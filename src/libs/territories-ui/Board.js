import React, { Component } from "react";

import Table, { Cell, Row } from "./Table";

class Board extends Component {
  renderRow = ({
    rows,
    row,
    rowIndex,
    rowRenderer,
    rowClassName,
    rowStyle,
    cellRenderer,
    cellRadius,
    cellClassName,
    cellStyle
  }) => {
    return rowRenderer ? (
      rowRenderer({
        rows,
        value: row,
        rowIndex,
        rowClassName,
        rowStyle,
        cellRenderer,
        cellRadius,
        cellClassName,
        cellStyle
      })
    ) : (
      <Row
        key={`board-row-${rowIndex}`}
        className={rowClassName}
        style={rowStyle}
      >
        {row.map((cell, columnIndex) =>
          this.renderCell({
            rows,
            row,
            rowIndex,
            columnIndex,
            cell,
            cellRenderer,
            cellRadius,
            cellClassName,
            cellStyle
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
    cellRadius,
    cellClassName,
    cellStyle
  }) => {
    return cellRenderer ? (
      cellRenderer({
        rows,
        row,
        rowIndex,
        columnIndex,
        value: cell,
        cellRadius,
        cellClassName,
        cellStyle
      })
    ) : (
      <Cell
        key={`board-cell-${columnIndex}`}
        className={cellClassName}
        style={cellStyle}
        cellRadius={cellRadius}
      />
    );
  };

  render() {
    const {
      rows,
      rowClassName,
      rowStyle,
      cellClassName,
      cellStyle,
      cellRadius,
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
            rowStyle,
            cellRenderer,
            cellRadius,
            cellClassName,
            cellStyle
          })
        )}
      </Table>
    );
  }
}

export default Board;
