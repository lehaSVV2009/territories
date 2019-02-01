import React, { Fragment } from "react";
import { createMuiTheme } from "@material-ui/core/styles";
import { MuiThemeProvider } from "@material-ui/core";
import { ThemeProvider } from "styled-components";

import AppWraper from "./AppWrapper";
import initLocale from "./initLocale";
import LinearProgress from "./libs/territories-ui/LinearProgress";
import Routes from "./Routes";

// TODO replace styled-components theme with mui theme
const theme = {
  colors: {
    white: "#FFFFFF",
    yellow: "#FFFF00",
    gray: "#DDDDDD",
    green: "#00FF00",
    red: "#FF0000",
    player1: "#375E97",
    player2: "#FB6542"
  }
};

const muiTheme = createMuiTheme({
  palette: {
    primary: {
      main: theme.colors.player1,
      contrastText: theme.colors.white
    },
    secondary: {
      main: theme.colors.player2,
      contrastText: theme.colors.white
    }
  }
});

class App extends React.Component {
  state = {
    loadingLocales: true
  };

  componentDidMount() {
    initLocale().then(() => this.setState({ loadingLocales: false }));
  }

  render() {
    return (
      <MuiThemeProvider theme={muiTheme}>
        <ThemeProvider theme={theme}>
          <Fragment>
            {this.state.loadingLocales ? (
              <LinearProgress color="secondary" />
            ) : (
              <AppWraper>
                <Routes />
               </AppWraper>
            )}
          </Fragment>
        </ThemeProvider>
      </MuiThemeProvider>
    );
  }
}

export default App;
