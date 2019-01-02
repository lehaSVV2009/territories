import React, { Component } from "react";
import { createMuiTheme } from "@material-ui/core/styles";
import { MuiThemeProvider } from "@material-ui/core";
import { ThemeProvider } from "styled-components";

import Board from "../Board";
import Congratulations from "../Congratulations";
import { Container, Item } from "../libs/territories-ui/Grid";
import DiceRoller from "../DiceRoller";
import * as GameUtils from "../gameUtils";
import Header from "../Header";
import IconButton from "../libs/territories-ui/IconButton";
import Paper from "../libs/territories-ui/Paper";
import Player from "../Player";
import Rectangle from "../libs/territories-ui/Rectangle";
import RotateIcon from "../libs/territories-icons/Rotate";
import Tooltip from "../libs/territories-ui/Tooltip";

const theme = {
  colors: {
    white: "#FFFFFF",
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

// TODO simplify UI to have only grid stuff and handlers
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
      // TODO add material-ui theme
      <MuiThemeProvider theme={muiTheme}>
        <ThemeProvider theme={theme}>
          <div>
            <Header />
            <Container column>
              {/* TODO change style to classname */}
              {/* Margin from fixed appbar and from board */}
              <Item
                centered
                style={{ marginTop: "80px", marginBottom: "24px" }}
              >
                <Container>
                  <Item>
                    <Player
                      player={GameUtils.PLAYER_1}
                      isCurrent={GameUtils.isPlayer1(currentPlayer)}
                      allCellsCount={allCellsCount}
                      playerCellsCount={occupiedCounters[GameUtils.PLAYER_1]}
                      onSkipTurn={this.handleEndTurn}
                    />
                  </Item>
                  <Item centered>
                    <div>
                      {/* TODO add fixed-size paper with roll button or rectangle + rotate */}
                      <Paper
                        style={{
                          minHeight: CELL_RADIUS * 20,
                          minWidth: CELL_RADIUS * 20
                        }}
                      >
                        {dices && dices[0] !== 0 ? (
                          <Container
                            column
                            center
                            alignItems="center"
                            style={{ height: "200px" }}
                          >
                            <Item>
                              <Tooltip title="Rotate rectangle">
                                <IconButton
                                  size="small"
                                  onClick={this.handleRotateRectangle}
                                >
                                  <RotateIcon />
                                </IconButton>
                              </Tooltip>
                            </Item>
                            <Item centered>
                              <Rectangle
                                rows={Array(dices[0]).fill(
                                  Array(dices[1]).fill()
                                )}
                                cellRadius={CELL_RADIUS}
                              />
                            </Item>
                          </Container>
                        ) : (
                          <Container
                            column
                            center
                            alignItems="center"
                            style={{ height: "200px" }}
                          >
                            <DiceRoller
                              buttonProps={{
                                color: GameUtils.isPlayer1(currentPlayer)
                                  ? "primary"
                                  : "secondary"
                              }}
                              onRoll={this.handleRollDices}
                            />
                          </Container>
                        )}
                      </Paper>
                    </div>
                  </Item>
                  <Item>
                    <Player
                      player={GameUtils.PLAYER_2}
                      isCurrent={GameUtils.isPlayer2(currentPlayer)}
                      allCellsCount={allCellsCount}
                      playerCellsCount={occupiedCounters[GameUtils.PLAYER_2]}
                      onSkipTurn={this.handleEndTurn}
                    />
                  </Item>
                </Container>
              </Item>
              <Item centered>
                <Board
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
