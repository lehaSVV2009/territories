import React from "react";
import { HashRouter as Router, Route, Switch } from "react-router-dom";

import VersusBotPage from "./VersusBotPage";
import HomePage from "./HomePage";
import NotFoundPage from "./NotFoundPage";
import OnlineLobbyPage from "./OnlineLobbyPage";
import TwoPlayersPage from "./TwoPlayersPage";

// TODO add react-loadable
const Routes = () => (
  // Hash router is used because of github-pages issues with browser router
  <Router>
    <Switch>
      <Route exact path="/" component={HomePage} />
      <Route path="/versus-bot" component={VersusBotPage} />
      <Route path="/two-players" component={TwoPlayersPage} />
      <Route path="/online" component={OnlineLobbyPage} />
      <Route path="*" component={NotFoundPage} />
    </Switch>
  </Router>
);

export default Routes;
