// TODO import from territories-core
export const CELL_TYPE = {
  EMPTY: "EMPTY",
  OCCUPIED_BY_PLAYER_1: "OCCUPIED_BY_PLAYER_1",
  OCCUPIED_BY_PLAYER_2: "OCCUPIED_BY_PLAYER_2"
};

export const isEmptyCell = type => {
  return type === CELL_TYPE.EMPTY;
};
export const isOccupiedByPlayerOneCell = type => {
  return type === CELL_TYPE.OCCUPIED_BY_PLAYER_1;
};
export const isOccupiedByPlayerTwoCell = type => {
  return type === CELL_TYPE.OCCUPIED_BY_PLAYER_2;
};

export const PLAYER_1 = "0";
export const PLAYER_2 = "1";

export const isPlayer1 = player => {
  return player === PLAYER_1;
};
export const isPlayer2 = player => {
  return player === PLAYER_2;
};

export const canDropRectangle = ({
  rowIndex,
  columnIndex,
  value,
  rows,
  rectangleHeight,
  rectangleWidth,
  currentPlayer
}) => {
  // Check if cell is not occupied
  if (!isEmptyCell(value)) {
    return false;
  }

  const player1 = isPlayer1(currentPlayer);
  const player2 = isPlayer2(currentPlayer);

  // Check if it is the 1st turn for player 1
  if (rowIndex === 0 && columnIndex === 0 && player1) {
    return true;
  }

  // Check if it is the 1st turn for player 2
  if (
    rowIndex === rows.length - rectangleHeight &&
    rows.length > 0 &&
    columnIndex === rows[0].length - rectangleWidth &&
    player2
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
    isOccupiedFunction: player1
      ? isOccupiedByPlayerOneCell
      : isOccupiedByPlayerTwoCell
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
      if (!isEmptyCell(rows[currentRowIndex][currentColumnIndex])) {
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

export const inRectangle = ({
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
