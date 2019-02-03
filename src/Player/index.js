import React from "react";
import intl from "react-intl-universal";

import CardHeader from "../libs/territories-ui/CardHeader";
import IconButton from "../libs/territories-ui/IconButton";
import TimerIcon from "../libs/territories-icons/Timer";
import Tooltip from "../libs/territories-ui/Tooltip";
import PlayersNamesContext from "../playersNamesContext";
import { CenteredCardContent, DetachedCard, StyledAvatar } from "./elements";

const Player = ({
  isCurrent,
  player,
  allCellsCount,
  playerCellsCount,
  readOnly,
  onSkipTurn
}) => (
  <PlayersNamesContext.Consumer>
    {playersNames => {
      const playerName = player && playersNames[player];
      return (
        <DetachedCard>
          <CardHeader
            avatar={
              <StyledAvatar player={player}>
                {playerName && playerName.charAt(0)}
              </StyledAvatar>
            }
            title={playerName}
            subheader={
              Number.isInteger(playerCellsCount) &&
              Number.isInteger(allCellsCount) &&
              intl.get("player_controls.player_stats", {
                playerCellsCount,
                allCellsCount
              })
            }
          />
          {isCurrent && (
            <CenteredCardContent>
              <Tooltip title={intl.get("player_controls.skip_turn")}>
                <IconButton disabled={readOnly} onClick={onSkipTurn}>
                  <TimerIcon />
                </IconButton>
              </Tooltip>
            </CenteredCardContent>
          )}
        </DetachedCard>
      );
    }}
  </PlayersNamesContext.Consumer>
);

export default Player;
