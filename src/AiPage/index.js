import React, { Component } from "react";
import { Client } from "boardgame.io/react";
import { Game, PLAYER_2 } from "territories-core";
import styled from "styled-components";
import { Link } from "react-router-dom";

import Button from "../libs/territories-ui/Button";
import Chip from "../libs/territories-ui/Chip";
import IconTune from "@material-ui/icons/Tune";
import IconScience from "@material-ui/icons/Assessment";
import IconRefresh from "@material-ui/icons/Refresh";

import AI, { BOT_TYPES, MODELS_CATALOG } from "../AI";
import UI from "../UI";
import ModelSelectionDrawer from "./ModelSelectionDrawer";

const TopControlsBar = styled.div`
  display: flex;
  align-items: center;
  justify-content: space-between;
  flex-wrap: wrap;
  gap: 12px;
  background-color: #f8fafc;
  border-bottom: 1px solid #e2e8f0;
  padding: 10px 24px;
  margin-bottom: 12px;
`;

const ModelBadgeGroup = styled.div`
  display: flex;
  align-items: center;
  gap: 10px;
`;

const ModelLabel = styled.span`
  font-size: 14px;
  font-weight: 600;
  color: #334155;
`;

const ActionButtonGroup = styled.div`
  display: flex;
  align-items: center;
  gap: 10px;
`;

const BoardGameClient = Client({
  game: Game,
  board: ({ G, ctx, moves, events, isActive, ai }) => (
    <UI
      ai={ai}
      G={G}
      ctx={ctx}
      moves={moves}
      events={events}
      isActive={isActive}
    />
  ),
  debug: false
});

export default class AiPage extends Component {
  state = {
    selectedModel: BOT_TYPES.G40,
    drawerOpen: false,
    gameKey: 0
  };

  handleOpenDrawer = () => {
    this.setState({ drawerOpen: true });
  };

  handleCloseDrawer = () => {
    this.setState({ drawerOpen: false });
  };

  handleSelectModel = modelId => {
    this.setState(prevState => ({
      selectedModel: modelId,
      drawerOpen: false,
      gameKey: prevState.gameKey + 1
    }));
  };

  handleRestartGame = () => {
    this.setState(prevState => ({
      gameKey: prevState.gameKey + 1
    }));
  };

  render() {
    const { selectedModel, drawerOpen, gameKey } = this.state;
    const currentModelMeta =
      MODELS_CATALOG.find(m => m.id === selectedModel) || MODELS_CATALOG[0];

    const bot = AI({
      playerID: PLAYER_2,
      type: selectedModel,
      name: currentModelMeta.name
    });

    return (
      <div>
        <TopControlsBar>
          <ModelBadgeGroup>
            <ModelLabel>Бот:</ModelLabel>
            <Chip
              label={currentModelMeta.title}
              style={{
                backgroundColor: currentModelMeta.color || "#FB6542",
                color: "#ffffff",
                fontWeight: 600,
                fontSize: 13
              }}
            />
            <Chip
              label={currentModelMeta.badge}
              variant="outlined"
              size="small"
              style={{ fontWeight: 600, fontSize: 11 }}
            />
          </ModelBadgeGroup>

          <ActionButtonGroup>
            <Button
              variant="contained"
              color="secondary"
              size="small"
              onClick={this.handleOpenDrawer}
              style={{ fontWeight: 600 }}
            >
              <IconTune style={{ marginRight: 6, fontSize: 18 }} />
              Выбор бота
            </Button>

            <Button
              variant="outlined"
              color="primary"
              size="small"
              onClick={this.handleRestartGame}
              style={{ fontWeight: 600 }}
            >
              <IconRefresh style={{ marginRight: 6, fontSize: 18 }} />
              Заново
            </Button>

            <Button
              variant="outlined"
              color="primary"
              size="small"
              component={Link}
              to="/ai-lab"
              style={{ fontWeight: 600 }}
            >
              <IconScience style={{ marginRight: 6, fontSize: 18 }} />
              Лаборатория
            </Button>
          </ActionButtonGroup>
        </TopControlsBar>

        {/* BoardGame.io Client instance */}
        <BoardGameClient key={gameKey} ai={bot} />

        {/* Model Selection Drawer */}
        <ModelSelectionDrawer
          open={drawerOpen}
          onClose={this.handleCloseDrawer}
          selectedModel={selectedModel}
          onSelectModel={this.handleSelectModel}
        />
      </div>
    );
  }
}
