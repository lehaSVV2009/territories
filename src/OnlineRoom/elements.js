import styled from "styled-components";
import Button from "../libs/territories-ui/Button";
import { Container, Item } from "../libs/territories-ui/Grid";

export const DetachedButton = styled(Button)`
  && {
    margin-left: 4px;
  }
`;

export const DetachedContainer = styled(Container)`
  margin-top: 4px;
`;

export const AlignCenterItem = styled(Item)`
  text-align: center;
`;

export const AlignLeftItem = styled(Item)`
  text-align: left;
`;

export const AlignRightItem = styled(Item)`
  text-align: right;
`;
