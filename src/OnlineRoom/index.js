import React, { Component } from "react";

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
    const { gameName, playerName, players, roomId } = this.props;
    const playerSeat = findPlayerSeat(players, playerName);
    const freeSeat = findFreeSeat(players);

    return (
      <div>
        <span style={{ paddingRight: "30px" }}>{gameName}</span>
        {players.map(player => (
          <span
            key={`game-${roomId}-player-${player.id}`}
            style={{ padding: "10px" }}
          >
            {player.name ? player.name : "free"}
          </span>
        ))}

        {playerSeat && <button onClick={this.handleLeaveClick}>Leave</button>}
        {freeSeat && !playerSeat && (
          <button onClick={this.handleJoinClick}>Join</button>
        )}
        {!freeSeat && playerSeat && (
          <button onClick={this.handlePlayClick}>Play</button>
        )}
        {!freeSeat && !playerSeat && (
          <button onClick={this.handleSpectateClick}>Spectate</button>
        )}
      </div>
    );
  }
}

export default StyledRoom;
