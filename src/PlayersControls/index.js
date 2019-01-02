import React from "react";
import { Container, Item } from "../libs/territories-ui/Grid";
import DiceRoller from "../DiceRoller";
import * as GameUtils from "../gameUtils";
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
  allCellsCount,
  occupiedCounters,
  onRollDices,
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
                  <IconButton size="small" onClick={onRotateRectangle}>
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
              <DiceRoller
                buttonProps={{
                  color: GameUtils.isPlayer1(currentPlayer)
                    ? "primary"
                    : "secondary"
                }}
                onRoll={onRollDices}
              />
            </FullHeightContainer>
          )}
        </FixedSizePaper>
      </div>
    </Item>
    <Item>
      <Player
        player={GameUtils.PLAYER_2}
        isCurrent={GameUtils.isPlayer2(currentPlayer)}
        allCellsCount={allCellsCount}
        playerCellsCount={occupiedCounters[GameUtils.PLAYER_2]}
        onSkipTurn={onSkipTurn}
      />
    </Item>
  </Container>
);

export default PlayersControls;
