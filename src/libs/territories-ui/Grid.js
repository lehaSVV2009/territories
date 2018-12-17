import styled, { css } from "styled-components";

export const Container = styled.div`
  display: flex;
  justify-content: ${props =>
    props.left ? "flex-start" : props.right ? "flex-end" : "none"};
  flex-direction: ${props => (props.column ? "column" : "row")};

  ${props =>
    props.fullPage &&
    css`
      height: 100vh;
    `};
`;

export const Item = styled.div`
  flex: ${props => props.flex || "none"};
`;
