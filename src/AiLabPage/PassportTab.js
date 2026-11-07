import React, { Component } from "react";
import styled from "styled-components";
import { Link } from "react-router-dom";
import Typography from "@material-ui/core/Typography";
import Paper from "../libs/territories-ui/Paper";
import Chip from "../libs/territories-ui/Chip";
import Button from "../libs/territories-ui/Button";
import Select from "../libs/territories-ui/Select";
import MenuItem from "../libs/territories-ui/MenuItem";
import Table from "@material-ui/core/Table";
import TableBody from "@material-ui/core/TableBody";
import TableCell from "@material-ui/core/TableCell";
import TableHead from "@material-ui/core/TableHead";
import TableRow from "@material-ui/core/TableRow";

import IconAssessment from "@material-ui/icons/Assessment";
import IconSecurity from "@material-ui/icons/Security";
import IconTimeline from "@material-ui/icons/Timeline";
import IconStorage from "@material-ui/icons/Storage";
import IconCompare from "@material-ui/icons/CompareArrows";
import IconPlay from "@material-ui/icons/PlayArrow";
import IconTune from "@material-ui/icons/Tune";
import IconGames from "@material-ui/icons/Games";

import { MODELS_CATALOG, BOT_TYPES } from "../AI";
import { PASSPORTS_DATA } from "./passportData";
import weightsData from "../AI/weights.json";

const Container = styled.div`
  display: flex;
  flex-direction: column;
  gap: 20px;
`;

const ModelSelectorCard = styled(Paper)`
  padding: 18px 20px;
  background-color: #ffffff;
`;

const ModelCardsGrid = styled.div`
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(130px, 1fr));
  gap: 10px;
  margin-top: 12px;

  @media (max-width: 600px) {
    display: none;
  }
`;

const ModelSelectMiniCard = styled.div`
  padding: 10px 12px;
  border-radius: 8px;
  cursor: pointer;
  background-color: ${props => (props.selected ? "#ffffff" : "#f8fafc")};
  border: 2px solid ${props => (props.selected ? props.color : "#e2e8f0")};
  box-shadow: ${props =>
    props.selected ? `0 4px 12px ${props.color}25` : "none"};
  transition: all 0.2s ease;
  display: flex;
  flex-direction: column;
  gap: 4px;

  &:hover {
    border-color: ${props => props.color};
    background-color: #ffffff;
    transform: translateY(-1px);
  }
`;

const MobileSelectWrapper = styled.div`
  display: none;
  margin-top: 12px;

  @media (max-width: 600px) {
    display: block;
  }
`;

const PassportHeader = styled(Paper)`
  padding: 24px;
  background-color: #ffffff;
  border-top: 5px solid ${props => props.accentColor || "#fb6542"};
`;

const BadgesRow = styled.div`
  display: flex;
  align-items: center;
  gap: 8px;
  flex-wrap: wrap;
  margin-top: 14px;
`;

const ActionButtonsBar = styled(Paper)`
  padding: 14px 20px;
  background-color: #ffffff;
  display: flex;
  align-items: center;
  justify-content: space-between;
  flex-wrap: wrap;
  gap: 12px;
`;

const MetricsGrid = styled.div`
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(220px, 1fr));
  gap: 16px;
`;

const MetricCard = styled(Paper)`
  padding: 18px;
  background-color: #ffffff;
  border-left: 4px solid ${props => props.color || "#375e97"};
  display: flex;
  flex-direction: column;
  gap: 6px;
`;

const MetricValue = styled.div`
  font-size: 28px;
  font-weight: 800;
  color: ${props => props.color || "#1e293b"};
`;

const ArchitectureGrid = styled.div`
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(260px, 1fr));
  gap: 16px;
`;

const ArchCard = styled(Paper)`
  padding: 20px;
  background-color: #ffffff;
  display: flex;
  flex-direction: column;
  gap: 10px;
`;

