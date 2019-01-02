import { Game } from "boardgame.io/core";
import { CELL_TYPE, PLAYER_1, PLAYER_2 } from "./gameUtils";

const selectVictoryContext = ({ allCellsCount, occupiedCounters }) => {
  const occupiedByPlayer1 = occupiedCounters[PLAYER_1];
  const occupiedByPlayer2 = occupiedCounters[PLAYER_2];
  return {
    allCellsCount,
    occupiedByPlayer1,
    occupiedByPlayer2
  };
};

// TODO think about nice algorithm, calculating locked borders
const selectWinner = ({
  allCellsCount,
  occupiedByPlayer1,
  occupiedByPlayer2
}) => {
  const halfCellsCount = allCellsCount / 2;
  return occupiedByPlayer1 > halfCellsCount
    ? PLAYER_1
    : occupiedByPlayer2 > halfCellsCount
    ? PLAYER_2
    : null;
};

const isDraw = ({ allCellsCount, occupiedByPlayer1, occupiedByPlayer2 }) => {
  return (
    occupiedByPlayer1 + occupiedByPlayer2 === allCellsCount &&
    occupiedByPlayer1 === occupiedByPlayer2
  );
};

const calculateCellsCount = ({ board }) => {
  return board.length * (board.length > 0 ? board[0].length : 0);
};

const DEFAULT_BOARD = [...Array(6).fill([...Array(6).fill(CELL_TYPE.EMPTY)])];
const DEFAULT_DICES = [0, 0];

const Territories = ({ dices, board }) =>
  Game({
    setup: () => {
      const boardOrDefault = board || DEFAULT_BOARD;

      return {
        // Board in format [["EMPTY", "EMPTY"], ["OCCUPIED_BY_PLAYER_1", "OCCUPIED_BY_PLAYER_2"]]
        board: boardOrDefault,
        dices: dices || DEFAULT_DICES,
        occupiedCounters: {
          [PLAYER_1]: 0,
          [PLAYER_2]: 0
        },
        allCellsCount: calculateCellsCount({ board: boardOrDefault })
      };
    },

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
            rowStateIndex >= rowIndex &&
            rowStateIndex < rowIndex + rectangleHeight
              ? row.map((cell, columnStateIndex) =>
                  columnStateIndex >= columnIndex &&
                  columnStateIndex < columnIndex + rectangleWidth
                    ? currentPlayer === PLAYER_1
                      ? CELL_TYPE.OCCUPIED_BY_PLAYER_1
                      : CELL_TYPE.OCCUPIED_BY_PLAYER_2
                    : cell
                )
              : row
          ),
          occupiedCounters: {
            ...G.occupiedCounters,
            [currentPlayer]:
              G.occupiedCounters[currentPlayer] +
              rectangleHeight * rectangleWidth
          }
        };
      }
    },

    flow: {
      endGameIf: G => {
        const victoryContext = selectVictoryContext(G);
        const winner = selectWinner(victoryContext);
        if (winner) {
          return { winner };
        }
        if (isDraw(victoryContext)) {
          return { draw: true };
        }
      }
    }
  });

export default Territories;
