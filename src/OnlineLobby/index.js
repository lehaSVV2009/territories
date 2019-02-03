import React, { Component, Fragment } from "react";
import intl from "react-intl-universal";

import OnlineLogin from "../OnlineLogin";
import OnlineRooms from "../OnlineRooms";
import OnlineExit from "../OnlineExit";
import { Error } from "./elements";

const selectGameName = props => props.gameComponents[0].game.name;
const selectAllPlayersNames = props => {
  const playersNames = [];

  if (!Array.isArray(props.gameInstances)) {
    return playersNames;
  }

  props.gameInstances.forEach(gameInstance => {
    gameInstance.players.forEach(player => {
      if (player.name) {
        playersNames.push(player.name);
      }
    });
  });

  return playersNames;
};

class OnlineLobby extends Component {
  componentDidMount() {
    // Refresh all rooms every 5 seconds
    setInterval(() => this.props.onRefreshRooms(), 5000);
  }

  handleLoginClick = name => {
    this.props.onEnterLobby(name);
  };

  handleLogoutClick = () => {
    const { gameInstances, playerName } = this.props;
    const currentPlayerGames = gameInstances.filter(gameInstance =>
      gameInstance.players.some(player => player.name === playerName)
    );
    currentPlayerGames.forEach(game => this.handleLeaveRoomClick(game.gameID));
    this.props.onExitLobby();
  };

  handleCreateRoomClick = () => {
    this.props.onCreateRoom(selectGameName(this.props), 2);
  };

  handleJoinRoomClick = (gameId, playerId) => {
    this.props.onJoinRoom(selectGameName(this.props), gameId, playerId);
  };

  handleLeaveRoomClick = gameId => {
    this.props.onLeaveRoom(selectGameName(this.props), gameId);
  };

  handlePlayClick = (gameId, playerId, numPlayers) => {
    this.props.onStartGame(selectGameName(this.props), {
      gameID: gameId,
      playerID: playerId,
      numPlayers
    });
  };

  handleSpectateClick = (gameId, numPlayers) => {
    this.props.onStartGame(selectGameName(this.props), {
      gameID: gameId,
      numPlayers
    });
  };

  handleExitRoomClick = () => {
    const { gameID } = this.props.runningGame;
    this.props.onExitRoom();
    setTimeout(() => this.handleLeaveRoomClick(gameID), 100);
  };

  render() {
    const {
      errorMsg,
      phase,
      playerName,
      gameInstances,
      gameComponents,
      runningGame
    } = this.props;

    if (errorMsg) {
      return <Error>Error: {errorMsg}</Error>;
    }

    if (phase === "enter") {
      return (
        <OnlineLogin
          playerName={playerName}
          playersNames={selectAllPlayersNames(this.props)}
          onLogin={this.handleLoginClick}
        />
      );
    }

    if (phase === "list") {
      return (
        <Fragment>
          <OnlineExit
            exitButtonLabel={intl.get("online.logout")}
            playerName={playerName}
            onExit={this.handleLogoutClick}
          />
          <OnlineRooms
            gameComponents={gameComponents}
            gameInstances={gameInstances}
            playerName={playerName}
            onCreate={this.handleCreateRoomClick}
            onJoin={this.handleJoinRoomClick}
            onLeave={this.handleLeaveRoomClick}
            onPlay={this.handlePlayClick}
            onSpectate={this.handleSpectateClick}
            onLogout={this.handleLogoutClick}
          />
        </Fragment>
      );
    }

    if (phase === "play") {
      return (
        <Fragment>
          <OnlineExit
            exitButtonLabel={intl.get("online.exit")}
            playerName={playerName}
            onExit={this.handleExitRoomClick}
          />
          {runningGame && (
            <runningGame.app
              gameID={runningGame.gameID}
              playerID={runningGame.playerID}
              credentials={runningGame.credentials}
            />
          )}
        </Fragment>
      );
    }

    return "Phase is unknown...";
  }
}

export default OnlineLobby;
