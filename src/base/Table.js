import styled from "styled-components";

export default styled.div`
  display: table;
  border: 1px solid ${props => props.theme.colors.gray};
`;

export const Row = styled.div`
  display: table-row;
`;

export const Cell = styled.div`
  display: table-cell;
  padding: 10px;
  border: 1px solid ${props => props.theme.colors.gray};
`;
