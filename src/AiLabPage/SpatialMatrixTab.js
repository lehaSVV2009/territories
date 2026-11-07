import React, { Component } from "react";
import styled from "styled-components";
import Typography from "@material-ui/core/Typography";
import Paper from "../libs/territories-ui/Paper";
import Chip from "../libs/territories-ui/Chip";
import Button from "../libs/territories-ui/Button";

import weightsData from "../AI/weights.json";
import { generateDefaultMatrix } from "../AI/SpatialSlayerBot";
import { BOARD_ROWS, BOARD_COLS } from "./simulatorEngine";

const Container = styled.div`
  display: flex;
  flex-direction: column;
  gap: 20px;
`;

const TopCard = styled(Paper)`
  padding: 20px;
  background-color: #ffffff;
  display: flex;
  align-items: center;
  justify-content: space-between;
  flex-wrap: wrap;
  gap: 16px;
`;

const MatrixCard = styled(Paper)`
  padding: 20px;
  background-color: #ffffff;
  display: flex;
  flex-direction: column;
  align-items: center;
`;

const SvgBoard = styled.svg`
  width: 100%;
  max-width: 960px;
  height: auto;
  border-radius: 6px;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.08);
  background-color: #0f172a;
`;

const LegendRow = styled.div`
  display: flex;
  align-items: center;
  justify-content: space-between;
  width: 100%;
  max-width: 960px;
  margin-top: 14px;
  flex-wrap: wrap;
  gap: 12px;
`;

const GradientBar = styled.div`
  height: 14px;
  width: 220px;
  border-radius: 4px;
  background: linear-gradient(
    to right,
    #1e293b,
    #0284c7,
    #14b8a6,
    #f59e0b,
    #fb6542
  );
  box-shadow: inset 0 1px 3px rgba(0, 0, 0, 0.2);
`;

const InspectionCard = styled(Paper)`
  padding: 16px 20px;
  background-color: #f8fafc;
  display: flex;
  align-items: center;
  justify-content: space-between;
  flex-wrap: wrap;
  gap: 16px;
`;

function getCellTacticalRole(r, c) {
  if (r === 0 && c === 0) return "Стартовый угол P1";
  if (r === 14 && c === 39) return "Стартовый угол P2";
  if (c >= 18 && c <= 21 && r >= 6 && r <= 8) return "Центр";
  if (c >= 18 && c <= 21) return "Центральная линия";
  if (r === 0 || r === 14) return "Край поля";
  if (c === 10 || c === 30) return "Фланг";
  return "Клетка поля";
}

function interpolateHeatColor(val, min, max) {
  const norm = Math.max(0, Math.min(1, (val - min) / (max - min || 1)));

  let r = 30,
    g = 41,
    b = 59;
  if (norm <= 0.25) {
    const t = norm / 0.25;
    r = Math.round(30 + t * (2 - 30));
    g = Math.round(41 + t * (132 - 41));
    b = Math.round(59 + t * (199 - 59));
  } else if (norm <= 0.5) {
    const t = (norm - 0.25) / 0.25;
    r = Math.round(2 + t * (20 - 2));
    g = Math.round(132 + t * (184 - 132));
    b = Math.round(199 + t * (166 - 199));
  } else if (norm <= 0.75) {
    const t = (norm - 0.5) / 0.25;
    r = Math.round(20 + t * (245 - 20));
    g = Math.round(184 + t * (158 - 184));
    b = Math.round(166 + t * (11 - 166));
  } else {
    const t = (norm - 0.75) / 0.25;
    r = Math.round(245 + t * (251 - 245));
    g = Math.round(158 + t * (101 - 158));
    b = Math.round(11 + t * (66 - 11));
  }

  return `rgb(${r}, ${g}, ${b})`;
}

export default class SpatialMatrixTab extends Component {
  state = {
    mode: "g40", // "g40", "baseline", "delta"
    hoveredCell: null
  };

  handleHoverCell = (r, c, val) => {
    this.setState({ hoveredCell: { r, c, val } });
  };

  handleLeaveCell = () => {
    this.setState({ hoveredCell: null });
  };

