import styled from "styled-components";
import { Container } from "../libs/territories-ui/Grid";
import Paper from "../libs/territories-ui/Paper";

export const FixedSizePaper = styled(Paper)`
  && {
    height: ${props => props.height};
    width: ${props => props.width};
  }
`;

export const FullHeightContainer = styled(Container)`
  height: 100%;
`;
