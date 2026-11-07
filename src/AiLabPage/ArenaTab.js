import React, { Component } from "react";
import styled from "styled-components";
import Typography from "@material-ui/core/Typography";
import Paper from "../libs/territories-ui/Paper";
import Button from "../libs/territories-ui/Button";
import Chip from "../libs/territories-ui/Chip";
import Select from "../libs/territories-ui/Select";
import MenuItem from "../libs/territories-ui/MenuItem";
import LinearProgress from "../libs/territories-ui/LinearProgress";
import Table from "@material-ui/core/Table";
import TableBody from "@material-ui/core/TableBody";
import TableCell from "@material-ui/core/TableCell";
import TableHead from "@material-ui/core/TableHead";
import TableRow from "@material-ui/core/TableRow";

import IconPlay from "@material-ui/icons/PlayArrow";
import IconStop from "@material-ui/icons/Stop";
import IconEye from "@material-ui/icons/Visibility";

import { MODELS_CATALOG, BOT_TYPES } from "../AI";
import { runTournamentBatch, simulateSingleMatch } from "./simulatorEngine";

const ArenaContainer = styled.div`
  display: flex;
  flex-direction: column;
  gap: 20px;
`;

const SelectorsGrid = styled.div`
  display: grid;
  grid-template-columns: 1fr auto 1fr;
  align-items: center;
  gap: 16px;

  @media (max-width: 768px) {
    grid-template-columns: 1fr;
  }
`;

const FighterCard = styled(Paper)`
  padding: 18px;
  border-top: 4px solid ${props => props.accentColor || "#375e97"};
  background-color: #ffffff;
  display: flex;
  flex-direction: column;
  gap: 12px;
`;

const VersusBadge = styled.div`
  display: flex;
  align-items: center;
  justify-content: center;
  width: 48px;
  height: 48px;
  border-radius: 50%;
  background-color: #2c3e50;
  color: #ffffff;
  font-weight: 800;
  font-size: 14px;
  margin: 0 auto;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.15);
`;

const ControlsBar = styled(Paper)`
  padding: 16px 20px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  flex-wrap: wrap;
  gap: 16px;
  background-color: #ffffff;
`;

const BatchButtons = styled.div`
  display: flex;
  align-items: center;
  gap: 8px;
`;

const WinrateBarContainer = styled.div`
  display: flex;
  height: 32px;
  border-radius: 6px;
  overflow: hidden;
  box-shadow: inset 0 2px 4px rgba(0, 0, 0, 0.1);
  margin: 8px 0;
  background-color: #e2e8f0;
`;

const WinrateSegment = styled.div`
  width: ${props => props.width || 0}%;
  background-color: ${props => props.color};
  display: flex;
  align-items: center;
  justify-content: center;
  color: #ffffff;
  font-weight: 700;
  font-size: 13px;
  transition: width 0.3s ease;
  overflow: hidden;
  white-space: nowrap;
`;

const StatsGrid = styled.div`
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
  gap: 14px;
`;

const StatCard = styled(Paper)`
  padding: 16px;
  text-align: center;
  background-color: #ffffff;
  border-left: 4px solid ${props => props.color || "#cbd5e1"};
`;

const StatValue = styled.div`
  font-size: 24px;
  font-weight: 800;
  color: ${props => props.color || "#1e293b"};
  margin-top: 4px;
`;

export default class ArenaTab extends Component {
  state = {
    p1Type: this.props.initialP1Type || BOT_TYPES.APPEX,
    p2Type: BOT_TYPES.G40,
    batchSize: 10,
    isRunning: false,
    progress: null,
    tournamentResults: null,
    matchesHistory: []
  };

  componentDidUpdate(prevProps) {
    if (
      this.props.initialP1Type &&
      this.props.initialP1Type !== prevProps.initialP1Type
    ) {
      this.setState({ p1Type: this.props.initialP1Type });
    }
  }

  stopRequested = false;

