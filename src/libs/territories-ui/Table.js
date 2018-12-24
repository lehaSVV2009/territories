import styled from "styled-components";

const DEFAULT_BORDER = 1;
const DEFAULT_CELL_RADIUS = 10;

export default styled.div`
  display: table;
  border: ${DEFAULT_BORDER}px solid ${props => props.theme.colors.gray};
`;

export const Row = styled.div`
  display: table-row;
`;

export const Cell = styled.div`
  display: table-cell;
  padding: ${props =>
    props.cellRadius
      ? props.cellRadius - DEFAULT_BORDER
      : DEFAULT_CELL_RADIUS}px;
  border: ${DEFAULT_BORDER}px solid ${props => props.theme.colors.gray};
`;
