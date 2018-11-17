import { Game } from "boardgame.io/core";

// Return true if `cells` is in a winning configuration.
const isVictory = cells => {
  const winCombinations = [
    [0, 1, 2],
    [3, 4, 5],
    [6, 7, 8],
    [0, 3, 6],
    [1, 4, 7],
    [2, 5, 8],
    [0, 4, 8],
    [2, 4, 6]
  ];

  return winCombinations.some(winCombination => {
    const playerMark = cells[winCombination[0]];
    return (
      playerMark !== null &&
      (cells[winCombination[1]] === playerMark &&
        cells[winCombination[2]] === playerMark)
    );
  });
};

// Return true if all `cells` are occupied.
const isDraw = cells => {
  return cells.filter(c => c === null).length === 0;
};

const TicTacToe = Game({
  setup: () => ({ cells: Array(9).fill(null) }),

  moves: {
    clickCell(G, ctx, id) {
      const cells = [...G.cells];

      // Ensure we can't overwrite cells.
      if (cells[id] === null) {
        cells[id] = ctx.currentPlayer;
      }

      return { ...G, cells };
    }
  },

  flow: {
    movesPerTurn: 1,
    endGameIf: (G, ctx) => {
      if (isVictory(G.cells)) {
        return { winner: ctx.currentPlayer };
      }
      if (isDraw(G.cells)) {
        return { draw: true };
      }
    }
  }
});

export default TicTacToe;
