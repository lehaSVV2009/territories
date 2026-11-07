import React, { Component } from "react";
import styled from "styled-components";
import Typography from "@material-ui/core/Typography";
import Paper from "../libs/territories-ui/Paper";
import Button from "../libs/territories-ui/Button";
import Chip from "../libs/territories-ui/Chip";
import Select from "../libs/territories-ui/Select";
import MenuItem from "../libs/territories-ui/MenuItem";
import IconButton from "../libs/territories-ui/IconButton";

import IconPlay from "@material-ui/icons/PlayArrow";
import IconPause from "@material-ui/icons/Pause";
import IconSkipPrev from "@material-ui/icons/ChevronLeft";
import IconSkipNext from "@material-ui/icons/ChevronRight";
import IconRefresh from "@material-ui/icons/Refresh";
import IconFirstPage from "@material-ui/icons/FastRewind";
import IconLastPage from "@material-ui/icons/FastForward";

import { MODELS_CATALOG, BOT_TYPES } from "../AI";
import {
  simulateSingleMatch,
  BOARD_ROWS,
  BOARD_COLS
} from "./simulatorEngine";
import { CELL_TYPE } from "territories-core";

const Container = styled.div`
  display: flex;
  flex-direction: column;
  gap: 16px;
`;

const TopBar = styled(Paper)`
  padding: 16px 20px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  flex-wrap: wrap;
  gap: 16px;
`;

const ModelPairGroup = styled.div`
  display: flex;
  align-items: center;
  gap: 12px;
  flex-wrap: wrap;
`;

const BoardCard = styled(Paper)`
  padding: 16px;
  background-color: #ffffff;
  display: flex;
  flex-direction: column;
  align-items: center;
`;

const BoardSvg = styled.svg`
  width: 100%;
  max-width: 960px;
  height: auto;
  border-radius: 6px;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.08);
  background-color: #f1f5f9;
`;

const ControlsBar = styled(Paper)`
  padding: 14px 20px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  flex-wrap: wrap;
  gap: 16px;
  background-color: #ffffff;
`;

const PlaybackGroup = styled.div`
  display: flex;
  align-items: center;
  gap: 8px;
`;

const ScoreGrid = styled.div`
  display: grid;
  grid-template-columns: 1fr 1fr 1fr;
  gap: 12px;
  width: 100%;
  max-width: 960px;
  margin-top: 12px;
`;

const ScoreBox = styled.div`
  padding: 10px 14px;
  border-radius: 6px;
  background-color: ${props => props.bgColor || "#f8fafc"};
  border-left: 4px solid ${props => props.borderColor || "#cbd5e1"};
  text-align: center;
`;

const StepSlider = styled.input`
  width: 100%;
  max-width: 960px;
  margin: 8px 0;
  cursor: pointer;
`;

const SPEED_OPTIONS = [
  { label: "0.5x", delay: 800 },
  { label: "1x", delay: 400 },
  { label: "2x", delay: 180 },
  { label: "5x", delay: 60 }
];

export default class StepSimulatorTab extends Component {
  state = {
    p1Type: this.props.initialP1Type || BOT_TYPES.APPEX,
    p2Type: BOT_TYPES.G40,
    currentStepIndex: 0,
    isPlaying: false,
    speedIndex: 1,
    matchData: null
  };

  playTimer = null;

  componentDidMount() {
    if (this.props.initialMatch) {
      this.loadMatch(this.props.initialMatch);
    } else {
      this.handleGenerateNewMatch();
    }
  }

  componentDidUpdate(prevProps) {
    if (
      this.props.initialMatch &&
      this.props.initialMatch !== prevProps.initialMatch
    ) {
      this.loadMatch(this.props.initialMatch);
    } else if (
      this.props.initialP1Type &&
      this.props.initialP1Type !== prevProps.initialP1Type
    ) {
      this.setState({ p1Type: this.props.initialP1Type }, () => {
        this.handleGenerateNewMatch();
      });
    }
  }

