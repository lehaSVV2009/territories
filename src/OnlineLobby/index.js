import React, { Component } from "react";
import OnlineRoom from "../OnlineRoom";

const selectGameName = props => props.gameComponents[0].game.name;

class OnlineLobby extends Component {
  constructor(props) {
    super(props);
    this.state = {
      changingPlayerName: props.playerName
    };
  }

  handlePlayerNameChange = ({ target: { value } }) => {
    this.setState({ changingPlayerName: value });
  };

  handleEnterClick = () => {
    // TODO check if name is not already taken
    this.props.onEnterLobby(this.state.changingPlayerName);
  };

  handleExitClick = () => {
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
    const { changingPlayerName } = this.state;
    const {
      errorMsg,
      phase,
      playerName,
      gameInstances,
      gameComponents,
      runningGame
    } = this.props;
    const gameName = selectGameName(this.props);

    if (errorMsg) {
      return <div style={{ color: "red" }}>Error: {errorMsg}</div>;
    }

    if (phase === "enter") {
      return (
        <div>
          Enter your name:
          <input
            value={changingPlayerName}
            onChange={this.handlePlayerNameChange}
          />
          <button onClick={this.handleEnterClick}>Login</button>
        </div>
      );
    }

    if (phase === "list") {
      return (
        <div>
          <div>
            <b>Hello, {playerName}</b>
          </div>
          <div>
            {gameComponents.length === 1 && (
              <button
                style={{ color: "red" }}
                onClick={this.handleCreateRoomClick}
              >
                Create new room for {gameComponents[0].game.name}
              </button>
            )}
          </div>
          <div>
            {gameInstances.length === 0 ? (
              <i>There are no rooms</i>
            ) : (
              gameInstances.map(gameInstance => (
                <OnlineRoom
                  key={`game-${gameInstance.gameID}`}
                  gameName={gameName}
                  roomId={gameInstance.gameID}
                  players={gameInstance.players}
                  playerName={playerName}
                  onJoin={this.handleJoinRoomClick}
                  onLeave={this.handleLeaveRoomClick}
                  onPlay={this.handlePlayClick}
                  onSpectate={this.handleSpectateClick}
                />
              ))
            )}
          </div>
          <div>
            <button onClick={this.handleExitClick}>Logout</button>
          </div>
        </div>
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
