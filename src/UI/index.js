import React, { Component } from "react";
import { createMuiTheme } from "@material-ui/core/styles";
import { MuiThemeProvider } from "@material-ui/core";
import { ThemeProvider } from "styled-components";

import Board from "../Board";
import Congratulations from "../Congratulations";
import { Container, Item } from "../libs/territories-ui/Grid";
import Header from "../Header";
import PlayersControls from "../PlayersControls";
import { DetachedItem } from "./elements";

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

const CELL_RADIUS = 10;

// TODO Add internationalization
class UI extends Component {
  handleRollDices = dices => {
    this.props.moves.changeDices(dices);
  };

  handleRotateRectangle = () => {
    this.props.moves.switchDices();
  };

  handleDropRectangle = ({
    rowIndex,
    columnIndex,
    rectangleHeight,
    rectangleWidth
  }) => {
    this.props.moves.dropRectangle(
      rowIndex,
      columnIndex,
      rectangleHeight,
      rectangleWidth
    );
    this.handleEndTurn();
  };

  handleEndTurn = () => {
    this.props.moves.clearDices();
    this.props.events.endTurn();
  };

  render() {
    const {
      G: { board, dices, allCellsCount, occupiedCounters },
      ctx: { currentPlayer, gameover }
    } = this.props;

    return (
      <MuiThemeProvider theme={muiTheme}>
        <ThemeProvider theme={theme}>
          <div>
            <Header />
            <Container column>
              <DetachedItem center>
                <PlayersControls
                  cellRadius={CELL_RADIUS}
                  dices={dices}
                  currentPlayer={currentPlayer}
                  allCellsCount={allCellsCount}
                  occupiedCounters={occupiedCounters}
                  onRollDices={this.handleRollDices}
                  onRotateRectangle={this.handleRotateRectangle}
                  onSkipTurn={this.handleEndTurn}
                />
              </DetachedItem>
              <Item center>
                <Board
                  cellRadius={CELL_RADIUS}
                  rows={board}
                  rectangleHeight={dices[0]}
                  rectangleWidth={dices[1]}
                  currentPlayer={currentPlayer}
                  onDropRectangle={this.handleDropRectangle}
                />
              </Item>
              {/* Win or Draw modal */}
              <Congratulations gameover={gameover} />
            </Container>
          </div>
        </ThemeProvider>
      </MuiThemeProvider>
    );
  }
}

export default UI;
