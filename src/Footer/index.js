import React from "react";
import { Container, Item } from "../libs/territories-ui/Grid";

export default () => (
  <Container spaceBetween>
    <Item>
      <a
        href="https://pikabu.ru/story/prikolnaya_miniigra_stoit_poprobovat_6269129"
        target="_blank"
        rel="noopener noreferrer"
      >
        Rules
      </a>
    </Item>
    <Item>
      <a href="/?lang=en">en</a> <a href="/?lang=ru">ru</a>
    </Item>
  </Container>
);
