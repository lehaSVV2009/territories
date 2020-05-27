import Bot from "./Bot";
import {
  isEmptyCell,
  inRectangle,
  findPotentiallyOccupiedRectangles,
} from "territories-core";

export default class HardBot extends Bot {
  static RECTANGLE_EDGE = {
    TOP_LEFT: "TOP_LEFT",
    TOP_RIGHT: "TOP_RIGHT",
    BOTTOM_LEFT: "BOTTOM_LEFT",
    BOTTOM_RIGHT: "BOTTOM_RIGHT",
    CENTER_RIGHT: "CENTER_RIGHT",
  };

  constructor({ playerID, name }) {
    super({ playerID, name: name || "Bot" });
  }

  guessCellToDropRectangle = ({
    currentPlayer,
    rectangleHeight,
    rectangleWidth,
    rows,
  }) => {
    const result = {
      rowIndex: -1,
      columnIndex: -1,
      rectangleHeight,
      rectangleWidth,
    };

    const rectangles = findPotentiallyOccupiedRectangles({
      currentPlayer,
      rectangleHeight,
      rectangleWidth,
      rows,
    }).concat(
      rectangleHeight === rectangleWidth
        ? []
        : findPotentiallyOccupiedRectangles({
            currentPlayer,
            rectangleHeight: rectangleWidth,
            rectangleWidth: rectangleHeight,
            rows,
          })
    );

    if (rectangles.length === 0) {
      return result;
    }

    const boardWidth = rows.length;
    const boardHeight = rows[0].length;
    const boardCenterRowIndex = Math.floor(boardWidth / 2);
    const boardCenterColumnIndex = Math.floor(boardHeight / 2);

    let chosenRectangle = rectangles[0];

    if (isEmptyCell(rows[boardCenterRowIndex][boardCenterColumnIndex])) {
      // TODO Go to the top left until 7 cells horizontal distance from the rightmost enemy cell?

      // center is not taken
      // if any rectangle touches board center - find one that is closest to rectangle right center
      // else find one that makes left top closest to board center
      const rectanglesInCenter = rectangles.filter((rectangle) =>
        inRectangle({
          rowIndex: boardCenterRowIndex,
          columnIndex: boardCenterColumnIndex,
          selectedRowIndex: rectangle.rowIndex,
          selectedColumnIndex: rectangle.columnIndex,
          rectangleHeight: rectangle.rectangleHeight,
          rectangleWidth: rectangle.rectangleWidth,
        })
      );
      if (rectanglesInCenter.length > 0) {
        chosenRectangle = this.findClosestRectangleToCell(
          rectanglesInCenter,
          boardCenterRowIndex,
          boardCenterColumnIndex,
          HardBot.RECTANGLE_EDGE.CENTER_RIGHT
        );
      } else {
        chosenRectangle = this.findClosestRectangleToCell(
          rectangles,
          boardCenterRowIndex,
          boardCenterColumnIndex,
          HardBot.RECTANGLE_EDGE.TOP_LEFT
        );
      }
    } else if (isEmptyCell(rows[0][boardCenterColumnIndex])) {
      // top center is not taken
      chosenRectangle = this.findClosestRectangleToCell(
        rectangles,
        0,
        boardCenterColumnIndex - Math.floor(Math.random() * 4),
        HardBot.RECTANGLE_EDGE.TOP_RIGHT
      );
    } else if (isEmptyCell(rows[rows.length - 1][boardCenterColumnIndex])) {
      // bottom center is not taken
      chosenRectangle = this.findClosestRectangleToCell(
        rectangles,
        rows.length - 1,
        boardCenterColumnIndex - Math.floor(Math.random() * 4),
        HardBot.RECTANGLE_EDGE.BOTTOM_RIGHT
      );
    }

    // else if center is taken
    // randomly go to the top or bottom top point
    // if not taken

    // else if center is taken
    // if enemy direction is top - find rightest row and go to the highest point with lefttop rectangle (block his way) until 2 cells before border
    // if enemy direction is bottom - find rightest row and go to the lowest point with leftbottom rectangle (block his way) until 2 cells before border
    // if enemy direction is center???

    // or random for attack-block?

    // else ?? Try to block the way
    // Or choose leftmost from all cells?
    // Or find the way to try all cases and see which one gives more income

    result.rowIndex = chosenRectangle.rowIndex;
    result.columnIndex = chosenRectangle.columnIndex;
    result.rectangleHeight = chosenRectangle.rectangleHeight;
    result.rectangleWidth = chosenRectangle.rectangleWidth;
    return result;
  };

  calculateDistance = ({
    fromRowIndex,
    fromColumnIndex,
    toRowIndex,
    toColumnIndex,
  }) => {
    return Math.sqrt(
      (fromRowIndex - toRowIndex) ** 2 + (fromColumnIndex - toColumnIndex) ** 2
    );
  };

  findClosestRectangleToCell = (
    rectangles,
    rowIndex,
    columnIndex,
    rectangleEdge = HardBot.RECTANGLE_EDGE.TOP_LEFT // TODO fill with (TOP_LEFT, TOP_CENTER, TOP_RIGHT, CENTER_LEFT, CENTER, CENTER_RIGHT, BOTTOM_LEFT, BOTTOM_CENTER, BOTTOM_RIGHT)
  ) => {
    let chosenRectangle = rectangles[0];

    let shift = this.calculateRectangleShift(rectangleEdge, chosenRectangle);
    let minDistanceToCenter = this.calculateDistance({
      fromRowIndex: chosenRectangle.rowIndex + shift.rowShift,
      fromColumnIndex: chosenRectangle.columnIndex + shift.columnShift,
      toRowIndex: rowIndex,
      toColumnIndex: columnIndex,
    });
    for (const rectangle of rectangles) {
      shift = this.calculateRectangleShift(rectangleEdge, rectangle);

      const distanceToCenter = this.calculateDistance({
        fromRowIndex: rectangle.rowIndex + shift.rowShift,
        fromColumnIndex: rectangle.columnIndex + shift.columnShift,
        toRowIndex: rowIndex,
        toColumnIndex: columnIndex,
      });
      if (distanceToCenter < minDistanceToCenter) {
        minDistanceToCenter = distanceToCenter;
        chosenRectangle = rectangle;
      }
    }
    return chosenRectangle;
  };

  calculateRectangleShift = (rectangleEdge, rectangle) => {
    const shift = {
      rowShift: 0,
      columnShift: 0,
    };
    switch (rectangleEdge) {
      case HardBot.RECTANGLE_EDGE.CENTER_RIGHT: {
        shift.rowShift = Math.floor(rectangle.rectangleHeight / 2);
        shift.columnShift = Math.floor(rectangle.rectangleWidth);
        break;
      }
      case HardBot.RECTANGLE_EDGE.BOTTOM_LEFT: {
        shift.rowShift = rectangle.rectangleHeight;
        break;
      }
      case HardBot.RECTANGLE_EDGE.BOTTOM_RIGHT: {
        shift.rowShift = rectangle.rectangleHeight;
        shift.columnShift = rectangle.rectangleWidth;
        break;
      }
      case HardBot.RECTANGLE_EDGE.TOP_RIGHT: {
        shift.columnShift = rectangle.rectangleWidth;
        break;
      }
      case HardBot.RECTANGLE_EDGE.TOP_LEFT:
      default: {
        break;
      }
    }
    return shift;
  };
}
