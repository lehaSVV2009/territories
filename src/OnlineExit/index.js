import React from "react";
import intl from "react-intl-universal";

import { Container, Item } from "../libs/territories-ui/Grid";
import IconButton from "../libs/territories-ui/IconButton";
import IconLogout from "../libs/territories-icons/Logout";
import Tooltip from "../libs/territories-ui/Tooltip";
import { Layout } from "./elements";

const OnlineExit = ({ exitButtonLabel, playerName, onExit }) => (
  <Layout>
    <Container alignItems="center" spaceBetween>
      <Item>
        <h2>{intl.get("online.hello", { name: playerName })}</h2>
      </Item>
      <Item>
        <Tooltip title={exitButtonLabel}>
          <IconButton onClick={onExit}>
            <IconLogout />
          </IconButton>
        </Tooltip>
      </Item>
    </Container>
  </Layout>
);

export default OnlineExit;
