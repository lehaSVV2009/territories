import Bot from "./Bot";
import { findPotentiallyOccupiedCells } from "../gameUtils";

export default class EasyBot extends Bot {
  constructor({ playerID, name }) {
    super({ playerID, name: name || "Easy Bot" });
  }

  findBestPlaceForRectangle = ({
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

    const cells = findPotentiallyOccupiedCells({
      currentPlayer,
      rectangleHeight: result.rectangleHeight,
      rectangleWidth: result.rectangleWidth,
      rows
    });

    if (cells.length === 0) {
      return result;
    }

    const randomCell = cells[Math.floor(Math.random() * cells.length)];
    result.rowIndex = randomCell.rowIndex;
    result.columnIndex = randomCell.columnIndex;
    return result;
  };
}
