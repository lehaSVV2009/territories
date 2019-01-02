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

export const Overlay = styled.div`
  position: absolute;
  top: 0;
  left: 0;
  height: 100%;
  width: 100%;
  z-index: 1;
  opacity: 0.5;
`;

export const GreenOverlay = styled(Overlay)`
  background-color: ${props => props.theme.colors.green};
`;

export const RedOverlay = styled(Overlay)`
  background-color: ${props => props.theme.colors.red};
`;

export const YellowOverlay = styled(Overlay)`
  background-color: ${props => props.theme.colors.yellow};
`;
