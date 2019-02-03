import React from "react";
import intl from "react-intl-universal";
import { Link } from "react-router-dom";

import Fab from "../libs/territories-ui/Fab";
import { Container } from "../libs/territories-ui/Grid";
import { Face, Band, White, Red, Eyes, Dimples, Mouth, Text } from "./elements";

const NotFoundPage = () => (
  <div>
    <Face>
      <Band>
        <White />
        <Red />
      </Band>
      <Eyes />
      <Dimples />
      <Mouth />
    </Face>
    <Text>{intl.get("not_found.oops")}</Text>
    <Container center>
      <Fab variant="extended" component={Link} color="primary" to="/">
        {intl.get("not_found.go_home")}
      </Fab>
    </Container>
  </div>
);

export default NotFoundPage;
