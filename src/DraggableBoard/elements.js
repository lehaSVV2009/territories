import styled, { css } from "styled-components";
import Board from "../base/Board";

export const StyledBoard = styled(Board)`
  ${props =>
    props.isDragging
      ? css`
          background-color: ${props.theme.colors.green};
          opacity: 0.5;
        `
      : css`
          cursor: move;
          cursor: grab;
        `}:
`;
