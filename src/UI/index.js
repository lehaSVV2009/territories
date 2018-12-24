import React, { Component } from "react";
import { DragDropContext } from "react-dnd";
import MultiBackend from "react-dnd-multi-backend";
import HTML5toTouch from "react-dnd-multi-backend/lib/HTML5toTouch";
import { ThemeProvider } from "styled-components";

import Board from "../libs/territories-ui/Board";
import Button from "../libs/territories-ui/Button";
import Congratulations from "../Congratulations";
import { Container, Item } from "../libs/territories-ui/Grid";
import DiceRoller from "../DiceRoller";
import DraggableRectangle from "../DraggableRectangle";
import DropzoneCell from "../DropzoneCell";
import Footer from "../Footer";
import Header from "../Header";
import Player from "../Player";

const theme = {
  colors: {
    gray: "#DDDDDD",
    green: "#00FF00",
    player1: "#375E97",
    player2: "#FB6542"
  }
};

const CELL_RADIUS = 5;

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
        <Container fullPage column>
          <Item>
            {/* TODO add some nice title */}
            <Header />
          </Item>
          <Item flex="auto">
            {/* TODO Add current player right side within highlighted from all players */}
            <Player player={currentPlayer} />

            <Board
              rows={board}
              cellRenderer={({ value, rowIndex, columnIndex }) => (
                <DropzoneCell
                  rowIndex={rowIndex}
                  columnIndex={columnIndex}
                  value={value}
                  rows={board}
                  cellRadius={CELL_RADIUS}
                  currentPlayer={currentPlayer}
                  onDropCell={this.handleDropRectangle}
                />
              )}
            />

            <Button onClick={this.handleEndTurn}>Skip Turn</Button>

            <DiceRoller onRoll={this.handleRollDices} />

            {/* TODO Think of how not to move game board down after rotating and dice rolling */}
            {dices && dices[0] !== 0 && (
              <Container column>
                <Item>Drag rectangle and drop to the board</Item>
                <Item>
                  <Button onClick={this.handleRotateRectangle}>Rotate</Button>
                </Item>
              </Container>
            )}

            <DraggableRectangle
              cellRadius={CELL_RADIUS}
              rows={Array(dices[0]).fill(Array(dices[1]).fill())}
            />

            {/* Win or Draw modal */}
            <Congratulations gameover={gameover} />
          </Item>
          <Item>
            {/* TODO add some links */}
            <Footer />
          </Item>
        </Container>
      </ThemeProvider>
    );
  }
}

export default DragDropContext(MultiBackend(HTML5toTouch))(UI);
