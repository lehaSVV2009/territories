import React, { Component } from "react";
import { DragDropContextProvider } from "react-dnd";
import HTML5Backend from "react-dnd-html5-backend";
import { ThemeProvider } from "styled-components";

import Board from "../base/Board";
import Button from "../base/Button";
import { Container, Item } from "../base/Grid";
import Dice from "../base/Dice";
import DraggableRectangle from "../DraggableRectangle";
import DropzoneCell from "../DropzoneCell";
import Footer from "../Footer";
import Header from "../Header";
import Player from "../base/Player";

const theme = {
  colors: {
    gray: "#DDDDDD",
    green: "#00FF00",
    player1: "#375E97",
    player2: "#FB6542"
  }
};

export default class UI extends Component {
  handleRollDices = () => {
    this.props.moves.rollDices();
  };

  handleRotateRectangle = () => {
    this.props.moves.switchDices();
  };

  handleDropSquare = ({
    rowIndex,
    columnIndex,
    rectangleHeight,
    rectangleWidth
  }) => {
    this.props.moves.dropSquare(
      rowIndex,
      columnIndex,
      rectangleHeight,
      rectangleWidth
    );
    this.props.moves.clearDices();
    this.props.events.endTurn();
  };

  render() {
    const {
      G: { board, dices },
      ctx: { currentPlayer, gameover }
    } = this.props;

    return (
      <DragDropContextProvider backend={HTML5Backend}>
        <ThemeProvider theme={theme}>
          <Container fullPage column>
            <Item>
              {/* TODO add some nice title */}
              <Header />
            </Item>
            <Item flex="auto">
              {/* TODO make gameover fullscreen window */}
              {gameover &&
                (gameover.winner ? `Winner: ${gameover.winner}` : "Draw!")}

              {/* TODO Add current player right side within highlighted from all players */}
              <Player player={currentPlayer} />

              {/* TODO Big button with icon in the middle of the screen */}
              <Button onClick={this.handleRollDices}>Roll Dices</Button>

              <Dice />
              {/* TODO Add nice dices animation */}
              {dices && dices[0] !== 0 && (
                <div>
                  Dices: {dices}
                  <Button onClick={this.handleRotateRectangle}>Rotate</Button>
                </div>
              )}
              {/* TODO Think of how not to move game board down after rotating and dice rolling */}
              <DraggableRectangle
                rows={Array(dices[0]).fill(Array(dices[1]).fill())}
              />

              <Board
                rows={board}
                cellRenderer={({
                  value,
                  rowIndex,
                  columnIndex,
                  cellClassName
                }) => (
                  <DropzoneCell
                    rowIndex={rowIndex}
                    columnIndex={columnIndex}
                    value={value}
                    cellClassName={cellClassName}
                    onDropCell={this.handleDropSquare}
                  />
                )}
              />
            </Item>
            <Item>
              {/* TODO add some links */}
              <Footer />
            </Item>
          </Container>
        </ThemeProvider>
      </DragDropContextProvider>
    );
  }
}
