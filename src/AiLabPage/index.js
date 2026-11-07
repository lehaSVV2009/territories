import React, { Component } from "react";
import styled from "styled-components";
import { Link } from "react-router-dom";
import Typography from "@material-ui/core/Typography";
import Tabs from "@material-ui/core/Tabs";
import Tab from "@material-ui/core/Tab";

import Button from "../libs/territories-ui/Button";
import Chip from "../libs/territories-ui/Chip";
import IconArrowBack from "@material-ui/icons/ArrowBack";
import IconCompare from "@material-ui/icons/CompareArrows";
import IconPlay from "@material-ui/icons/PlayArrow";
import IconTune from "@material-ui/icons/Tune";
import IconGrid from "@material-ui/icons/GridOn";
import IconAssessment from "@material-ui/icons/Assessment";

import ArenaTab from "./ArenaTab";
import StepSimulatorTab from "./StepSimulatorTab";
import HeuristicsTab from "./HeuristicsTab";
import SpatialMatrixTab from "./SpatialMatrixTab";
import PassportTab from "./PassportTab";
import { BOT_TYPES } from "../AI";

const PageContainer = styled.div`
  max-width: 1200px;
  margin: 0 auto;
  padding: 24px 16px 48px;
  font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif;
  color: #1e293b;
`;

const HeaderNav = styled.div`
  display: flex;
  align-items: center;
  justify-content: space-between;
  flex-wrap: wrap;
  gap: 16px;
  margin-bottom: 20px;
  padding-bottom: 16px;
  border-bottom: 1px solid #e2e8f0;
`;

const TitleGroup = styled.div`
  display: flex;
  flex-direction: column;
  gap: 4px;
`;

const TabsWrapper = styled.div`
  background-color: #ffffff;
  border-radius: 8px;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.06);
  margin-bottom: 24px;
  border: 1px solid #e2e8f0;
  overflow-x: auto;
`;

const TabContent = styled.div`
  min-height: 500px;
`;

export default class AiLabPage extends Component {
  state = {
    activeTab: 0,
    replayMatch: null,
    selectedPassportModelId: BOT_TYPES.APPEX,
    arenaP1Model: BOT_TYPES.APPEX,
    simulatorP1Model: BOT_TYPES.APPEX,
    heuristicsModelA: BOT_TYPES.APPEX
  };

  handleTabChange = (event, activeTab) => {
    this.setState({ activeTab });
  };

  handleSelectReplayMatch = match => {
    this.setState({
      replayMatch: match,
      activeTab: 1
    });
  };

  handleNavigateToArena = modelId => {
    this.setState({
      activeTab: 0,
      arenaP1Model: modelId
    });
  };

  handleNavigateToSimulator = modelId => {
    this.setState({
      activeTab: 1,
      simulatorP1Model: modelId,
      replayMatch: null
    });
  };

  handleNavigateToHeuristics = modelId => {
    this.setState({
      activeTab: 2,
      heuristicsModelA: modelId
    });
  };

  handleViewPassport = modelId => {
    this.setState({
      activeTab: 4,
      selectedPassportModelId: modelId
    });
  };

  render() {
    const {
      activeTab,
      replayMatch,
      selectedPassportModelId,
      arenaP1Model,
      simulatorP1Model,
      heuristicsModelA
    } = this.state;

    return (
      <PageContainer>
        {/* Navigation Header */}
        <HeaderNav>
          <TitleGroup>
            <Typography variant="h5" style={{ fontWeight: 800, color: "#1E293B" }}>
              Лаборатория ИИ
            </Typography>
            <Typography variant="body2" color="textSecondary">
              Тестирование и анализ алгоритмов
            </Typography>
          </TitleGroup>

          <div style={{ display: "flex", gap: 10 }}>
            <Button
              variant="outlined"
              color="primary"
              component={Link}
              to="/ai"
              style={{ fontWeight: 600 }}
            >
              <IconArrowBack style={{ marginRight: 6, fontSize: 18 }} />
              Назад к игре
            </Button>
          </div>
        </HeaderNav>

        {/* Tab Navigation */}
        <TabsWrapper>
          <Tabs
            value={activeTab}
            onChange={this.handleTabChange}
            indicatorColor="secondary"
            textColor="secondary"
            scrollable
            scrollButtons="auto"
          >
            <Tab
              label="Арена"
              icon={<IconCompare style={{ fontSize: 20 }} />}
              style={{ fontWeight: 600 }}
            />
            <Tab
              label="Симулятор"
              icon={<IconPlay style={{ fontSize: 20 }} />}
              style={{ fontWeight: 600 }}
            />
            <Tab
              label="Эвристики"
              icon={<IconTune style={{ fontSize: 20 }} />}
              style={{ fontWeight: 600 }}
            />
            <Tab
              label="Карта поля"
              icon={<IconGrid style={{ fontSize: 20 }} />}
              style={{ fontWeight: 600 }}
            />
            <Tab
              label="Паспорта моделей"
              icon={<IconAssessment style={{ fontSize: 20 }} />}
              style={{ fontWeight: 600 }}
            />
          </Tabs>
        </TabsWrapper>

        {/* Tab Panels */}
        <TabContent>
          {activeTab === 0 && (
            <ArenaTab
              initialP1Type={arenaP1Model}
              onSelectReplayMatch={this.handleSelectReplayMatch}
              onViewPassport={this.handleViewPassport}
            />
          )}

          {activeTab === 1 && (
            <StepSimulatorTab
              initialP1Type={simulatorP1Model}
              initialMatch={replayMatch}
              onViewPassport={this.handleViewPassport}
            />
          )}

          {activeTab === 2 && (
            <HeuristicsTab
              initialModelA={heuristicsModelA}
              onViewPassport={this.handleViewPassport}
            />
          )}

          {activeTab === 3 && <SpatialMatrixTab />}

          {activeTab === 4 && (
            <PassportTab
              selectedModelId={selectedPassportModelId}
              onSelectModel={id => this.setState({ selectedPassportModelId: id })}
              onNavigateToArena={this.handleNavigateToArena}
              onNavigateToSimulator={this.handleNavigateToSimulator}
              onNavigateToHeuristics={this.handleNavigateToHeuristics}
            />
          )}
        </TabContent>
      </PageContainer>
    );
  }
}
