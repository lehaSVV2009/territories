import { Game } from "boardgame.io/core";

const isVictory = dices => {
  return dices && dices[0] === dices[1];
};

const isDraw = dices => {
  return dices && dices[0] === 6 && dices[1] === 1;
};

const CELL_TYPE = {
  EMPTY: "EMPTY",
  ASSINGED_BY_PLAYER_1: "ASSINGED_BY_PLAYER_1",
  ASSINGED_BY_PLAYER_2: "ASSINGED_BY_PLAYER_1"
};

const Territories = ({ dices, board }) =>
  Game({
    setup: () => ({
      // Board 2x2 in format [[0, 0], [0, 0]]
      board: board || Array(10).fill(Array(30).fill(CELL_TYPE.EMPTY)),
      dices: dices || [0, 0]
    }),

    moves: {
      rollDices(G, ctx) {
        return { ...G, dices: ctx.random.Die(6, 2) };
      },

      switchDices(G) {
        return { ...G, dices: [G.dices[1], G.dices[0]] };
      },

      dropSquare(G, ctx) {}
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
