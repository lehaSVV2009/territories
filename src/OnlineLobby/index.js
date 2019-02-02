import React, { Component } from "react";

import OnlineLogin from "../OnlineLogin";
import OnlineRooms from "../OnlineRooms";

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
  handleLoginClick = name => {
    this.props.onEnterLobby(name);
  };

  handleLogoutClick = () => {
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
    this.props.onExitRoom();
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
      return <div style={{ color: "red" }}>Error: {errorMsg}</div>;
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
      );
    }

    if (phase === "play") {
      return (
        <div>
          <div>
            <button onClick={this.handleExitRoomClick}>Exit game</button>
          </div>
          {runningGame && (
            <runningGame.app
              gameID={runningGame.gameID}
              playerID={runningGame.playerID}
              credentials={runningGame.credentials}
            />
          )}
        </div>
      );
    }

    return "Phase is unknown...";
  }
}

export default OnlineLobby;