  handleP1Change = e => {
    this.setState({ p1Type: e.target.value });
  };

  handleP2Change = e => {
    this.setState({ p2Type: e.target.value });
  };

  handleBatchSizeChange = size => {
    this.setState({ batchSize: size });
  };

  handleStartTournament = async () => {
    const { p1Type, p2Type, batchSize } = this.state;
    this.stopRequested = false;
    this.setState({
      isRunning: true,
      progress: { completed: 0, total: batchSize, percentage: 0 },
      matchesHistory: []
    });

    const results = await runTournamentBatch({
      p1Type,
      p2Type,
      matchCount: batchSize,
      startSeed: Math.floor(Math.random() * 500000) + 1000,
      onProgress: progress => {
        this.setState({
          progress,
          matchesHistory: progress.currentMatch
            ? [progress.currentMatch, ...this.state.matchesHistory]
            : this.state.matchesHistory
        });
      },
      shouldStop: () => this.stopRequested
    });

    this.setState({
      isRunning: false,
      tournamentResults: results
    });
  };

  handleStopTournament = () => {
    this.stopRequested = true;
    this.setState({ isRunning: false });
  };

  handleReplayClick = match => {
    // Generate detailed replay with history if needed
    const detailedMatch = simulateSingleMatch({
      p1Type: match.p1Type,
      p2Type: match.p2Type,
      seed: match.seed,
      recordHistory: true
    });
    this.props.onSelectReplayMatch(detailedMatch);
  };

