import styled, { css } from "styled-components";

export const Container = styled.div`
  display: flex;
  justify-content: ${props =>
    props.left
      ? "flex-start"
      : props.right
      ? "flex-end"
      : props.center
      ? "center"
      : props.spaceBetween
      ? "space-between"
      : props.spaceAround
      ? "space-around"
      : "none"};
  flex-direction: ${props => (props.column ? "column" : "row")};
`;

export const Item = styled.div`
  flex: ${props => props.flex || "none"};
  ${props =>
    props.centered &&
    css`
      margin: 0 auto;
    `};
`;
