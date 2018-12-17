export const CELL_TYPE = {
  EMPTY: "EMPTY",
  CAPTURED_BY_PLAYER_1: "CAPTURED_BY_PLAYER_1",
  CAPTURED_BY_PLAYER_2: "CAPTURED_BY_PLAYER_2"
};

export const isEmptyCell = type => {
  return type === CELL_TYPE.EMPTY;
};
export const isCapturedByPlayerOneCell = type => {
  return type === CELL_TYPE.CAPTURED_BY_PLAYER_1;
};
export const isCapturedByPlayerTwoCell = type => {
  return type === CELL_TYPE.CAPTURED_BY_PLAYER_2;
};

export const PLAYER_1 = "0";
export const PLAYER_2 = "1";

export const isPlayer1 = player => {
  return player === PLAYER_1;
};
export const isPlayer2 = player => {
  return player === PLAYER_2;
};
