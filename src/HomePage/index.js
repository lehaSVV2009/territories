import React from "react";
import intl from "react-intl-universal";
import { Link } from "react-router-dom";

import { Container, Item } from "../libs/territories-ui/Grid";
import { DetachedFab, DetachedText } from "./elements";
import IconBot from "../libs/territories-icons/Bot";
import IconOnline from "../libs/territories-icons/Online";
import IconPeople from "../libs/territories-icons/People";

const HomePage = () => (
  <Container column center alignItems="center">
    <Item>
      <DetachedFab
        variant="extended"
        color="primary"
        component={Link}
        to="/online"
      >
        <IconOnline />
        <DetachedText>{intl.get("home.online")}</DetachedText>
      </DetachedFab>
    </Item>
    <Item>
      <DetachedFab
        variant="extended"
        color="secondary"
        component={Link}
        to="/ai"
      >
        <IconBot />
        <DetachedText>{intl.get("home.game_vs_ai")}</DetachedText>
      </DetachedFab>
    </Item>
    <Item>
      <DetachedFab variant="extended" component={Link} to="/two-players">
        <IconPeople />
        <DetachedText>{intl.get("home.two_players")}</DetachedText>
      </DetachedFab>
    </Item>
  </Container>
);

export default HomePage;
