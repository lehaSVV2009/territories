import { Game } from "boardgame.io/core";
import {
  CELL_TYPE,
  PLAYER_1,
  PLAYER_2,
  isEmptyCell,
  isOccupiedByPlayerOneCell
} from "./gameUtils";
import findClosedLoops from "./libs/find-closed-loops";

const selectVictoryContext = ({ allCellsCount, occupiedCounters }) => {
  const occupiedByPlayer1 = occupiedCounters[PLAYER_1];
  const occupiedByPlayer2 = occupiedCounters[PLAYER_2];
  return {
    allCellsCount,
    occupiedByPlayer1,
    occupiedByPlayer2
  };
};

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

const DEFAULT_BOARD = [...Array(15).fill([...Array(40).fill(CELL_TYPE.EMPTY)])];
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

        const newBoard = G.board.map((row, rowStateIndex) =>
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