  componentWillUnmount() {
    this.stopPlayback();
  }

  loadMatch = match => {
    this.stopPlayback();
    this.setState({
      matchData: match,
      currentStepIndex: match.history && match.history.length > 0 ? 0 : 0,
      p1Type: match.p1Type,
      p2Type: match.p2Type
    });
  };

  handleGenerateNewMatch = () => {
    this.stopPlayback();
    const { p1Type, p2Type } = this.state;
    const seed = Math.floor(Math.random() * 1000000);
    const match = simulateSingleMatch({
      p1Type,
      p2Type,
      seed,
      recordHistory: true
    });
    this.setState({
      matchData: match,
      currentStepIndex: 0
    });
  };

  startPlayback = () => {
    this.setState({ isPlaying: true });
    this.scheduleNextStep();
  };

  stopPlayback = () => {
    if (this.playTimer) {
      clearTimeout(this.playTimer);
      this.playTimer = null;
    }
    this.setState({ isPlaying: false });
  };

  scheduleNextStep = () => {
    const { speedIndex } = this.state;
    const delay = SPEED_OPTIONS[speedIndex].delay;

    this.playTimer = setTimeout(() => {
      const { matchData, currentStepIndex } = this.state;
      if (!matchData || !matchData.history) {
        this.stopPlayback();
        return;
      }

      if (currentStepIndex < matchData.history.length - 1) {
        this.setState(
          { currentStepIndex: currentStepIndex + 1 },
          () => this.scheduleNextStep()
        );
      } else {
        this.stopPlayback();
      }
    }, delay);
  };

  handleTogglePlay = () => {
    if (this.state.isPlaying) {
      this.stopPlayback();
    } else {
      const { matchData, currentStepIndex } = this.state;
      if (
        matchData &&
        matchData.history &&
        currentStepIndex >= matchData.history.length - 1
      ) {
        this.setState({ currentStepIndex: 0 }, () => this.startPlayback());
      } else {
        this.startPlayback();
      }
    }
  };

  handleStepForward = () => {
    this.stopPlayback();
    const { matchData, currentStepIndex } = this.state;
    if (matchData && currentStepIndex < matchData.history.length - 1) {
      this.setState({ currentStepIndex: currentStepIndex + 1 });
    }
  };

  handleStepBackward = () => {
    this.stopPlayback();
    const { currentStepIndex } = this.state;
    if (currentStepIndex > 0) {
      this.setState({ currentStepIndex: currentStepIndex - 1 });
    }
  };

  handleFirstStep = () => {
    this.stopPlayback();
    this.setState({ currentStepIndex: 0 });
  };

  handleLastStep = () => {
    this.stopPlayback();
    const { matchData } = this.state;
    if (matchData && matchData.history) {
      this.setState({ currentStepIndex: matchData.history.length - 1 });
    }
  };

  handleSliderChange = e => {
    this.stopPlayback();
    this.setState({ currentStepIndex: parseInt(e.target.value, 10) });
  };

  handleSpeedChange = index => {
    this.setState({ speedIndex: index });
  };

