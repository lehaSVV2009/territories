import styled from "styled-components";
import Button from "../libs/territories-ui/Button";
import Select from "../libs/territories-ui/Select";

export const Logo = styled.span`
  flex-grow: 1;
  text-transform: uppercase;
  :hover {
    cursor: pointer;
  }
`;

export const WhiteSelect = styled(Select)`
  && {
    color: ${props => props.theme.colors.white};
  }
`;

export const DetachedButton = styled(Button)`
  && {
    margin-left: 8px;
    margin-right: 8px;
  }
`;

export const FixedAppBarMargin = styled.div`
  /* Top margin from fixed app bar */
  margin-top: 80px;
`;