const ArchHeader = styled.div`
  display: flex;
  align-items: center;
  gap: 10px;
  font-weight: 700;
  font-size: 15px;
  color: #1e293b;
`;

const DeepDiveCard = styled(Paper)`
  padding: 24px;
  background-color: #ffffff;
  border: 1px solid #e2e8f0;
`;

const MathFormulaBox = styled.div`
  background-color: #0f172a;
  color: #f8fafc;
  padding: 16px 20px;
  border-radius: 8px;
  font-family: "SFMono-Regular", Consolas, "Liberation Mono", Menlo, monospace;
  font-size: 14px;
  line-height: 1.6;
  margin: 12px 0;
  overflow-x: auto;
  border-left: 4px solid #9b59b6;
`;

const DossierGrid = styled.div`
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(240px, 1fr));
  gap: 12px;
  margin-top: 14px;
`;

const DossierItem = styled.div`
  padding: 12px 14px;
  background-color: #f8fafc;
  border-radius: 6px;
  border: 1px solid #e2e8f0;
  display: flex;
  flex-direction: column;
  gap: 4px;
`;

export default class PassportTab extends Component {
  state = {
    selectedModelId: this.props.selectedModelId || BOT_TYPES.APPEX
  };

  componentDidUpdate(prevProps) {
    if (
      this.props.selectedModelId &&
      this.props.selectedModelId !== prevProps.selectedModelId
    ) {
      this.setState({ selectedModelId: this.props.selectedModelId });
    }
  }

  handleModelChange = modelId => {
    this.setState({ selectedModelId: modelId });
    if (this.props.onSelectModel) {
      this.props.onSelectModel(modelId);
    }
  };

  renderArchIcon(iconName, color) {
    const style = { color: color || "#375E97", fontSize: 22 };
    switch (iconName) {
      case "Timeline":
        return <IconTimeline style={style} />;
      case "Storage":
        return <IconStorage style={style} />;
      case "Assessment":
        return <IconAssessment style={style} />;
      case "Security":
        return <IconSecurity style={style} />;
      case "Tune":
        return <IconTune style={style} />;
      default:
        return <IconAssessment style={style} />;
    }
  }

