import React from "react";
import styled from "styled-components";

import { Container, Item } from "../base/Grid";
import { isPlayer1 } from "../gameUtils";

const PlayerIcon = styled.div`
  border: 1px solid ${props => props.theme.colors.gray};
  padding: 8px;
  background-color: ${props =>
    isPlayer1(props.player)
      ? props.theme.colors.player1
      : props.theme.colors.player2};
`;

const Player = ({ player }) => (
  <Container>
    <Item>
      <PlayerIcon player={player} />
    </Item>
    <Item>Player: {player}</Item>
  </Container>
);

export default Player;
