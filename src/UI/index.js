import React, { Component } from "react";
import { DragDropContextProvider } from "react-dnd";
import HTML5Backend from "react-dnd-html5-backend";
import { ThemeProvider } from "styled-components";

import Board from "../base/Board";
import Button from "../base/Button";
import { Container, Item } from "../base/Grid";
import DraggableBoard from "../DraggableBoard";
import DropzoneCell from "../DropzoneCell";
import Footer from "../Footer";
import Header from "../Header";

const theme = {
  colors: {
    gray: "#DDDDDD",
    green: "#00FF00"
  }
};

export default class UI extends Component {
  handleRollDices = () => {
    this.props.moves.rollDices();
  };

  handleRotateRectangle = () => {
    this.props.moves.switchDices();
  };

  handleDropSquare = (rowIndex, columnIndex, squareSize) => {
    this.props.moves.dropSquare(rowIndex, columnIndex, squareSize);
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
              <Header />
            </Item>
            <Item flex="auto">
              <div>Player: {currentPlayer}</div>
              <Button onClick={this.handleRollDices}>Roll Dices</Button>
              <div>Dices: {dices}</div>
              {gameover &&
                (gameover.winner ? `Winner: ${gameover.winner}` : "Draw!")}
              <Button onClick={this.handleRotateRectangle}>Rotate</Button>
              <DraggableBoard
                data={Array(dices[0]).fill(Array(dices[1]).fill())}
              />

              <Board
                data={board}
                cellRenderer={({ rowIndex, columnIndex, cellClassName }) => (
                  <DropzoneCell
                    columnIndex={columnIndex}
                    cellClassName={cellClassName}
                  />
                )}
              />
            </Item>
            <Item>
              <Footer />
            </Item>
          </Container>
        </ThemeProvider>
      </DragDropContextProvider>
    );
  }
}