  renderAppexDeepDive() {
    return (
      <DeepDiveCard>
        <div style={{ display: "flex", alignItems: "center", gap: 10, marginBottom: 8 }}>
          <IconTune style={{ color: "#9B59B6", fontSize: 24 }} />
          <Typography variant="h6" style={{ fontWeight: 800, color: "#1E293B" }}>
            Архитектура Appex
          </Typography>
        </div>
        <Typography variant="body2" color="textSecondary" style={{ marginBottom: 14 }}>
          Модель динамически интерполирует веса под стиль противника, сохраняя базовые веса G40 (не менее 35%).
        </Typography>

        {/* Math Blending Formula */}
        <Typography variant="subtitle2" style={{ fontWeight: 700, color: "#334155" }}>
          Формула смешивания весов:
        </Typography>
        <MathFormulaBox>
          W_active = (1 - α) · W_G40 + α · ∑ (p_k · W_counter,k)
          {"\n"}
          где α = min(0.65, (turn / 8) · (1 - H(p)))
          {"\n"}
          Базовый якорь: W_G40 сохраняет ≥ 35% весов G40
        </MathFormulaBox>

        {/* 4 Counter Vectors Table */}
        <Typography variant="subtitle2" style={{ fontWeight: 700, color: "#334155", marginTop: 18, marginBottom: 8 }}>
          Контр-векторы:
        </Typography>
        <div style={{ overflowX: "auto" }}>
          <Table size="small">
            <TableHead>
              <TableRow>
                <TableCell style={{ fontWeight: 700 }}>Кластер</TableCell>
                <TableCell style={{ fontWeight: 700 }}>Стиль</TableCell>
                <TableCell style={{ fontWeight: 700 }}>Ключевые веса</TableCell>
                <TableCell style={{ fontWeight: 700 }}>Задача</TableCell>
              </TableRow>
            </TableHead>
            <TableBody>
              <TableRow hover>
                <TableCell style={{ fontWeight: 700, color: "#E11D48" }}>Rush</TableCell>
                <TableCell>Ранний прорыв к центру</TableCell>
                <TableCell style={{ fontFamily: "monospace" }}>
                  FRONTIER: 22.0, BOTTLENECK: 20.0, CHOKE: 12.0
                </TableCell>
                <TableCell>Перекрытие коридоров</TableCell>
              </TableRow>
              <TableRow hover>
                <TableCell style={{ fontWeight: 700, color: "#27AE60" }}>Turtle</TableCell>
                <TableCell>Замкнутая оборона</TableCell>
                <TableCell style={{ fontFamily: "monospace" }}>
                  DENY_CAPTURE: 125.0, CAPTURE: 75.0
                </TableCell>
                <TableCell>Блокировка петель</TableCell>
              </TableRow>
              <TableRow hover>
                <TableCell style={{ fontWeight: 700, color: "#2980B9" }}>Flank</TableCell>
                <TableCell>Игра по краям поля</TableCell>
                <TableCell style={{ fontFamily: "monospace" }}>
                  SAFE_TERRITORY: 22.0, MATRIX: 0.12, FRONTIER: 16.0
                </TableCell>
                <TableCell>Защита тылов и краев</TableCell>
              </TableRow>
              <TableRow hover>
                <TableCell style={{ fontWeight: 700, color: "#8E44AD" }}>Chaser</TableCell>
                <TableCell>Контактное давление</TableCell>
                <TableCell style={{ fontFamily: "monospace" }}>
                  SAFE_TERRITORY: 25.0, COMPACTNESS: 20.0, CHOKE: 15.0
                </TableCell>
                <TableCell>Компактная форма</TableCell>
              </TableRow>
            </TableBody>
          </Table>
        </div>
      </DeepDiveCard>
    );
  }

  renderG40DeepDive() {
    const params = weightsData.params || weightsData.weights || {};

    return (
      <DeepDiveCard>
        <div style={{ display: "flex", alignItems: "center", gap: 10, marginBottom: 8 }}>
          <IconAssessment style={{ color: "#FB6542", fontSize: 24 }} />
          <Typography variant="h6" style={{ fontWeight: 800, color: "#1E293B" }}>
            Архитектура G40
          </Typography>
        </div>
        <Typography variant="body2" color="textSecondary" style={{ marginBottom: 14 }}>
          Эвристическая модель с оптимизированными весами целевой функции.
        </Typography>

        <div style={{ overflowX: "auto" }}>
          <Table size="small">
            <TableHead>
              <TableRow>
                <TableCell style={{ fontWeight: 700 }}>Параметр</TableCell>
                <TableCell style={{ fontWeight: 700 }}>Значение</TableCell>
                <TableCell style={{ fontWeight: 700 }}>Описание</TableCell>
              </TableRow>
            </TableHead>
            <TableBody>
              <TableRow hover>
                <TableCell style={{ fontWeight: 700 }}>DENY_CAPTURE</TableCell>
                <TableCell style={{ fontFamily: "monospace", fontWeight: 800, color: "#FB6542" }}>
                  {params.WEIGHT_DENY_CAPTURE || 101.99}
                </TableCell>
                <TableCell>Защита от замыкания противника</TableCell>
              </TableRow>
              <TableRow hover>
                <TableCell style={{ fontWeight: 700 }}>CAPTURE</TableCell>
                <TableCell style={{ fontFamily: "monospace", fontWeight: 800 }}>
                  {params.WEIGHT_CAPTURE || 70.0}
                </TableCell>
                <TableCell>Замыкание контура союзными клетками</TableCell>
              </TableRow>
              <TableRow hover>
                <TableCell style={{ fontWeight: 700 }}>AREA</TableCell>
                <TableCell style={{ fontFamily: "monospace", fontWeight: 800 }}>
                  {params.WEIGHT_AREA || 50.0}
                </TableCell>
                <TableCell>Базовый вес площади фигуры</TableCell>
              </TableRow>
              <TableRow hover>
                <TableCell style={{ fontWeight: 700 }}>VORONOI_DELTA</TableCell>
                <TableCell style={{ fontFamily: "monospace", fontWeight: 800 }}>
                  {params.WEIGHT_VORONOI_DELTA || 45.0}
                </TableCell>
                <TableCell>Контроль свободных зон поля</TableCell>
              </TableRow>
            </TableBody>
          </Table>
        </div>
      </DeepDiveCard>
    );
  }

