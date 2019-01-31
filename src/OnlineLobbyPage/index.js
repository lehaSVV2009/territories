import React from "react";
// TODO import from territories-core
import Game from "../Game";
import LobbyLogic from "../libs/boardgame.io/react";
import OnlineLobby from "../OnlineLobby";

import UI from "../UI";
// import StyledLobby from "./StyledLobby";

const game = Game({});
game.name = "territories";

game.minPlayers = game.maxPlayers = 2;

const OnlineLobbyPage = () => (
  <LobbyLogic
    gameServer={process.env.REACT_APP_API_URL}
    lobbyServer={process.env.REACT_APP_API_URL}
    gameComponents={[{ game, board: UI }]}
    renderer={({
      errorMsg,
      gameComponents,
      gameInstances,
      phase,
      playerName,
      runningGame,
      handleEnterLobby,
      handleExitLobby,
      handleCreateRoom,
      handleJoinRoom,
      handleLeaveRoom,
      handleExitRoom,
      handleStartGame
    }) => (
      <OnlineLobby
        errorMsg={errorMsg}
        gameComponents={gameComponents}
        gameInstances={gameInstances}
        phase={phase}
        playerName={playerName}
        runningGame={runningGame}
        onEnterLobby={handleEnterLobby}
        onExitLobby={handleExitLobby}
        onCreateRoom={handleCreateRoom}
        onJoinRoom={handleJoinRoom}
        onLeaveRoom={handleLeaveRoom}
        onExitRoom={handleExitRoom}
        onStartGame={handleStartGame}
      />
    )}
  />
);

export default OnlineLobbyPage;
