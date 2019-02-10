import Bot from "./Bot";
import { findPotentiallyOccupiedRectangles } from "../gameUtils";

export default class EasyBot extends Bot {
  constructor({ playerID, name }) {
    super({ playerID, name: name || "Easy Bot" });
  }

  guessCellToDropRectangle = ({
    currentPlayer,
    rectangleHeight,
    rectangleWidth,
    rows
  }) => {
    const result = {
      rowIndex: -1,
      columnIndex: -1,
      rectangleHeight,
      rectangleWidth
    };

    if (Math.random() >= 0.5) {
      result.rectangleHeight = rectangleWidth;
      result.rectangleWidth = rectangleHeight;
    }

    const rectangles = findPotentiallyOccupiedRectangles({
      currentPlayer,
      rectangleHeight: result.rectangleHeight,
      rectangleWidth: result.rectangleWidth,
      rows
    });

    if (rectangles.length === 0) {
      return result;
    }

    const randomCell =
      rectangles[Math.floor(Math.random() * rectangles.length)];
    result.rowIndex = randomCell.rowIndex;
    result.columnIndex = randomCell.columnIndex;
    return result;
  };
}
