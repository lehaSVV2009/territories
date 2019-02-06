export default class Bot {
  constructor({ playerID, name }) {
    this.playerID = playerID;
    this.name = name;
  }

  findBestPlaceForRectangle = ({
    currentPlayer,
    rectangleHeight,
    rectangleWidth,
    rows
  }) => ({
    rowIndex: -1,
    cellIndex: -1,
    rectangleHeight,
    rectangleWidth,
    currentPlayer,
    rows
  });
}
