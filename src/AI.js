import { AI } from "boardgame.io/ai";
import { findPotentiallyOccupiedCells } from "./gameUtils";

const TerritoriesAI = AI({
  enumerate: (G, ctx, playerID) => {
    const botDices = [
      Math.floor(Math.random() * 6) + 1,
      Math.floor(Math.random() * 6) + 1
    ];
    return findPotentiallyOccupiedCells({
      currentPlayer: playerID,
      rectangleHeight: botDices[0],
      rectangleWidth: botDices[1],
      rows: G.board
    }).map(({ rowIndex, columnIndex }) => ({
      move: "dropRectangle",
      args: [rowIndex, columnIndex, botDices[0], botDices[1]]
    }));
  }
});

export default TerritoriesAI;
