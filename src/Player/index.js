import React from "react";
import intl from "react-intl-universal";

import CardHeader from "../libs/territories-ui/CardHeader";
import IconButton from "../libs/territories-ui/IconButton";
import TimerIcon from "../libs/territories-icons/Timer";
import Tooltip from "../libs/territories-ui/Tooltip";
import { CenteredCardContent, DetachedCard, StyledAvatar } from "./elements";

const Player = ({
  isCurrent,
  player,
  allCellsCount,
  playerCellsCount,
  onSkipTurn
}) => (
  <DetachedCard>
    <CardHeader
      avatar={<StyledAvatar player={player}>{player}</StyledAvatar>}
      title={`${intl.get("player_controls.player")} ${player}`}
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
          <IconButton onClick={onSkipTurn}>
            <TimerIcon />
          </IconButton>
        </Tooltip>
      </CenteredCardContent>
    )}
  </DetachedCard>
);

export default Player;
