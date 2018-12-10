import React, { Component } from "react";
import { DragDropContextProvider } from "react-dnd";
import HTML5Backend from "react-dnd-html5-backend";
import { ThemeProvider } from "styled-components";

import Board from "../base/Board";
import Button from "../base/Button";
import { Container, Item } from "../base/Grid";
import DraggableRectangle from "../DraggableRectangle";
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

  handleDropSquare = ({
    rowIndex,
    columnIndex,
    rectangleHeight,
    rectangleWidth
  }) => {
    debugger
    this.props.moves.dropSquare(
      rowIndex,
      columnIndex,
      rectangleHeight,
      rectangleWidth
    );
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
              <Footer />
            </Item>
          </Container>
        </ThemeProvider>
      </DragDropContextProvider>
    );
  }
}
