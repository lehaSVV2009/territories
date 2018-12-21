import { Game } from "boardgame.io/core";
import { CELL_TYPE, PLAYER_1, PLAYER_2 } from "./gameUtils";

// TODO reimplement and optimize when game is implemented by rules
// TODO probably store capturing counters in G state
const selectVictoryContext = board => {
  let empty = 0;
  let occupiedByPlayer1 = 0;
  let occupiedByPlayer2 = 0;
  board.forEach(row =>
    row.forEach(cell => {
      if (cell === CELL_TYPE.EMPTY) {
        empty++;
      } else if (cell === CELL_TYPE.OCCUPIED_BY_PLAYER_1) {
        occupiedByPlayer1++;
      } else if (cell === CELL_TYPE.OCCUPIED_BY_PLAYER_2) {
        occupiedByPlayer2++;
      }
    })
  );
  return {
    empty,
    occupiedByPlayer1,
    occupiedByPlayer2
  };
};

const isVictory = ({ empty, occupiedByPlayer1, occupiedByPlayer2 }) => {
  return empty === 0 && occupiedByPlayer1 !== occupiedByPlayer2;
};

const isDraw = ({ empty, occupiedByPlayer1, occupiedByPlayer2 }) => {
  return empty === 0 && occupiedByPlayer1 === occupiedByPlayer2;
};

const Territories = ({ dices, board }) =>
  Game({
    setup: () => ({
      // Board in format [["EMPTY", "EMPTY"], ["OCCUPIED_BY_PLAYER_1", "OCCUPIED_BY_PLAYER_2"]]
      board: board || [...Array(20).fill([...Array(10).fill(CELL_TYPE.EMPTY)])],
      dices: dices || [0, 0]
    }),

    moves: {
      changeDices: (G, ctx, dices) => {
        return { ...G, dices };
      },

      switchDices: G => {
        return { ...G, dices: [G.dices[1], G.dices[0]] };
      },

      clearDices: G => {
        return { ...G, dices: [0, 0] };
      },

      dropRectangle: (
        G,
        ctx,
        rowIndex,
        columnIndex,
        rectangleHeight,
        rectangleWidth
      ) => {
        const { currentPlayer } = ctx;
        return {
          ...G,
          board: G.board.map((row, rowStateIndex) =>
            rowStateIndex === rowIndex
              ? row.map((cell, columnStateIndex) =>
                  columnStateIndex === columnIndex
                    ? currentPlayer === PLAYER_1
                      ? CELL_TYPE.OCCUPIED_BY_PLAYER_1
                      : CELL_TYPE.OCCUPIED_BY_PLAYER_2
                    : cell
                )
              : row
          )
        };
      }
    },

    flow: {
      endGameIf: (G, ctx) => {
        const victoryContext = selectVictoryContext(G.board);
        if (isVictory(victoryContext)) {
          const { occupiedByPlayer1, occupiedByPlayer2 } = victoryContext;
          return {
            winner: occupiedByPlayer1 > occupiedByPlayer2 ? PLAYER_1 : PLAYER_2
          };
        }
        if (isDraw(victoryContext)) {
          return { draw: true };
        }
      }
    }
  });

export default Territories;
