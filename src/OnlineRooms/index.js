import React, { Component, Fragment } from "react";
import intl from "react-intl-universal";

import Card from "../libs/territories-ui/Card";
import CardContent from "../libs/territories-ui/CardContent";
import CardHeader from "../libs/territories-ui/CardHeader";
import IconAdd from "../libs/territories-icons/Add";
import IconButton from "../libs/territories-ui/IconButton";
import IconRefresh from "../libs/territories-icons/Refresh";
import OnlineRoom from "../OnlineRoom";
import Tooltip from "../libs/territories-ui/Tooltip";
import { Layout } from "./elements";

class OnlineRooms extends Component {
  handleCreateRoomClick = () => {
    this.props.onCreate();
  };

  handleRefreshRooms = () => {
    this.props.onRefresh();
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
    this.props.onSpectate(gameId, numPlayers);
  };

  render() {
    const { gameInstances, playerName, alreadyJoined } = this.props;

    return (
      <Layout>
        <Card>
          <CardHeader
            title={intl.get("online.rooms_title")}
            subheader={intl.get("online.rooms_subtitle")}
            action={
              <Fragment>
                <Tooltip title={intl.get("online.new_room")}>
                  <IconButton
                    color="primary"
                    disabled={gameInstances.length > 4}
                    onClick={this.handleCreateRoomClick}
                  >
                    <IconAdd />
                  </IconButton>
                </Tooltip>
                <Tooltip title={intl.get("online.refresh")}>
                  <IconButton color="primary" onClick={this.handleRefreshRooms}>
                    <IconRefresh />
                  </IconButton>
                </Tooltip>
              </Fragment>
            }
          />
          <CardContent>
            {gameInstances.map(gameInstance => (
              <OnlineRoom
                key={`game-${gameInstance.gameID}`}
                name={`Territories ${gameInstance.gameID.substring(0, 3)}`}
                roomId={gameInstance.gameID}
                players={gameInstance.players}
                playerName={playerName}
                alreadyJoined={alreadyJoined}
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