  render() {
    const {
      p1Type,
      p2Type,
      batchSize,
      isRunning,
      progress,
      tournamentResults,
      matchesHistory
    } = this.state;

    const p1Meta =
      MODELS_CATALOG.find(m => m.id === p1Type) || MODELS_CATALOG[0];
    const p2Meta =
      MODELS_CATALOG.find(m => m.id === p2Type) || MODELS_CATALOG[1];

    const currentResults = tournamentResults || progress;

    const p1Width = currentResults ? currentResults.p1Winrate : 50;
    const p2Width = currentResults ? currentResults.p2Winrate : 50;
    const drawWidth = currentResults ? currentResults.drawRate : 0;

    return (
      <ArenaContainer>
        {/* Model Selectors */}
        <SelectorsGrid>
          <FighterCard accentColor="#375E97">
            <Typography variant="caption" style={{ color: "#375E97", fontWeight: 700 }}>
              Игрок 1 (P1)
            </Typography>
            <Select
              value={p1Type}
              onChange={this.handleP1Change}
              disabled={isRunning}
            >
              {MODELS_CATALOG.map(model => (
                <MenuItem key={model.id} value={model.id}>
                  {model.title}
                </MenuItem>
              ))}
            </Select>
            <div style={{ display: "flex", gap: 8, alignItems: "center" }}>
              <Chip
                label={`Ранг: ${p1Meta.tier}`}
                size="small"
                style={{ backgroundColor: "#375E97", color: "#fff", fontWeight: 600 }}
              />
              <Chip label={p1Meta.badge} size="small" variant="outlined" />
              {this.props.onViewPassport && (
                <Button
                  size="small"
                  variant="outlined"
                  onClick={() => this.props.onViewPassport(p1Type)}
                  style={{ marginLeft: "auto", fontSize: 11, padding: "2px 8px", fontWeight: 700 }}
                >
                  Паспорт
                </Button>
              )}
            </div>
            <Typography variant="body2" color="textSecondary">
              {p1Meta.description}
            </Typography>
          </FighterCard>

          <VersusBadge>VS</VersusBadge>

          <FighterCard accentColor="#FB6542">
            <Typography variant="caption" style={{ color: "#FB6542", fontWeight: 700 }}>
              Игрок 2 (P2)
            </Typography>
            <Select
              value={p2Type}
              onChange={this.handleP2Change}
              disabled={isRunning}
            >
              {MODELS_CATALOG.map(model => (
                <MenuItem key={model.id} value={model.id}>
                  {model.title}
                </MenuItem>
              ))}
            </Select>
            <div style={{ display: "flex", gap: 8, alignItems: "center" }}>
              <Chip
                label={`Ранг: ${p2Meta.tier}`}
                size="small"
                style={{ backgroundColor: "#FB6542", color: "#fff", fontWeight: 600 }}
              />
              <Chip label={p2Meta.badge} size="small" variant="outlined" />
              {this.props.onViewPassport && (
                <Button
                  size="small"
                  variant="outlined"
                  onClick={() => this.props.onViewPassport(p2Type)}
                  style={{ marginLeft: "auto", fontSize: 11, padding: "2px 8px", fontWeight: 700 }}
                >
                  Паспорт
                </Button>
              )}
            </div>
            <Typography variant="body2" color="textSecondary">
              {p2Meta.description}
            </Typography>
          </FighterCard>
        </SelectorsGrid>

        {/* Controls & Batch Selection */}
        <ControlsBar>
          <div style={{ display: "flex", alignItems: "center", gap: 12 }}>
            <Typography variant="subtitle2" style={{ fontWeight: 600, color: "#475569" }}>
              Серия партий:
            </Typography>
            <BatchButtons>
              {[1, 5, 10, 25, 50].map(size => (
                <Button
                  key={size}
                  variant={batchSize === size ? "contained" : "outlined"}
                  color={batchSize === size ? "primary" : "default"}
                  size="small"
                  disabled={isRunning}
                  onClick={() => this.handleBatchSizeChange(size)}
                  style={{ minWidth: 42, fontWeight: 600 }}
                >
                  {size}
                </Button>
              ))}
            </BatchButtons>
          </div>

          <div>
            {isRunning ? (
              <Button
                variant="contained"
                style={{ backgroundColor: "#e11d48", color: "#fff", fontWeight: 600 }}
                onClick={this.handleStopTournament}
              >
                <IconStop style={{ marginRight: 6 }} />
                Остановить
              </Button>
            ) : (
              <Button
                variant="contained"
                color="secondary"
                style={{ fontWeight: 600 }}
                onClick={this.handleStartTournament}
              >
                <IconPlay style={{ marginRight: 6 }} />
                Запустить турнир ({batchSize} партий)
              </Button>
            )}
          </div>
        </ControlsBar>

        {/* Progress Bar */}
        {isRunning && progress && (
          <Paper style={{ padding: 16 }}>
            <div style={{ display: "flex", justifyContent: "space-between", marginBottom: 8 }}>
              <Typography variant="body2" style={{ fontWeight: 600 }}>
                Симуляция турнира: {progress.completed} из {progress.total}
              </Typography>
              <Typography variant="body2" style={{ fontWeight: 600, color: "#375E97" }}>
                {progress.percentage}%
              </Typography>
            </div>
            <LinearProgress variant="determinate" value={progress.percentage} color="secondary" />
          </Paper>
        )}

        {/* Winrate Bar & Stats */}
        {currentResults && (
          <Paper style={{ padding: 20 }}>
            <Typography variant="h6" style={{ fontWeight: 700, marginBottom: 12 }}>
              Результаты
            </Typography>

            <WinrateBarContainer>
              {p1Width > 0 && (
                <WinrateSegment width={p1Width} color="#375E97">
                  {p1Meta.name}: {p1Width}%
                </WinrateSegment>
              )}
              {drawWidth > 0 && (
                <WinrateSegment width={drawWidth} color="#94A3B8">
                  Ничьи: {drawWidth}%
                </WinrateSegment>
              )}
              {p2Width > 0 && (
                <WinrateSegment width={p2Width} color="#FB6542">
                  {p2Meta.name}: {p2Width}%
                </WinrateSegment>
              )}
            </WinrateBarContainer>

            <StatsGrid style={{ marginTop: 16 }}>
              <StatCard color="#375E97">
                <Typography variant="caption" color="textSecondary">
                  ПОБЕД P1
                </Typography>
                <StatValue color="#375E97">
                  {currentResults.p1Wins} ({currentResults.p1Winrate}%)
                </StatValue>
              </StatCard>

              <StatCard color="#FB6542">
                <Typography variant="caption" color="textSecondary">
                  ПОБЕД P2
                </Typography>
                <StatValue color="#FB6542">
                  {currentResults.p2Wins} ({currentResults.p2Winrate}%)
                </StatValue>
              </StatCard>

              <StatCard color="#2C3E50">
                <Typography variant="caption" color="textSecondary">
                  РАЗНИЦА КЛЕТОК
                </Typography>
                <StatValue color={currentResults.avgMargin >= 0 ? "#375E97" : "#FB6542"}>
                  {currentResults.avgMargin >= 0
                    ? `+${currentResults.avgMargin}`
                    : currentResults.avgMargin}
                </StatValue>
              </StatCard>

              <StatCard color="#64748B">
                <Typography variant="caption" color="textSecondary">
                  ЗАХВАТ (P1 / P2)
                </Typography>
                <StatValue color="#1E293B">
                  {currentResults.avgP1Cells} / {currentResults.avgP2Cells}
                </StatValue>
              </StatCard>
            </StatsGrid>
          </Paper>
        )}

        {/* History Table */}
        {matchesHistory.length > 0 && (
          <Paper style={{ padding: 20 }}>
            <Typography variant="h6" style={{ fontWeight: 700, marginBottom: 12 }}>
              История партий ({matchesHistory.length})
            </Typography>
            <div style={{ overflowX: "auto" }}>
              <Table>
                <TableHead>
                  <TableRow>
                    <TableCell>Партия</TableCell>
                    <TableCell>Сид</TableCell>
                    <TableCell>Победитель</TableCell>
                    <TableCell>Счет (P1 : P2)</TableCell>
                    <TableCell>Разница</TableCell>
                    <TableCell>Ходов</TableCell>
                    <TableCell align="right">Пошаговый просмотр</TableCell>
                  </TableRow>
                </TableHead>
                <TableBody>
                  {matchesHistory.slice(0, 20).map((match, idx) => {
                    const isP1Win = match.winner === "0";
                    const isP2Win = match.winner === "1";
                    const winnerName = isP1Win
                      ? p1Meta.name
                      : isP2Win
                      ? p2Meta.name
                      : "Ничья";
                    const winnerColor = isP1Win
                      ? "#375E97"
                      : isP2Win
                      ? "#FB6542"
                      : "#64748B";

                    return (
                      <TableRow key={idx} hover>
                        <TableCell>#{matchesHistory.length - idx}</TableCell>
                        <TableCell style={{ fontFamily: "monospace" }}>
                          {match.seed}
                        </TableCell>
                        <TableCell>
                          <Chip
                            label={winnerName}
                            size="small"
                            style={{
                              backgroundColor: winnerColor,
                              color: "#fff",
                              fontWeight: 600
                            }}
                          />
                        </TableCell>
                        <TableCell style={{ fontWeight: 700 }}>
                          {match.p1Score} : {match.p2Score}
                        </TableCell>
                        <TableCell
                          style={{
                            fontWeight: 700,
                            color: match.margin >= 0 ? "#375E97" : "#FB6542"
                          }}
                        >
                          {match.margin >= 0 ? `+${match.margin}` : match.margin}
                        </TableCell>
                        <TableCell>{match.totalTurns}</TableCell>
                        <TableCell align="right">
                          <Button
                            variant="outlined"
                            size="small"
                            color="primary"
                            onClick={() => this.handleReplayClick(match)}
                            style={{ fontWeight: 600 }}
                          >
                            <IconEye style={{ marginRight: 4, fontSize: 16 }} />
                            Воспроизвести
                          </Button>
                        </TableCell>
                      </TableRow>
                    );
                  })}
                </TableBody>
              </Table>
            </div>
          </Paper>
        )}
      </ArenaContainer>
    );
  }
}
