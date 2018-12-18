import styled from "styled-components";
import { Cell } from "../libs/territories-ui/Table";

export const TYPE_PLAYER_1 = "PLAYER_1";
export const TYPE_PLAYER_2 = "PLAYER_2";
export const TYPE_EMPTY = "EMPTY";

export const OccupiedCell = styled(Cell)`
  position: relative;
  background-color: ${props =>
    props.type === TYPE_PLAYER_1
      ? props.theme.colors.player1
      : props.type === TYPE_PLAYER_2
      ? props.theme.colors.player2
      : "inherit"};
`;
