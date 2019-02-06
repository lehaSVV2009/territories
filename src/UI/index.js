import React, { Component } from "react";
import axios from "axios";

import Board from "../Board";
import Congratulations from "../Congratulations";
import { Container, Item } from "../libs/territories-ui/Grid";
import LinearProgress from "../libs/territories-ui/LinearProgress";
import PlayersControls from "../PlayersControls";
import PlayersNamesContext from "../playersNamesContext";
import { selectGameover, PLAYER_1, PLAYER_2 } from "../gameUtils";
import { DetachedItem } from "./elements";

const CELL_RADIUS = 10;

class UI extends Component {
  state = {
    isLoadingNames: false,
    playersNames: {
      [PLAYER_1]: "Alice",
      [PLAYER_2]: "Bob"
    }
  };

  componentDidMount() {
    const { ai, isMultiplayer, gameID } = this.props;

    if (ai) {
      this.setState({
        playersNames: {
          ...this.state.playersNames,
          [ai.playerID]: ai.name || "Bot"
        }
      });
      return;
    }

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

  componentDidUpdate(prevProps) {
    const { ai } = this.props;
    if (!ai) {
      return;
    }

    const { currentPlayer } = this.props.ctx;
    if (ai.playerID !== currentPlayer) {
      return;
    }

    if (prevProps.ctx.currentPlayer !== currentPlayer) {
      setTimeout(() => {
        this.handleStartRollDices();
      }, 500);
    }

    const { dices, board } = this.props.G;
    if (
      dices &&
      prevProps.G.dices &&
      prevProps.G.dices !== dices &&
      dices[0] !== 0 &&
      // Skip bot recalculation after rotating
      dices[0] !== prevProps.G.dices[1] &&
      dices[1] !== prevProps.G.dices[0]
    ) {
      const rectangleHeight = dices[0];
      const rectangleWidth = dices[1];
      const result = ai.findBestPlaceForRectangle({
        currentPlayer,
        rectangleHeight,
        rectangleWidth,
        rows: board
      });
      if (result.rowIndex === -1 || result.columnIndex === -1) {
        // Skip turn if there are no available cells
        setTimeout(() => {
          this.handleEndTurn();
        }, 1000);
      } else if (result.rectangleHeight !== rectangleHeight) {
        // Rotate if needed
        setTimeout(() => {
          this.handleRotateRectangle();
          setTimeout(() => {
            this.handleDropRectangle(result);
          }, 1000);
        }, 1000);
      } else {
        setTimeout(() => {
          this.handleDropRectangle(result);
        }, 1000);
      }
    }
  }

  handleStartRollDices = () => {
    this.props.moves.startRollDices();
  };

  handleFinishRollDices = () => {
    const {
      isMultiplayer,
      playerID,
      ctx: { currentPlayer }
    } = this.props;
    // Skip roll finish for multiplayers
    if (isMultiplayer && playerID !== currentPlayer) {
      return;
    }
    this.props.moves.finishRollDices();
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
      G: { board, dices, rollingDices, allCellsCount, occupiedCounters },
      ctx: { currentPlayer },
      isActive,
      ai
    } = this.props;
    const { isLoadingNames, playersNames } = this.state;

    if (isLoadingNames) {
      return <LinearProgress color="primary" />;
    }

    const readOnly = !isActive || (ai && ai.playerID === currentPlayer);

    return (
      <PlayersNamesContext.Provider value={playersNames}>
        <Container column>
          <DetachedItem center>
            <PlayersControls
              cellRadius={CELL_RADIUS}
              dices={dices}
              rollingDices={rollingDices}
              currentPlayer={currentPlayer}
              allCellsCount={allCellsCount}
              occupiedCounters={occupiedCounters}
              readOnly={readOnly}
              onStartRollDices={this.handleStartRollDices}
              onFinishRollDices={this.handleFinishRollDices}
              onRotateRectangle={this.handleRotateRectangle}
              onSkipTurn={this.handleEndTurn}
            />
          </DetachedItem>
          <Item center>
            <Board
              disabled={readOnly || !dices || dices[0] === 0}
              cellRadius={CELL_RADIUS}
              rows={board}
              rectangleHeight={dices ? dices[0] : 0}
              rectangleWidth={dices ? dices[1] : 0}
              currentPlayer={currentPlayer}
              onDropRectangle={this.handleDropRectangle}
            />
          </Item>
          {/* Win or Draw modal */}
          <Congratulations
            gameover={selectGameover({ allCellsCount, occupiedCounters })}
          />
        </Container>
      </PlayersNamesContext.Provider>
    );
  }
}

export default UI;
