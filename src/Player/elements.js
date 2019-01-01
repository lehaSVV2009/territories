import styled from "styled-components";

import Avatar from "../libs/territories-ui/Avatar";
import Card from "../libs/territories-ui/Card";
import CardContent from "../libs/territories-ui/CardContent";
import { isPlayer1 } from "../gameUtils";

export const DetachedCard = styled(Card)`
  && {
    margin-right: 8px;
    margin-left: 8px;
  }
`;

export const StyledAvatar = styled(Avatar)`
  && {
    background-color: ${props =>
      isPlayer1(props.player)
        ? props.theme.colors.player1
        : props.theme.colors.player2};
  }
`;

export const CenteredCardContent = styled(CardContent)`
  && {
    text-align: center;
  }
`;