  render() {
    const { selectedModelId } = this.state;
    const {
      onNavigateToArena,
      onNavigateToSimulator,
      onNavigateToHeuristics
    } = this.props;

    const passport =
      PASSPORTS_DATA[selectedModelId] || PASSPORTS_DATA[BOT_TYPES.APPEX];

    return (
      <Container>
        {/* Model Selector Bar */}
        <ModelSelectorCard>
          <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", flexWrap: "wrap", gap: 8 }}>
            <Typography variant="subtitle1" style={{ fontWeight: 700, color: "#1E293B" }}>
              Модель:
            </Typography>
            <Chip
              label={`${passport.name} (${passport.tier})`}
              size="small"
              style={{ backgroundColor: passport.color, color: "#FFF", fontWeight: 700 }}
            />
          </div>

          {/* Desktop mini-cards */}
          <ModelCardsGrid>
            {MODELS_CATALOG.map(model => {
              const isSelected = model.id === selectedModelId;
              return (
                <ModelSelectMiniCard
                  key={model.id}
                  selected={isSelected}
                  color={model.color}
                  onClick={() => this.handleModelChange(model.id)}
                >
                  <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center" }}>
                    <Typography
                      variant="caption"
                      style={{
                        fontWeight: 800,
                        color: isSelected ? model.color : "#334155"
                      }}
                    >
                      {model.name}
                    </Typography>
                    <Chip
                      label={model.tier}
                      size="small"
                      style={{
                        height: 18,
                        fontSize: 10,
                        backgroundColor: model.color,
                        color: "#FFF",
                        fontWeight: 700
                      }}
                    />
                  </div>
                  <Typography
                    variant="caption"
                    style={{
                      fontSize: 11,
                      color: "#64748B",
                      whiteSpace: "nowrap",
                      overflow: "hidden",
                      textOverflow: "ellipsis"
                    }}
                  >
                    {model.badge}
                  </Typography>
                </ModelSelectMiniCard>
              );
            })}
          </ModelCardsGrid>

          {/* Mobile dropdown */}
          <MobileSelectWrapper>
            <Select
              value={selectedModelId}
              onChange={e => this.handleModelChange(e.target.value)}
              fullWidth
            >
              {MODELS_CATALOG.map(model => (
                <MenuItem key={model.id} value={model.id}>
                  {model.title} [{model.tier}]
                </MenuItem>
              ))}
            </Select>
          </MobileSelectWrapper>
        </ModelSelectorCard>

        {/* Passport Header */}
        <PassportHeader accentColor={passport.color}>
          <div style={{ display: "flex", justifyContent: "space-between", flexWrap: "wrap", gap: 12 }}>
            <div>
              <Typography variant="h5" style={{ fontWeight: 800, color: "#1E293B" }}>
                {passport.name}
              </Typography>
              <Typography variant="body2" color="textSecondary" style={{ marginTop: 4 }}>
                {passport.subtitle}
              </Typography>
            </div>
            <Chip
              label={passport.statusText}
              style={{
                backgroundColor: passport.statusColor || passport.color,
                color: "#FFF",
                fontWeight: 700
              }}
            />
          </div>

          <BadgesRow>
            {passport.badges.map((b, idx) => (
              <Chip
                key={idx}
                label={b.label}
                variant={b.filled ? "default" : "outlined"}
                style={{
                  backgroundColor: b.filled ? b.color || passport.color : undefined,
                  color: b.filled ? "#FFF" : undefined,
                  fontWeight: 600
                }}
              />
            ))}
          </BadgesRow>
        </PassportHeader>

        {/* Quick Action Buttons */}
        <ActionButtonsBar>
          <div style={{ display: "flex", alignItems: "center", gap: 8, flexWrap: "wrap" }}>
            <Typography variant="body2" style={{ fontWeight: 700, color: "#475569", marginRight: 4 }}>
              Тестирование модели:
            </Typography>

            <Button
              variant="outlined"
              size="small"
              onClick={() => onNavigateToArena && onNavigateToArena(selectedModelId)}
              style={{ fontWeight: 600 }}
            >
              <IconCompare style={{ marginRight: 6, fontSize: 18, color: passport.color }} />
              Сразиться на Арене
            </Button>

            <Button
              variant="outlined"
              size="small"
              onClick={() => onNavigateToSimulator && onNavigateToSimulator(selectedModelId)}
              style={{ fontWeight: 600 }}
            >
              <IconPlay style={{ marginRight: 6, fontSize: 18, color: "#27AE60" }} />
              Пошаговый симулятор
            </Button>

            <Button
              variant="outlined"
              size="small"
              onClick={() => onNavigateToHeuristics && onNavigateToHeuristics(selectedModelId)}
              style={{ fontWeight: 600 }}
            >
              <IconTune style={{ marginRight: 6, fontSize: 18, color: "#375E97" }} />
              Сравнить эвристики
            </Button>
          </div>

          <Button
            variant="contained"
            color="primary"
            size="small"
            component={Link}
            to="/ai"
            style={{ fontWeight: 700, backgroundColor: passport.color }}
          >
            <IconGames style={{ marginRight: 6, fontSize: 18 }} />
            Играть против {passport.name}
          </Button>
        </ActionButtonsBar>

        {/* Core Passport Metrics */}
        <MetricsGrid>
          {passport.metrics.map((metric, idx) => (
            <MetricCard key={idx} color={metric.color}>
              <Typography variant="caption" color="textSecondary" style={{ fontWeight: 700 }}>
                {metric.title}
              </Typography>
              <MetricValue color={metric.color}>{metric.value}</MetricValue>
              <Typography variant="caption" style={{ fontWeight: 600, color: "#334155" }}>
                {metric.note}
              </Typography>
              <Typography variant="caption" color="textSecondary">
                {metric.subnote}
              </Typography>
            </MetricCard>
          ))}
        </MetricsGrid>

        {/* Model-Specific Deep Dive (Appex or G40) */}
        {selectedModelId === BOT_TYPES.APPEX && this.renderAppexDeepDive()}
        {(selectedModelId === BOT_TYPES.G40 || selectedModelId === "ARGON_4") &&
          this.renderG40DeepDive()}

        {/* Architecture Components Description */}
        <ArchitectureGrid>
          {passport.architectureCards.map((arch, idx) => (
            <ArchCard key={idx}>
              <ArchHeader>
                {this.renderArchIcon(arch.icon, arch.color)}
                {arch.title}
              </ArchHeader>
              <Typography variant="body2" color="textSecondary">
                {arch.desc}
              </Typography>
            </ArchCard>
          ))}
        </ArchitectureGrid>

        {/* Benchmark Results Table */}
        <Paper style={{ padding: 24, backgroundColor: "#ffffff" }}>
          <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", flexWrap: "wrap", gap: 8, marginBottom: 14 }}>
            <Typography variant="h6" style={{ fontWeight: 700, color: "#1E293B" }}>
              Результаты матчей: {passport.name}
            </Typography>
          </div>

          <div style={{ overflowX: "auto" }}>
            <Table>
              <TableHead>
                <TableRow>
                  <TableCell>Соперник</TableCell>
                  <TableCell>Побед</TableCell>
                  <TableCell>Поражений</TableCell>
                  <TableCell>Винрейт (%)</TableCell>
                  <TableCell>Разница</TableCell>
                  <TableCell>Итог</TableCell>
                </TableRow>
              </TableHead>
              <TableBody>
                {passport.protocol.map((row, idx) => (
                  <TableRow key={idx} hover>
                    <TableCell style={{ fontWeight: 700 }}>{row.opponent}</TableCell>
                    <TableCell style={{ color: "#27AE60", fontWeight: 700 }}>
                      {row.wins}
                    </TableCell>
                    <TableCell style={{ color: "#E11D48", fontWeight: 700 }}>
                      {row.losses}
                    </TableCell>
                    <TableCell style={{ fontWeight: 800, color: row.color || "#1E293B" }}>
                      {row.winrate}
                    </TableCell>
                    <TableCell style={{ color: row.margin.startsWith("+") ? "#27AE60" : "#E11D48", fontWeight: 800 }}>
                      {row.margin}
                    </TableCell>
                    <TableCell>{row.verdict}</TableCell>
                  </TableRow>
                ))}
              </TableBody>
            </Table>
          </div>
        </Paper>

        {/* Dossier Specifications Summary */}
        <Paper style={{ padding: 24, backgroundColor: "#ffffff" }}>
          <Typography variant="h6" style={{ fontWeight: 700, marginBottom: 12, color: "#1E293B" }}>
            Параметры: {passport.name}
          </Typography>
          <DossierGrid>
            <DossierItem>
              <Typography variant="caption" color="textSecondary" style={{ fontWeight: 700 }}>
                АРХИТЕКТУРА
              </Typography>
              <Typography variant="body2" style={{ fontWeight: 700, color: "#1E293B" }}>
                {passport.dossier.type}
              </Typography>
            </DossierItem>

            <DossierItem>
              <Typography variant="caption" color="textSecondary" style={{ fontWeight: 700 }}>
                ВЕРСИЯ
              </Typography>
              <Typography variant="body2" style={{ fontWeight: 700, color: "#1E293B" }}>
                {passport.dossier.generation}
              </Typography>
            </DossierItem>

            <DossierItem>
              <Typography variant="caption" color="textSecondary" style={{ fontWeight: 700 }}>
                МЕТОД ОБУЧЕНИЯ
              </Typography>
              <Typography variant="body2" style={{ fontWeight: 600, color: "#1E293B" }}>
                {passport.dossier.trainingMethod}
              </Typography>
            </DossierItem>

            <DossierItem>
              <Typography variant="caption" color="textSecondary" style={{ fontWeight: 700 }}>
                СИММЕТРИЯ
              </Typography>
              <Typography variant="body2" style={{ fontWeight: 600, color: "#1E293B" }}>
                {passport.dossier.invariance}
              </Typography>
            </DossierItem>

            <DossierItem>
              <Typography variant="caption" color="textSecondary" style={{ fontWeight: 700 }}>
                БАЗОВЫЙ ЯКОРЬ
              </Typography>
              <Typography variant="body2" style={{ fontWeight: 600, color: "#1E293B" }}>
                {passport.dossier.anchorPreservation}
              </Typography>
            </DossierItem>

            <DossierItem>
              <Typography variant="caption" color="textSecondary" style={{ fontWeight: 700 }}>
                PRNG
              </Typography>
              <Typography variant="body2" style={{ fontWeight: 600, color: "#1E293B" }}>
                {passport.dossier.prng}
              </Typography>
            </DossierItem>
          </DossierGrid>
        </Paper>
      </Container>
    );
  }
}
