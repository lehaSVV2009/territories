import Bot from "./Bot";
import { findPotentiallyOccupiedRectangles } from "territories-core";

export default class GreedyBot extends Bot {
  constructor({ playerID, name } = {}) {
    super({ playerID, name: name || "GreedyBot" });
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

    const rectangles = findPotentiallyOccupiedRectangles({
      currentPlayer,
      rectangleHeight,
      rectangleWidth,
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

    if (!rectangles || rectangles.length === 0) {
      return result;
    }

    let bestRect = rectangles[0];
    let bestScore = -Infinity;

    for (let i = 0; i < rectangles.length; i++) {
      const rect = rectangles[i];
      const area = rect.rectangleHeight * rect.rectangleWidth;

      // Prefer larger area, then advancing towards enemy corner
      const advance =
        currentPlayer === 0
          ? rect.rowIndex + rect.columnIndex
          : (14 - rect.rowIndex) + (39 - rect.columnIndex);

      const score = area * 1000 + advance;
      if (score > bestScore) {
        bestScore = score;
        bestRect = rect;
      }
    }

    result.rowIndex = bestRect.rowIndex;
    result.columnIndex = bestRect.columnIndex;
    result.rectangleHeight = bestRect.rectangleHeight;
    result.rectangleWidth = bestRect.rectangleWidth;
    return result;
  };
}
