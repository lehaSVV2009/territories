import React from "react";
import { Container, Item } from "../libs/territories-ui/Grid";

// TODO add link to github in the end
export default () => (
  <Container spaceBetween>
    <Item>Territories</Item>
    <Item>
      <a
        href="https://github.com/lehaSVV2009/territories"
        target="_blank"
        rel="noopener noreferrer"
      >
        Github
      </a>
    </Item>
  </Container>
);
