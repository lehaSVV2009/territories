import { Game } from "boardgame.io/core";
import { CELL_TYPE } from "./gameUtils";

const isVictory = board => {
  return false;
};

const isDraw = board => {
  return false;
};

const Territories = ({ dices, board }) =>
  Game({
    setup: () => ({
      // Board 2x2 in format [[0, 0], [0, 0]]
      board: board || [...Array(10).fill([...Array(30).fill(CELL_TYPE.EMPTY)])],
      dices: dices || [0, 0]
    }),

    moves: {
      rollDices(G, ctx) {
        return { ...G, dices: ctx.random.Die(6, 2) };
      },

      switchDices(G) {
        return { ...G, dices: [G.dices[1], G.dices[0]] };
      },

      clearDices(G) {
        return { ...G, dices: [0, 0] };
      },

      dropSquare(
        G,
        ctx,
        rowIndex,
        columnIndex,
        rectangleHeight,
        rectangleWidth
      ) {
        const { currentPlayer } = ctx;
        return {
          ...G,
          board: G.board.map((row, rowStateIndex) =>
            rowStateIndex === rowIndex
              ? row.map((cell, columnStateIndex) =>
                  columnStateIndex === columnIndex
                    ? currentPlayer === "0"
                      ? CELL_TYPE.CAPTURED_BY_PLAYER_1
                      : CELL_TYPE.CAPTURED_BY_PLAYER_2
                    : cell
                )
              : row
          )
        };
      }
    },

    flow: {
      endGameIf: (G, ctx) => {
        if (isVictory(G.board)) {
          return { winner: ctx.currentPlayer };
        }
        if (isDraw(G.board)) {
          return { draw: true };
        }
      }
    }
  });

export default Territories;
