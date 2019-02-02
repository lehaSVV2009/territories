import React, { Component } from "react";
import intl from "react-intl-universal";

import Chip from "../libs/territories-ui/Chip";
import IconButton from "../libs/territories-ui/IconButton";
import IconVersus from "../libs/territories-icons/Versus";
import {
  AlignCenterItem,
  AlignLeftItem,
  AlignRightItem,
  DetachedButton,
  DetachedContainer
} from "./elements";

const findPlayerSeat = (players, playerName) =>
  players.find(player => player.name === playerName);
const findFreeSeat = players => players.find(player => !player.name);

class StyledRoom extends Component {
  handleJoinClick = () => {
    const { roomId, players } = this.props;
    this.props.onJoin(roomId, findFreeSeat(players).id);
  };

  handleLeaveClick = () => {
    this.props.onLeave(this.props.roomId);
  };

  handlePlayClick = () => {
    const { roomId, players, playerName } = this.props;
    this.props.onPlay(
      roomId,
      `${findPlayerSeat(players, playerName).id}`,
      players.length
    );
  };

  handleSpectateClick = () => {
    const { roomId, players } = this.props;
    this.props.onSpectate(roomId, players.length);
  };

  render() {
    const { name, playerName, players } = this.props;
    const playerSeat = findPlayerSeat(players, playerName);
    const freeSeat = findFreeSeat(players);

    return (
      <DetachedContainer alignItems="center">
        <AlignLeftItem flex={3}>
          <Chip label={name} />
        </AlignLeftItem>
        <AlignRightItem flex={2}>{players[0].name || ""}</AlignRightItem>
        <AlignCenterItem flex={2}>
          <IconButton disabled>
            <IconVersus />
          </IconButton>
        </AlignCenterItem>
        <AlignLeftItem flex={2}>{players[1].name || ""}</AlignLeftItem>
        <AlignRightItem flex={3}>
          {playerSeat && (
            <DetachedButton variant="outlined" onClick={this.handleLeaveClick}>
              {intl.get("online.room_leave")}
            </DetachedButton>
          )}
          {freeSeat && !playerSeat && (
            <DetachedButton variant="outlined" onClick={this.handleJoinClick}>
              {intl.get("online.room_join")}
            </DetachedButton>
          )}
          {!freeSeat && playerSeat && (
            <DetachedButton variant="outlined" onClick={this.handlePlayClick}>
              {intl.get("online.room_play")}
            </DetachedButton>
          )}
          {!freeSeat && !playerSeat && (
            <DetachedButton
              variant="outlined"
              onClick={this.handleSpectateClick}
            >
              {intl.get("online.room_spectate")}
            </DetachedButton>
          )}
        </AlignRightItem>
      </DetachedContainer>
    );
  }
}

export default StyledRoom;
