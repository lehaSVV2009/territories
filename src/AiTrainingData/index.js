import { findPotentiallyOccupiedRectangles } from "territories-core";
const LOCAL_STORAGE_KEY = "territories-ai-data";

export const readAiTrainingItems = () =>
  localStorage.getItem(LOCAL_STORAGE_KEY)
    ? JSON.parse(localStorage.getItem(LOCAL_STORAGE_KEY))
    : [];

export const addAiTrainingItems = items =>
  localStorage.setItem(
    LOCAL_STORAGE_KEY,
    JSON.stringify(readAiTrainingItems().concat(items))
  );

export const collect = ({
  columnIndex,
  rowIndex,
  rectangleHeight,
  rectangleWidth,
  turn,
  currentPlayer,
  rows,
  emptyCellsCount
}) => {
  const chosenRectangle = {
    columnIndex,
    rowIndex,
    rectangleHeight,
    rectangleWidth
  };
  const potentiallyOccupiedRectangles = findPotentiallyOccupiedRectangles({
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

  const boardHeight = rows.length;
  const boardWidth = rows[0].length;
  const trainingItems = potentiallyOccupiedRectangles.map(rectangle =>
    buildTrainingItem({
      chosenRectangle,
      turn,
      rectangle,
      boardHeight,
      boardWidth,
      emptyCellsCount
    })
  );

  addAiTrainingItems(trainingItems);
  return trainingItems;
};

const buildTrainingItem = ({
  chosenRectangle,
  turn,
  rectangle,
  boardHeight,
  boardWidth,
  emptyCellsCount
}) => {
  return {
    input: {
      turn: turn / 10000,
      columnIndex: rectangle.columnIndex / boardWidth,
      rowIndex: rectangle.rowIndex / boardHeight,
      rectangleHeight: rectangle.rectangleHeight / 6,
      rectangleWidth: rectangle.rectangleWidth / 6,
      emptyCellsCount: emptyCellsCount / (boardHeight * boardWidth)
    },
    output: {
      best:
        rectangle.columnIndex === chosenRectangle.columnIndex &&
        rectangle.rowIndex === chosenRectangle.rowIndex &&
        rectangle.rectangleHeight === chosenRectangle.rectangleHeight &&
        rectangle.rectangleWidth === chosenRectangle.rectangleWidth
    }
  };
};