  render() {
    const { mode, hoveredCell } = this.state;

    const g40Matrix = weightsData.matrix;
    const baselineMatrix = generateDefaultMatrix();

    let activeMatrix = g40Matrix;
    if (mode === "baseline") {
      activeMatrix = baselineMatrix;
    } else if (mode === "delta") {
      activeMatrix = g40Matrix.map(
        (val, idx) => Math.round((val - baselineMatrix[idx]) * 100) / 100
      );
    }

    let minVal = Infinity;
    let maxVal = -Infinity;
    for (let i = 0; i < activeMatrix.length; i++) {
      const v = activeMatrix[i];
      if (v < minVal) minVal = v;
      if (v > maxVal) maxVal = v;
    }

    const cellW = 20;
    const cellH = 20;
    const svgW = BOARD_COLS * cellW;
    const svgH = BOARD_ROWS * cellH;

    return (
      <Container>
        {/* Header Bar */}
        <TopCard>
          <div>
            <Typography variant="h6" style={{ fontWeight: 700 }}>
              Матрица ценности клеток (15x40)
            </Typography>
            <Typography
              variant="body2"
              color="textSecondary"
              style={{ marginTop: 4 }}
            >
              Статические веса ячеек игрового поля
            </Typography>
          </div>

          <div style={{ display: "flex", gap: 8 }}>
            <Button
              variant={mode === "g40" ? "contained" : "outlined"}
              color={mode === "g40" ? "primary" : "default"}
              size="small"
              onClick={() => this.setState({ mode: "g40" })}
              style={{ fontWeight: 600 }}
            >
              G40
            </Button>
            <Button
              variant={mode === "baseline" ? "contained" : "outlined"}
              color={mode === "baseline" ? "primary" : "default"}
              size="small"
              onClick={() => this.setState({ mode: "baseline" })}
              style={{ fontWeight: 600 }}
            >
              Базовая V2
            </Button>
            <Button
              variant={mode === "delta" ? "contained" : "outlined"}
              color={mode === "delta" ? "primary" : "default"}
              size="small"
              onClick={() => this.setState({ mode: "delta" })}
              style={{ fontWeight: 600 }}
            >
              Дельта эволюции
            </Button>
          </div>
        </TopCard>

        {/* Heatmap Matrix SVG */}
        <MatrixCard>
          <SvgBoard viewBox={`0 0 ${svgW} ${svgH}`}>
            {Array.from({ length: BOARD_ROWS }).map((_, r) =>
              Array.from({ length: BOARD_COLS }).map((_, c) => {
                const idx = r * BOARD_COLS + c;
                const val = activeMatrix[idx];
                const color = interpolateHeatColor(val, minVal, maxVal);
                const isHovered =
                  hoveredCell && hoveredCell.r === r && hoveredCell.c === c;

                return (
                  <rect
                    key={`${r}-${c}`}
                    x={c * cellW}
                    y={r * cellH}
                    width={cellW}
                    height={cellH}
                    fill={color}
                    stroke={isHovered ? "#FFFFFF" : "#0f172a"}
                    strokeWidth={isHovered ? 2 : 0.5}
                    onMouseEnter={() => this.handleHoverCell(r, c, val)}
                    onMouseLeave={this.handleLeaveCell}
                    style={{ cursor: "pointer", transition: "all 0.15s ease" }}
                  />
                );
              })
            )}
          </SvgBoard>

          {/* Legend */}
          <LegendRow>
            <div style={{ display: "flex", alignItems: "center", gap: 10 }}>
              <Typography
                variant="caption"
                style={{ fontWeight: 600, color: "#64748B" }}
              >
                Мин ({minVal})
              </Typography>
              <GradientBar />
              <Typography
                variant="caption"
                style={{ fontWeight: 600, color: "#FB6542" }}
              >
                Макс ({maxVal})
              </Typography>
            </div>

            <Typography variant="caption" color="textSecondary">
              Наведите курсор на клетку для просмотра координат и веса
            </Typography>
          </LegendRow>
        </MatrixCard>

        {/* Inspection Details */}
        <InspectionCard>
          {hoveredCell ? (
            <>
              <div style={{ display: "flex", alignItems: "center", gap: 12 }}>
                <Chip
                  label={`Ряд ${hoveredCell.r}, Колонка ${hoveredCell.c}`}
                  style={{
                    backgroundColor: "#2C3E50",
                    color: "#FFF",
                    fontWeight: 700
                  }}
                />
                <Typography variant="subtitle2" style={{ fontWeight: 700 }}>
                  {getCellTacticalRole(hoveredCell.r, hoveredCell.c)}
                </Typography>
              </div>
              <div style={{ display: "flex", alignItems: "center", gap: 10 }}>
                <Typography variant="body2" color="textSecondary">
                  Вес:
                </Typography>
                <Chip
                  label={`${hoveredCell.val}`}
                  style={{
                    backgroundColor: "#FB6542",
                    color: "#FFF",
                    fontWeight: 800,
                    fontSize: 14
                  }}
                />
              </div>
            </>
          ) : (
            <Typography variant="body2" color="textSecondary">
              Наведите курсор на клетку поля.
            </Typography>
          )}
        </InspectionCard>
      </Container>
    );
  }
}