import React from "react";
import { HashRouter as Router, Route, Switch } from "react-router-dom";

import ErrorPage from "./Error";
import Loadable from "react-loadable";
import LinearProgress from "./libs/territories-ui/LinearProgress";

const Loading = props =>
  props.error ? <ErrorPage /> : <LinearProgress color="secondary" />;

const LoadableHomePage = Loadable({
  loader: () => import("./HomePage"),
  loading: Loading
});

const LoadableAiPage = Loadable({
  loader: () => import("./AiPage"),
  loading: Loading
});

const LoadableTwoPlayersPage = Loadable({
  loader: () => import("./TwoPlayersPage"),
  loading: Loading
});

const LoadableOnlinePage = Loadable({
  loader: () => import("./OnlineLobbyPage"),
  loading: Loading
});

// TODO add react-loadable
const Routes = () => (
  // Hash router is used because of github-pages issues with browser router
  <Router>
    <Switch>
      <Route exact path="/" component={LoadableHomePage} />
      <Route path="/ai" component={LoadableAiPage} />
      <Route path="/two-players" component={LoadableTwoPlayersPage} />
      <Route path="/online" component={LoadableOnlinePage} />
      <Route path="*" component={ErrorPage} />
    </Switch>
  </Router>
);

export default Routes;
