import { Game } from "boardgame.io/core";

const isVictory = dices => {
  return dices && dices[0] === dices[1];
};

const isDraw = dices => {
  return dices && dices[0] === 6 && dices[1] === 1;
};

export const isEmptyCell = type => type === CELL_TYPE.EMPTY;
export const isCapturedCell = type =>
  type === CELL_TYPE.CAPTURED_BY_PLAYER_1 ||
  type === CELL_TYPE.CAPTURED_BY_PLAYER_2;

const CELL_TYPE = {
  EMPTY: "EMPTY",
  CAPTURED_BY_PLAYER_1: "CAPTURED_BY_PLAYER_1",
  CAPTURED_BY_PLAYER_2: "CAPTURED_BY_PLAYER_1"
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

      dropSquare(
        G,
        ctx,
        rowIndex,
        columnIndex,
        rectangleHeight,
        rectangleWidth
      ) {
        return {
          ...G,
          board: G.board.map((row, rowStateIndex) =>
            rowStateIndex === rowIndex
              ? row.map((cell, columnStateIndex) =>
                  columnStateIndex === columnIndex
                    ? CELL_TYPE.CAPTURED_BY_PLAYER_1
                    : cell
                )
              : row
          )
        };
      }
    },

    flow: {
      movesPerTurn: 1,
      endGameIf: (G, ctx) => {
        if (isVictory(G.dices)) {
          return { winner: ctx.currentPlayer };
        }
        if (isDraw(G.dices)) {
          return { draw: true };
        }
      }
    }
  });

export default Territories;
