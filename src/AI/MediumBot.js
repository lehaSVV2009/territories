import Bot from "./Bot";
import { findPotentiallyOccupiedRectangles } from "territories-core";
import runMediumNeuralNetwork from "./runMediumNeuralNetwork";

const normalizeInput = ({
  turn,
  rectangle,
  boardHeight,
  boardWidth,
  emptyCellsCount
}) => ({
  turn: turn / 10000,
  columnIndex: rectangle.columnIndex / boardWidth,
  rowIndex: rectangle.rowIndex / boardHeight,
  rectangleHeight: rectangle.rectangleHeight / 6,
  rectangleWidth: rectangle.rectangleWidth / 6,
  emptyCellsCount: emptyCellsCount / (boardHeight * boardWidth)
});

const findRectangleRate = ({
  turn,
  rectangle,
  boardHeight,
  boardWidth,
  emptyCellsCount
}) => {
  return runMediumNeuralNetwork(
    normalizeInput({
      turn,
      rectangle,
      boardHeight,
      boardWidth,
      emptyCellsCount
    })
  ).best;
};

export default class MediumBot extends Bot {
  constructor({ playerID, name }) {
    super({ playerID, name: name || "Medium Bot" });
  }

  guessCellToDropRectangle = ({
    turn,
    currentPlayer,
    rectangleHeight,
    rectangleWidth,
    rows,
    emptyCellsCount
  }) => {
    const result = {
      rowIndex: -1,
      columnIndex: -1,
      rectangleHeight,
      rectangleWidth
    };

    const rectangles = findPotentiallyOccupiedRectangles({
      currentPlayer,
      rectangleHeight: result.rectangleHeight,
      rectangleWidth: result.rectangleWidth,
      rows
    }).concat(
      rectangleHeight === rectangleWidth
        ? []
        : findPotentiallyOccupiedRectangles({
            currentPlayer,
            rectangleHeight: rectangleWidth,
            rectangleWidth: rectangleHeight,
            rows
          })
    );

    if (rectangles.length === 0) {
      return result;
    }

    const boardHeight = rows.length;
    const boardWidth = rows[0].length;

    let maxRate = 0;
    rectangles.forEach(rectangle => {
      const rate = findRectangleRate({
        turn,
        rectangle,
        boardHeight,
        boardWidth,
        emptyCellsCount
      });

      if (rate > maxRate) {
        maxRate = rate;
        result.rowIndex = rectangle.rowIndex;
        result.columnIndex = rectangle.columnIndex;
        result.rectangleHeight = rectangle.rectangleHeight;
        result.rectangleWidth = rectangle.rectangleWidth;
      }
    });

    return result;
  };
}