  render() {
    const {
      p1Type,
      p2Type,
      currentStepIndex,
      isPlaying,
      speedIndex,
      matchData
    } = this.state;

    const history = matchData ? matchData.history : [];
    const currentStep =
      history.length > 0 && history[currentStepIndex]
        ? history[currentStepIndex]
        : null;

    const board = currentStep ? currentStep.board : null;
    const scores = currentStep
      ? currentStep.scores
      : { p1: 0, p2: 0, empty: 600 };

    const p1Meta =
      MODELS_CATALOG.find(m => m.id === p1Type) || MODELS_CATALOG[0];
    const p2Meta =
      MODELS_CATALOG.find(m => m.id === p2Type) || MODELS_CATALOG[1];

    // SVG scaling constants: 40 cols, 15 rows
    const cellW = 20;
    const cellH = 20;
    const svgW = BOARD_COLS * cellW;
    const svgH = BOARD_ROWS * cellH;

    const moveRect = currentStep ? currentStep.move : null;

    return (
      <Container>
        {/* Match Setup Bar */}
        <TopBar>
          <ModelPairGroup>
            <Typography variant="subtitle2" style={{ fontWeight: 700, color: "#375E97" }}>
              P1:
            </Typography>
            <Select
              value={p1Type}
              onChange={e => this.setState({ p1Type: e.target.value })}
              disabled={isPlaying}
            >
              {MODELS_CATALOG.map(m => (
                <MenuItem key={m.id} value={m.id}>
                  {m.name}
                </MenuItem>
              ))}
            </Select>

            <Typography variant="subtitle2" style={{ fontWeight: 700, color: "#FB6542" }}>
              P2:
            </Typography>
            <Select
              value={p2Type}
              onChange={e => this.setState({ p2Type: e.target.value })}
              disabled={isPlaying}
            >
              {MODELS_CATALOG.map(m => (
                <MenuItem key={m.id} value={m.id}>
                  {m.name}
                </MenuItem>
              ))}
            </Select>

            <Button
              variant="outlined"
              size="small"
              color="primary"
              onClick={this.handleGenerateNewMatch}
              style={{ fontWeight: 600 }}
            >
              <IconRefresh style={{ marginRight: 6, fontSize: 18 }} />
              Новая партия
            </Button>
          </ModelPairGroup>

          {matchData && (
            <Chip
              label={`Сид: ${matchData.seed}`}
              variant="outlined"
              size="small"
              style={{ fontFamily: "monospace", fontWeight: 600 }}
            />
          )}
        </TopBar>

        {/* Board View */}
        <BoardCard>
          <BoardSvg viewBox={`0 0 ${svgW} ${svgH}`}>
            {board &&
              board.map((row, r) =>
                row.map((cell, c) => {
                  let fill = "#f8fafc";
                  if (cell === CELL_TYPE.OCCUPIED_BY_PLAYER_1) {
                    fill = "#375E97";
                  } else if (cell === CELL_TYPE.OCCUPIED_BY_PLAYER_2) {
                    fill = "#FB6542";
                  }
                  return (
                    <rect
                      key={`${r}-${c}`}
                      x={c * cellW}
                      y={r * cellH}
                      width={cellW}
                      height={cellH}
                      fill={fill}
                      stroke="#cbd5e1"
                      strokeWidth="0.5"
                    />
                  );
                })
              )}

            {/* Highlight current placed rectangle */}
            {moveRect && (
              <rect
                x={moveRect.columnIndex * cellW}
                y={moveRect.rowIndex * cellH}
                width={moveRect.rectangleWidth * cellW}
                height={moveRect.rectangleHeight * cellH}
                fill="none"
                stroke="#F1C40F"
                strokeWidth="2.5"
                strokeDasharray="4,2"
              />
            )}
          </BoardSvg>

          {/* Turn Slider */}
          {history.length > 1 && (
            <div style={{ width: "100%", maxWidth: 960, marginTop: 12 }}>
              <StepSlider
                type="range"
                min={0}
                max={history.length - 1}
                value={currentStepIndex}
                onChange={this.handleSliderChange}
              />
            </div>
          )}

          {/* Scores Overview */}
          <ScoreGrid>
            <ScoreBox bgColor="#ebf3fb" borderColor="#375E97">
              <Typography variant="caption" style={{ color: "#375E97", fontWeight: 700 }}>
                {p1Meta.name} (P1)
              </Typography>
              <Typography variant="h6" style={{ fontWeight: 800, color: "#375E97" }}>
                {scores.p1} ({Math.round((scores.p1 / 600) * 100)}%)
              </Typography>
            </ScoreBox>

            <ScoreBox bgColor="#f8fafc" borderColor="#64748B">
              <Typography variant="caption" style={{ color: "#64748B", fontWeight: 700 }}>
                Свободно
              </Typography>
              <Typography variant="h6" style={{ fontWeight: 800, color: "#475569" }}>
                {scores.empty}
              </Typography>
            </ScoreBox>

            <ScoreBox bgColor="#fff3eb" borderColor="#FB6542">
              <Typography variant="caption" style={{ color: "#FB6542", fontWeight: 700 }}>
                {p2Meta.name} (P2)
              </Typography>
              <Typography variant="h6" style={{ fontWeight: 800, color: "#FB6542" }}>
                {scores.p2} ({Math.round((scores.p2 / 600) * 100)}%)
              </Typography>
            </ScoreBox>
          </ScoreGrid>
        </BoardCard>

        {/* Playback Controls & Commentary */}
        <ControlsBar>
          <PlaybackGroup>
            <IconButton onClick={this.handleFirstStep} disabled={currentStepIndex === 0}>
              <IconFirstPage />
            </IconButton>
            <IconButton onClick={this.handleStepBackward} disabled={currentStepIndex === 0}>
              <IconSkipPrev />
            </IconButton>
            <Button
              variant="contained"
              color={isPlaying ? "default" : "secondary"}
              onClick={this.handleTogglePlay}
              style={{ fontWeight: 600, minWidth: 110 }}
            >
              {isPlaying ? (
                <>
                  <IconPause style={{ marginRight: 6 }} /> Пауза
                </>
              ) : (
                <>
                  <IconPlay style={{ marginRight: 6 }} /> Старт
                </>
              )}
            </Button>
            <IconButton
              onClick={this.handleStepForward}
              disabled={!history.length || currentStepIndex >= history.length - 1}
            >
              <IconSkipNext />
            </IconButton>
            <IconButton
              onClick={this.handleLastStep}
              disabled={!history.length || currentStepIndex >= history.length - 1}
            >
              <IconLastPage />
            </IconButton>

            <Typography variant="body2" style={{ fontWeight: 700, marginLeft: 8 }}>
              Ход {currentStepIndex} из {Math.max(0, history.length - 1)}
            </Typography>
          </PlaybackGroup>

          {/* Speed Selector */}
          <div style={{ display: "flex", alignItems: "center", gap: 6 }}>
            <Typography variant="caption" style={{ fontWeight: 600, color: "#64748B" }}>
              Скорость:
            </Typography>
            {SPEED_OPTIONS.map((opt, idx) => (
              <Button
                key={opt.label}
                size="small"
                variant={speedIndex === idx ? "contained" : "outlined"}
                color={speedIndex === idx ? "primary" : "default"}
                onClick={() => this.handleSpeedChange(idx)}
                style={{ minWidth: 38, padding: "2px 6px", fontSize: 12, fontWeight: 600 }}
              >
                {opt.label}
              </Button>
            ))}
          </div>
        </ControlsBar>

        {/* Step Commentary Box */}
        {currentStep && (
          <Paper style={{ padding: 14, backgroundColor: "#f8fafc" }}>
            <div style={{ display: "flex", alignItems: "center", gap: 10 }}>
              <Chip
                label={currentStep.turn === 0 ? "Старт" : `Ход #${currentStep.turn}`}
                size="small"
                style={{ fontWeight: 700 }}
              />
              {currentStep.dices[0] > 0 && (
                <Chip
                  label={`Кубики: ${currentStep.dices[0]} x ${currentStep.dices[1]}`}
                  size="small"
                  variant="outlined"
                  style={{ fontWeight: 600 }}
                />
              )}
              <Typography variant="body2" style={{ fontWeight: 500, color: "#1E293B" }}>
                {currentStep.commentary}
              </Typography>
            </div>
          </Paper>
        )}
      </Container>
    );
  }
}
