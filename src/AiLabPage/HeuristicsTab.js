import React, { Component } from "react";
import styled from "styled-components";
import Typography from "@material-ui/core/Typography";
import Paper from "../libs/territories-ui/Paper";
import Chip from "../libs/territories-ui/Chip";
import Select from "../libs/territories-ui/Select";
import MenuItem from "../libs/territories-ui/MenuItem";

import weightsData from "../AI/weights.json";
import appexWeightsData from "../AI/appex_weights.json";
import { DEFAULT_V2_WEIGHTS } from "../AI/SpatialSlayerBot";
import { MODELS_CATALOG, BOT_TYPES } from "../AI";

const Container = styled.div`
  display: flex;
  flex-direction: column;
  gap: 20px;
`;

const HeaderCard = styled(Paper)`
  padding: 20px;
  background-color: #ffffff;
`;

const SelectorsRow = styled.div`
  display: flex;
  align-items: center;
  justify-content: space-between;
  flex-wrap: wrap;
  gap: 16px;
  margin-top: 14px;
`;

const SelectorBox = styled.div`
  display: flex;
  align-items: center;
  gap: 10px;
`;

const ComparisonGrid = styled.div`
  display: grid;
  grid-template-columns: 1fr;
  gap: 14px;
`;

const ParameterCard = styled(Paper)`
  padding: 16px 20px;
  background-color: #ffffff;
`;

const ParamHeader = styled.div`
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 8px;
`;

const ParamTitle = styled.div`
  font-weight: 700;
  font-size: 15px;
  color: #1e293b;
`;

const ParamDesc = styled.div`
  font-size: 13px;
  color: #64748b;
  margin-bottom: 12px;
`;

const BarTrack = styled.div`
  display: flex;
  flex-direction: column;
  gap: 6px;
`;

const BarRow = styled.div`
  display: flex;
  align-items: center;
  gap: 12px;
`;

const ModelLabel = styled.div`
  width: 140px;
  font-size: 13px;
  font-weight: 600;
  color: ${props => props.color || "#334155"};
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
`;

const BarFillContainer = styled.div`
  flex: 1;
  height: 18px;
  background-color: #e2e8f0;
  border-radius: 4px;
  overflow: hidden;
  position: relative;
`;

const BarFill = styled.div`
  height: 100%;
  width: ${props => props.width}%;
  background-color: ${props => props.color};
  border-radius: 4px;
  transition: width 0.4s ease;
`;

const BarValue = styled.div`
  width: 60px;
  text-align: right;
  font-weight: 700;
  font-size: 13px;
  color: #0f172a;
`;

const InsightsGrid = styled.div`
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(280px, 1fr));
  gap: 16px;
`;

const InsightCard = styled(Paper)`
  padding: 16px;
  border-left: 4px solid ${props => props.color || "#375E97"};
  background-color: #ffffff;
`;

const PARAMETERS_METADATA = [
  {
    key: "WEIGHT_DENY_CAPTURE",
    name: "Срыв захвата противника (Deny Capture)",
    desc: "Штраф/бонус за разрушение потенциальных замкнутых петель оппонента. Решающий фактор чемпионского профиля.",
    max: 120
  },
  {
    key: "WEIGHT_CAPTURE",
    name: "Захват контура (Contour Capture)",
    desc: "Бонус за замыкание пустого пространства в границах собственных клеток с автоматическим перекрашиванием.",
    max: 120
  },
  {
    key: "WEIGHT_VORONOI_DELTA",
    name: "Дельта контроля Вороного (Voronoi Influence)",
    desc: "Разность расстояний при многоисточниковом BFS. Определяет динамическое владение свободными зонами доски.",
    max: 60
  },
  {
    key: "WEIGHT_AREA",
    name: "Площадь фигуры (Фиксированный нумерарий)",
    desc: "Базовый вес площади выставленного прямоугольника (W x H). Нормирован на 50.0 во всех эволюционных запусках.",
    max: 80
  },
  {
    key: "WEIGHT_SAFE_TERRITORY",
    name: "Безопасная тыловая зона (Safe Territory)",
    desc: "Вес клеток, защищенных от вторжения границами доски и плотными рядами союзных блоков.",
    max: 40
  },
  {
    key: "WEIGHT_COMPACTNESS",
    name: "Компактность формы (Compactness)",
    desc: "Поощрение квадратоподобных форм и штраф за вытянутые изолированные полоски толщиной в 1 клетку.",
    max: 30
  },
  {
    key: "WEIGHT_FRONTIER",
    name: "Давление на линию фронта (Frontier)",
    desc: "Плотность контакта с нейтральной зоной соприкосновения с фигурами соперника.",
    max: 30
  },
  {
    key: "WEIGHT_OPP_CHOKE",
    name: "Блокировка коридоров противника (Opponent Choke)",
    desc: "Перекрытие узких проходов и блокирование путей расширения вражеских фигур.",
    max: 30
  },
  {
    key: "WEIGHT_BOTTLENECK",
    name: "Контроль узких мест (Bottleneck Control)",
    desc: "Занятие критических сечений между краями поля и центром.",
    max: 30
  },
  {
    key: "WEIGHT_SPATIAL_MATRIX",
    name: "Вес позиционной матрицы 15x40 (Spatial Matrix)",
    desc: "Масштабирующий коэффициент статической пространственной карты ценности клеток.",
    max: 1.0
  }
];

