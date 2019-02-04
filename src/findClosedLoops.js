/**
 * Slow Flood-Fill algorithm is used to find closed loops/contours
 * @returns list of closed loops. Loop consists of all the cells inside the loop and values of neighbours
 */
export default ({ matrix, loopHabitantsFilter }) => {
  if (!Array.isArray(matrix) || matrix.some(row => !Array.isArray(row))) {
    throw new Error("Matrix is not valid. Must be an array of arrays");
  }
  if (!loopHabitantsFilter) {
    loopHabitantsFilter = () => true;
  }

  const loops = [];

  const checkedCells = [];
  matrix.forEach((row, rowIndex) => {
    row.forEach((column, columnIndex) => {
      // Skip already checked cells
      if (
        checkedCells.some(
          cell => rowIndex === cell.rowIndex && columnIndex === cell.columnIndex
        )
      ) {
        return;
      }

      // Skip cell which are not valid for loop habitants
      if (!loopHabitantsFilter({ value: matrix[rowIndex][columnIndex] })) {
        return;
      }

      const loop = calculateLoopContext({
        matrix,
        rowIndex: rowIndex,
        columnIndex: columnIndex,
        loopHabitantsFilter
      });

      loop.cells.forEach(cell => {
        checkedCells.push({
          rowIndex: cell.rowIndex,
          columnIndex: cell.columnIndex
        });
      });

      loops.push(loop);
    });
  });

  return loops;
};

const calculateLoopContext = ({
  matrix,
  rowIndex,
  columnIndex,
  loopHabitantsFilter,
  loop
}) => {
  if (!loop) {
    loop = {
      cells: [],
      neigboursValues: []
    };
  }

  if (
    loop.cells.some(
      cell => cell.rowIndex === rowIndex && cell.columnIndex === columnIndex
    )
  ) {
    return loop;
  }
  loop.cells.push({ rowIndex, columnIndex });

  const matrixWidth = matrix.length;
  const matrixHeight = matrix.length === 0 ? 0 : matrix[0].length;

  if (rowIndex > 0) {
    // left
    const leftNeigbour = matrix[rowIndex - 1][columnIndex];
    if (loopHabitantsFilter({ value: leftNeigbour })) {
      loop = calculateLoopContext({
        matrix,
        rowIndex: rowIndex - 1,
        columnIndex: columnIndex,
        loop,
        loopHabitantsFilter
      });
    } else if (!loop.neigboursValues.includes(leftNeigbour)) {
      loop.neigboursValues.push(leftNeigbour);
    }
  }

  if (columnIndex > 0) {
    // up
    const topNeighbour = matrix[rowIndex][columnIndex - 1];
    if (loopHabitantsFilter({ value: topNeighbour })) {
      loop = calculateLoopContext({
        matrix,
        rowIndex,
        columnIndex: columnIndex - 1,
        loop,
        loopHabitantsFilter
      });
    } else if (!loop.neigboursValues.includes(topNeighbour)) {
      loop.neigboursValues.push(topNeighbour);
    }
  }

  if (rowIndex < matrixWidth - 1) {
    // right
    const rightNeighbour = matrix[rowIndex + 1][columnIndex];
    if (loopHabitantsFilter({ value: rightNeighbour })) {
      loop = calculateLoopContext({
        matrix,
        rowIndex: rowIndex + 1,
        columnIndex,
        loop,
        loopHabitantsFilter
      });
    } else if (!loop.neigboursValues.includes(rightNeighbour)) {
      loop.neigboursValues.push(rightNeighbour);
    }
  }

  if (columnIndex < matrixHeight - 1) {
    // down
    const bottomNeighbour = matrix[rowIndex][columnIndex + 1];
    if (loopHabitantsFilter({ value: bottomNeighbour })) {
      loop = calculateLoopContext({
        matrix,
        rowIndex,
        columnIndex: columnIndex + 1,
        loop,
        loopHabitantsFilter
      });
    } else if (!loop.neigboursValues.includes(bottomNeighbour)) {
      loop.neigboursValues.push(bottomNeighbour);
    }
  }

  return loop;
};
