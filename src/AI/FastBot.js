import Bot from "./Bot";
import { findPotentiallyOccupiedRectangles } from "territories-core";

export default class FastBot extends Bot {
  constructor({ playerID, name } = {}) {
    super({ playerID, name: name || "FastBot" });
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

    const rects = findPotentiallyOccupiedRectangles({
      currentPlayer,
      rectangleHeight,
      rectangleWidth,
      rows
    });

    if (rects && rects.length > 0) {
      // Pick first legal move without heavy search
      const chosen = rects[0];
      result.rowIndex = chosen.rowIndex;
      result.columnIndex = chosen.columnIndex;
      result.rectangleHeight = chosen.rectangleHeight;
      result.rectangleWidth = chosen.rectangleWidth;
      return result;
    }

    if (rectangleHeight !== rectangleWidth) {
      const rotated = findPotentiallyOccupiedRectangles({
        currentPlayer,
        rectangleHeight: rectangleWidth,
        rectangleWidth: rectangleHeight,
        rows
      });
      if (rotated && rotated.length > 0) {
        const chosen = rotated[0];
        result.rowIndex = chosen.rowIndex;
        result.columnIndex = chosen.columnIndex;
        result.rectangleHeight = chosen.rectangleHeight;
        result.rectangleWidth = chosen.rectangleWidth;
        return result;
      }
    }

    return result;
  };
}