export default class HeuristicsTab extends Component {
  state = {
    modelA: this.props.initialModelA || BOT_TYPES.APPEX,
    modelB: BOT_TYPES.G40
  };

  componentDidUpdate(prevProps) {
    if (
      this.props.initialModelA &&
      this.props.initialModelA !== prevProps.initialModelA
    ) {
      this.setState({ modelA: this.props.initialModelA });
    }
  }

  getModelWeights(modelId) {
    if (modelId === BOT_TYPES.APPEX) {
      return appexWeightsData.argon4_anchor;
    }
    if (modelId === BOT_TYPES.G40 || modelId === "ARGON_4") {
      return weightsData.params || weightsData.weights;
    }
    if (modelId === BOT_TYPES.SPATIAL_SLAYER) {
      return DEFAULT_V2_WEIGHTS;
    }
    if (modelId === BOT_TYPES.HARD) {
      return {
        WEIGHT_AREA: 40.0,
        WEIGHT_CAPTURE: 50.0,
        WEIGHT_DENY_CAPTURE: 40.0,
        WEIGHT_VORONOI_DELTA: 15.0,
        WEIGHT_SAFE_TERRITORY: 10.0,
        WEIGHT_COMPACTNESS: 5.0,
        WEIGHT_SPATIAL_MATRIX: 0.15,
        WEIGHT_FRONTIER: 10.0,
        WEIGHT_OPP_CHOKE: 25.0,
        WEIGHT_BOTTLENECK: 20.0
      };
    }
    if (modelId === BOT_TYPES.GREEDY) {
      return {
        WEIGHT_AREA: 100.0,
        WEIGHT_CAPTURE: 0.0,
        WEIGHT_DENY_CAPTURE: 0.0,
        WEIGHT_VORONOI_DELTA: 0.0,
        WEIGHT_SAFE_TERRITORY: 0.0,
        WEIGHT_COMPACTNESS: 0.0,
        WEIGHT_SPATIAL_MATRIX: 0.0,
        WEIGHT_FRONTIER: 0.0,
        WEIGHT_OPP_CHOKE: 0.0,
        WEIGHT_BOTTLENECK: 0.0
      };
    }
    return {
      WEIGHT_AREA: 20.0,
      WEIGHT_CAPTURE: 10.0,
      WEIGHT_DENY_CAPTURE: 10.0,
      WEIGHT_VORONOI_DELTA: 5.0,
      WEIGHT_SAFE_TERRITORY: 5.0,
      WEIGHT_COMPACTNESS: 0.0,
      WEIGHT_SPATIAL_MATRIX: 0.0,
      WEIGHT_FRONTIER: 5.0,
      WEIGHT_OPP_CHOKE: 5.0,
      WEIGHT_BOTTLENECK: 5.0
    };
  }

