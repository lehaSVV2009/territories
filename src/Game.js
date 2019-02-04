import { Game } from "boardgame.io/core";
import {
  CELL_TYPE,
  PLAYER_1,
  PLAYER_2,
  isEmptyCell,
  isOccupiedByPlayerOneCell,
  selectGameover
} from "./gameUtils";
import findClosedLoops from "./findClosedLoops";

const calculateCellsCount = ({ board }) => {
  return board.length * (board.length > 0 ? board[0].length : 0);
};

const DEFAULT_BOARD = [...Array(5).fill([...Array(5).fill(CELL_TYPE.EMPTY)])];
const DEFAULT_DICES = [0, 0];

const Territories = Game({
  name: "territories",

  setup: () => {
    return {
      // Board in format [["EMPTY", "EMPTY"], ["OCCUPIED_BY_PLAYER_1", "OCCUPIED_BY_PLAYER_2"]]
      board: DEFAULT_BOARD,
      dices: DEFAULT_DICES,
      rollingDices: null,
      occupiedCounters: {
        [PLAYER_1]: 0,
        [PLAYER_2]: 0
      },
      allCellsCount: calculateCellsCount({ board: DEFAULT_BOARD })
    };
  },

  moves: {
    startRollDices: (G, ctx) => {
      return { ...G, rollingDices: ctx.random.D6(2) };
    },

    finishRollDices: G => {
      return { ...G, dices: G.rollingDices, rollingDices: null };
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

      const newBoard = G.board.map((row, rowStateIndex) =>
        rowStateIndex >= rowIndex && rowStateIndex < rowIndex + rectangleHeight
          ? row.map((cell, columnStateIndex) =>
              columnStateIndex >= columnIndex &&
              columnStateIndex < columnIndex + rectangleWidth
                ? currentPlayer === PLAYER_1
                  ? CELL_TYPE.OCCUPIED_BY_PLAYER_1
                  : CELL_TYPE.OCCUPIED_BY_PLAYER_2
                : cell
            )
          : row
      );
      const newOccupiedCounters = {
        ...G.occupiedCounters,
        [currentPlayer]:
          G.occupiedCounters[currentPlayer] + rectangleHeight * rectangleWidth
      };

      // Calculate areas that are allowed for one player only
      let autoOccupiedByPlayer1 = [];
      let autoOccupiedByPlayer2 = [];

      // Do not try to find locked borders if only one player occupied cells (start of the game)
      if (
        newOccupiedCounters[PLAYER_1] !== 0 &&
        newOccupiedCounters[PLAYER_2] !== 0
      ) {
        const closedLoops = findClosedLoops({
          matrix: newBoard,
          loopHabitantsFilter: ({ value }) => isEmptyCell(value)
        });
        closedLoops.forEach(closedLoop => {
          // Skip loops that have both players as neighbours
          if (closedLoop.neigboursValues.length !== 1) {
            return;
          }
          if (isOccupiedByPlayerOneCell(closedLoop.neigboursValues[0])) {
            autoOccupiedByPlayer1 = autoOccupiedByPlayer1.concat(
              closedLoop.cells
            );
          } else {
            autoOccupiedByPlayer2 = autoOccupiedByPlayer2.concat(
              closedLoop.cells
            );
          }
        });
      }

      return {
        ...G,
        board: newBoard.map((row, rowIndex) =>
          row.map((cell, columnIndex) => {
            if (!isEmptyCell(cell)) {
              return cell;
            }
            if (
              autoOccupiedByPlayer1.some(
                autoOccupiedCell =>
                  rowIndex === autoOccupiedCell.rowIndex &&
                  columnIndex === autoOccupiedCell.columnIndex
              )
            ) {
              return CELL_TYPE.OCCUPIED_BY_PLAYER_1;
            }
            if (
              autoOccupiedByPlayer2.some(
                autoOccupiedCell =>
                  rowIndex === autoOccupiedCell.rowIndex &&
                  columnIndex === autoOccupiedCell.columnIndex
              )
            ) {
              return CELL_TYPE.OCCUPIED_BY_PLAYER_2;
            }
            return CELL_TYPE.EMPTY;
          })
        ),
        occupiedCounters: {
          [PLAYER_1]:
            newOccupiedCounters[PLAYER_1] + autoOccupiedByPlayer1.length,
          [PLAYER_2]:
            newOccupiedCounters[PLAYER_2] + autoOccupiedByPlayer2.length
        }
      };
    }
  },

  flow: {
    endGameIf: selectGameover
  }
});

Territories.minPlayers = 2;
Territories.maxPlayers = 2;

export default Territories;
