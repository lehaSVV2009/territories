import React from "react";

import CardHeader from "../libs/territories-ui/CardHeader";
import IconButton from "../libs/territories-ui/IconButton";
import Timer from "../libs/territories-icons/Timer";
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
      subheader={`${playerCellsCount} from ${allCellsCount} occupied`}
    />
    {isCurrent && (
      <CenteredCardContent>
        {/* TODO add tooltips */}
        <IconButton onClick={onSkipTurn}>
          <Timer />
        </IconButton>
      </CenteredCardContent>
    )}
  </DetachedCard>
);

export default Player;
