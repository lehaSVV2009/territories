import React, { Component } from "react";
import axios from "axios";

import Board from "../Board";
import Congratulations from "../Congratulations";
import { Container, Item } from "../libs/territories-ui/Grid";
import LinearProgress from "../libs/territories-ui/LinearProgress";
import PlayersControls from "../PlayersControls";
import PlayersNamesContext from "../playersNamesContext";
import { DetachedItem } from "./elements";

const CELL_RADIUS = 10;

class UI extends Component {
  state = {
    isLoadingNames: false,
    playersNames: {
      0: "Alice",
      1: "Bob"
    }
  };

  componentDidMount() {
    const { isMultiplayer, gameID } = this.props;
    if (!isMultiplayer || !gameID) {
      return;
    }

    // Load players names if it is a multiplayer
    // Cause boardgame.io framework doesn't support name handling..
    this.setState({ isLoadingNames: true });
    axios
      .get(`${process.env.REACT_APP_API_URL}/games/territories`)
      .then(response => {
        const gameInstance =
          response.data &&
          response.data.gameInstances.find(
            gameInstance => gameInstance.gameID === gameID
          );
        if (!gameInstance) {
          return;
        }
        this.setState({
          playersNames: gameInstance.players.reduce((object, player) => {
            object[`${player.id}`] = player.name;
            return object;
          }, {})
        });
      })
      .finally(() => this.setState({ isLoadingNames: false }));
  }

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
    const { isLoadingNames, playersNames } = this.state;

    if (isLoadingNames) {
      return <LinearProgress color="primary" />;
    }

    return (
      <PlayersNamesContext.Provider value={playersNames}>
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
              disabled={!dices || dices[0] === 0}
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
      </PlayersNamesContext.Provider>
    );
  }
}

export default UI;
