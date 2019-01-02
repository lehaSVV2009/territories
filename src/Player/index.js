import React from "react";

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
      title={`Player ${player}`}
      subheader={
        Number.isInteger(playerCellsCount) &&
        Number.isInteger(allCellsCount) &&
        `${playerCellsCount} from ${allCellsCount} occupied`
      }
    />
    {isCurrent && (
      <CenteredCardContent>
        <Tooltip title="Skip turn">
          <IconButton onClick={onSkipTurn}>
            <TimerIcon />
          </IconButton>
        </Tooltip>
      </CenteredCardContent>
    )}
  </DetachedCard>
);

export default Player;
