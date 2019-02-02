import React, { Component } from "react";
import intl from "react-intl-universal";

import Card from "../libs/territories-ui/Card";
import CardContent from "../libs/territories-ui/CardContent";
import CardHeader from "../libs/territories-ui/CardHeader";
import IconAdd from "../libs/territories-icons/Add";
import IconButton from "../libs/territories-ui/IconButton";
import OnlineRoom from "../OnlineRoom";
import Tooltip from "../libs/territories-ui/Tooltip";
import { Layout } from "./elements";

class OnlineRooms extends Component {
  handleCreateRoomClick = () => {
    this.props.onCreate();
  };

  handleJoinRoomClick = (gameId, playerId) => {
    this.props.onJoin(gameId, playerId);
  };

  handleLeaveRoomClick = gameId => {
    this.props.onLeave(gameId);
  };

  handlePlayClick = (gameId, playerId, numPlayers) => {
    this.props.onPlay(gameId, playerId, numPlayers);
  };

  handleSpectateClick = (gameId, numPlayers) => {
    this.props.onPlay(gameId, numPlayers);
  };

  render() {
    const { gameComponents, gameInstances, playerName } = this.props;

    return (
      <Layout>
        <Card>
          <CardHeader
            title={intl.get("online.rooms_title")}
            subheader={intl.get("online.rooms_subtitle")}
            action={
              gameComponents.length === 1 && (
                <Tooltip title={intl.get("online.new_room")}>
                  <IconButton
                    disabled={gameInstances.length > 4}
                    color="primary"
                    variant="extendedFab"
                    onClick={this.handleCreateRoomClick}
                  >
                    <IconAdd />
                  </IconButton>
                </Tooltip>
              )
            }
          />
          <CardContent>
            {gameInstances.map((gameInstance, index) => (
              <OnlineRoom
                key={`game-${gameInstance.gameID}`}
                name={`territories-${index + 1}`}
                roomId={gameInstance.gameID}
                players={gameInstance.players}
                playerName={playerName}
                onJoin={this.handleJoinRoomClick}
                onLeave={this.handleLeaveRoomClick}
                onPlay={this.handlePlayClick}
                onSpectate={this.handleSpectateClick}
              />
            ))}
          </CardContent>
        </Card>
      </Layout>
    );
  }
}

export default OnlineRooms;