  render() {
    const { modelA, modelB } = this.state;
    const metaA =
      MODELS_CATALOG.find(m => m.id === modelA) || MODELS_CATALOG[0];
    const metaB =
      MODELS_CATALOG.find(m => m.id === modelB) || MODELS_CATALOG[1];

    const weightsA = this.getModelWeights(modelA);
    const weightsB = this.getModelWeights(modelB);

    return (
      <Container>
        {/* Header & Model Selectors */}
        <HeaderCard>
          <Typography variant="h6" style={{ fontWeight: 700 }}>
            Сравнение эвристик
          </Typography>
          <Typography variant="body2" color="textSecondary" style={{ marginTop: 4 }}>
            Веса параметров целевой функции
          </Typography>

          <SelectorsRow>
            <SelectorBox>
              <Typography variant="subtitle2" style={{ fontWeight: 700, color: "#375E97" }}>
                Модель А:
              </Typography>
              <Select
                value={modelA}
                onChange={e => this.setState({ modelA: e.target.value })}
              >
                {MODELS_CATALOG.map(m => (
                  <MenuItem key={m.id} value={m.id}>
                    {m.name}
                  </MenuItem>
                ))}
              </Select>
            </SelectorBox>

            <SelectorBox>
              <Typography variant="subtitle2" style={{ fontWeight: 700, color: "#FB6542" }}>
                Модель Б:
              </Typography>
              <Select
                value={modelB}
                onChange={e => this.setState({ modelB: e.target.value })}
              >
                {MODELS_CATALOG.map(m => (
                  <MenuItem key={m.id} value={m.id}>
                    {m.name}
                  </MenuItem>
                ))}
              </Select>
            </SelectorBox>
          </SelectorsRow>
        </HeaderCard>

        {/* Tactical Insights */}
        <InsightsGrid>
          <InsightCard color="#375E97">
            <Typography variant="subtitle2" style={{ fontWeight: 700, color: "#375E97" }}>
              Защита от захвата
            </Typography>
            <Typography variant="body2" color="textSecondary" style={{ marginTop: 6 }}>
              Приоритет блокировки замыкания противника (101.99) выше немедленного собственного захвата (70.0).
            </Typography>
          </InsightCard>

          <InsightCard color="#FB6542">
            <Typography variant="subtitle2" style={{ fontWeight: 700, color: "#FB6542" }}>
              Зоны Вороного
            </Typography>
            <Typography variant="body2" color="textSecondary" style={{ marginTop: 6 }}>
              Вес VORONOI_DELTA (45.0) обеспечивает контроль свободных зон поля.
            </Typography>
          </InsightCard>

          <InsightCard color="#2C3E50">
            <Typography variant="subtitle2" style={{ fontWeight: 700, color: "#2C3E50" }}>
              Нормировка площади
            </Typography>
            <Typography variant="body2" color="textSecondary" style={{ marginTop: 6 }}>
              Вес площади зафиксирован на 50.0 в качестве базового масштаба.
            </Typography>
          </InsightCard>
        </InsightsGrid>

        {/* Parameter Comparison Bars */}
        <ComparisonGrid>
          {PARAMETERS_METADATA.map(param => {
            const valA = weightsA[param.key] || 0;
            const valB = weightsB[param.key] || 0;
            const pctA = Math.min(100, Math.round((valA / param.max) * 100));
            const pctB = Math.min(100, Math.round((valB / param.max) * 100));

            return (
              <ParameterCard key={param.key}>
                <ParamHeader>
                  <ParamTitle>{param.name}</ParamTitle>
                  <Chip
                    label={param.key}
                    size="small"
                    style={{ fontFamily: "monospace", fontSize: 11 }}
                  />
                </ParamHeader>
                <ParamDesc>{param.desc}</ParamDesc>

                <BarTrack>
                  <BarRow>
                    <ModelLabel color="#375E97">{metaA.name}:</ModelLabel>
                    <BarFillContainer>
                      <BarFill width={pctA} color="#375E97" />
                    </BarFillContainer>
                    <BarValue>{valA}</BarValue>
                  </BarRow>

                  <BarRow>
                    <ModelLabel color="#FB6542">{metaB.name}:</ModelLabel>
                    <BarFillContainer>
                      <BarFill width={pctB} color="#FB6542" />
                    </BarFillContainer>
                    <BarValue>{valB}</BarValue>
                  </BarRow>
                </BarTrack>
              </ParameterCard>
            );
          })}
        </ComparisonGrid>
      </Container>
    );
  }
}
