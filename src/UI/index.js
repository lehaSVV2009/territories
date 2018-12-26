import React, { Component } from "react";
import { ThemeProvider } from "styled-components";

import Button from "../libs/territories-ui/Button";
import Board from "../Board";
import Congratulations from "../Congratulations";
import { Container, Item } from "../libs/territories-ui/Grid";
import DiceRoller from "../DiceRoller";
import Footer from "../Footer";
import Header from "../Header";
import Player from "../Player";
import Rectangle from "../libs/territories-ui/Rectangle";

const theme = {
  colors: {
    gray: "#DDDDDD",
    green: "green",
    red: "red",
    player1: "#375E97",
    player2: "#FB6542"
  }
};

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
      G: { board, dices },
      ctx: { currentPlayer, gameover }
    } = this.props;

    return (
      <ThemeProvider theme={theme}>
        <Container column>
          <Item>
            {/* TODO add some nice title */}
            <Header />
          </Item>
          <Item centered>
            <Container>
              <Item>
                <Player player={currentPlayer} />
              </Item>
              <Item centered>
                <div
                  style={{
                    textAlign: "center"
                  }}
                >
                  {/* TODO add fixed-size paper with roll button or rectangle + rotate */}
                  <div
                    style={{
                      minHeight: CELL_RADIUS * 20,
                      minWidth: CELL_RADIUS * 20
                    }}
                  >
                    <DiceRoller onRoll={this.handleRollDices} />

                    {/* TODO Think of how not to move game board down after rotating and dice rolling */}
                    {dices && dices[0] !== 0 && (
                      <Container column>
                        <Item>
                          <Button onClick={this.handleRotateRectangle}>
                            Rotate
                          </Button>
                        </Item>
                        <Item centered>
                          <Rectangle
                            rows={Array(dices[0]).fill(Array(dices[1]).fill())}
                            cellRadius={CELL_RADIUS}
                          />
                        </Item>
                        <Item>Mouse over board</Item>
                      </Container>
                    )}
                  </div>
                  {/* TODO replace style with className */}
                  <Button onClick={this.handleEndTurn}>Skip Turn</Button>
                </div>
              </Item>

              <Item>
                <Player player={currentPlayer} />
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
          <Item>
            {/* TODO add some links */}
            <Footer />
          </Item>

          {/* Win or Draw modal */}
          <Congratulations gameover={gameover} />
        </Container>
      </ThemeProvider>
    );
  }
}

export default UI;
