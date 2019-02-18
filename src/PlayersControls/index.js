import React from "react";
import intl from "react-intl-universal";
import * as GameUtils from "territories-core";

import Button from "../libs/territories-ui/Button";
import { Container, Item } from "../libs/territories-ui/Grid";
import DicesWindow from "../DicesWindow";
import IconButton from "../libs/territories-ui/IconButton";
import Player from "../Player";
import Rectangle from "../libs/territories-ui/Rectangle";
import RotateIcon from "../libs/territories-icons/Rotate";
import Tooltip from "../libs/territories-ui/Tooltip";
import { FixedSizePaper, FullHeightContainer } from "./elements";

const PlayersControls = ({
  cellRadius,
  currentPlayer,
  dices,
  rollingDices,
  allCellsCount,
  occupiedCounters,
  readOnly,
  onStartRollDices,
  onFinishRollDices,
  onRotateRectangle,
  onSkipTurn
}) => (
  <Container>
    <Item>
      <Player
        player={GameUtils.PLAYER_1}
        isCurrent={GameUtils.isPlayer1(currentPlayer)}
        allCellsCount={allCellsCount}
        playerCellsCount={occupiedCounters[GameUtils.PLAYER_1]}
        readOnly={readOnly}
        onSkipTurn={onSkipTurn}
      />
    </Item>
    <Item center>
      <div>
        <FixedSizePaper
          height={`${cellRadius * 25}px`}
          width={`${cellRadius * 25}px`}
        >
          {dices && dices[0] !== 0 ? (
            <FullHeightContainer column center alignItems="center">
              <Item>
                <Tooltip title="Rotate rectangle">
                  <IconButton
                    disabled={readOnly}
                    size="small"
                    onClick={onRotateRectangle}
                  >
                    <RotateIcon />
                  </IconButton>
                </Tooltip>
              </Item>
              <Item center>
                <Rectangle
                  rows={Array(dices[0]).fill(Array(dices[1]).fill())}
                  cellRadius={cellRadius}
                />
              </Item>
            </FullHeightContainer>
          ) : (
            <FullHeightContainer column center alignItems="center">
              <Button
                disabled={readOnly}
                color={
                  GameUtils.isPlayer1(currentPlayer) ? "primary" : "secondary"
                }
                size="large"
                variant="contained"
                onClick={onStartRollDices}
              >
                {intl.get("player_controls.roll_dices")}
              </Button>
            </FullHeightContainer>
          )}
        </FixedSizePaper>
        <DicesWindow
          rollingDices={rollingDices}
          onFinishRoll={onFinishRollDices}
        />
      </div>
    </Item>
    <Item>
      <Player
        player={GameUtils.PLAYER_2}
        isCurrent={GameUtils.isPlayer2(currentPlayer)}
        allCellsCount={allCellsCount}
        playerCellsCount={occupiedCounters[GameUtils.PLAYER_2]}
        readOnly={readOnly}
        onSkipTurn={onSkipTurn}
      />
    </Item>
  </Container>
);

export default PlayersControls;
