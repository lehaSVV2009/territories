import React from "react";
import styled from "styled-components";
import { Link } from "react-router-dom";

import Drawer from "../libs/territories-ui/Drawer";
import Button from "../libs/territories-ui/Button";
import IconButton from "../libs/territories-ui/IconButton";
import Typography from "@material-ui/core/Typography";
import Chip from "../libs/territories-ui/Chip";
import Divider from "@material-ui/core/Divider";
import IconClose from "@material-ui/icons/Close";
import IconScience from "@material-ui/icons/Assessment";
import IconCheck from "@material-ui/icons/CheckCircle";

import { MODELS_CATALOG } from "../AI";

const DrawerContent = styled.div`
  width: 380px;
  max-width: 90vw;
  padding: 24px;
  box-sizing: border-box;
  font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif;
  color: #2c3e50;
`;

const HeaderRow = styled.div`
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 16px;
`;

const ModelCard = styled.div`
  border: 2px solid ${props => (props.selected ? "#FB6542" : "#e2e8f0")};
  background-color: ${props => (props.selected ? "#fffaf0" : "#ffffff")};
  border-radius: 8px;
  padding: 16px;
  margin-bottom: 14px;
  cursor: pointer;
  transition: all 0.2s ease-in-out;
  box-shadow: ${props =>
    props.selected ? "0 4px 12px rgba(251, 101, 66, 0.15)" : "0 1px 3px rgba(0,0,0,0.05)"};

  &:hover {
    border-color: ${props => (props.selected ? "#FB6542" : "#cbd5e1")};
    transform: translateY(-1px);
  }
`;

const CardTop = styled.div`
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 8px;
`;

const ModelTitle = styled.div`
  font-size: 16px;
  font-weight: 700;
  color: #1e293b;
  display: flex;
  align-items: center;
  gap: 8px;
`;

const ModelDescription = styled.div`
  font-size: 13px;
  color: #64748b;
  line-height: 1.4;
  margin-bottom: 10px;
`;

const StatsGrid = styled.div`
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 6px;
  background-color: ${props => (props.selected ? "#feebc8" : "#f8fafc")};
  border-radius: 6px;
  padding: 8px 10px;
  font-size: 12px;
`;

const StatItem = styled.div`
  display: flex;
  justify-content: space-between;
  color: #475569;
`;

const StatValue = styled.span`
  font-weight: 600;
  color: #0f172a;
`;

const BottomActions = styled.div`
  margin-top: 24px;
  display: flex;
  flex-direction: column;
  gap: 12px;
`;

export default class ModelSelectionDrawer extends React.Component {
  render() {
    const { open, onClose, selectedModel, onSelectModel } = this.props;

    return (
      <Drawer anchor="right" open={open} onClose={onClose}>
        <DrawerContent>
          <HeaderRow>
            <div>
              <Typography variant="h6" style={{ fontWeight: 700 }}>
                Выбор бота
              </Typography>
            </div>
            <IconButton onClick={onClose} size="small">
              <IconClose />
            </IconButton>
          </HeaderRow>

          <Divider style={{ marginBottom: 16 }} />

          <div>
            {MODELS_CATALOG.map(model => {
              const isSelected = selectedModel === model.id;
              return (
                <ModelCard
                  key={model.id}
                  selected={isSelected}
                  onClick={() => onSelectModel(model.id)}
                >
                  <CardTop>
                    <ModelTitle>
                      {isSelected && (
                        <IconCheck style={{ color: "#FB6542", fontSize: 18 }} />
                      )}
                      {model.name}
                    </ModelTitle>
                    <Chip
                      label={model.badge}
                      size="small"
                      style={{
                        backgroundColor: isSelected ? "#FB6542" : "#f1f5f9",
                        color: isSelected ? "#ffffff" : "#475569",
                        fontWeight: 600,
                        fontSize: 11
                      }}
                    />
                  </CardTop>

                  <ModelDescription>{model.description}</ModelDescription>

                  <StatsGrid selected={isSelected}>
                    <StatItem>
                      <span>Vs Greedy:</span>
                      <StatValue>{model.winrateVsGreedy}</StatValue>
                    </StatItem>
                    <StatItem>
                      <span>Vs Hard:</span>
                      <StatValue>{model.winrateVsHard}</StatValue>
                    </StatItem>
                    <StatItem>
                      <span>Разница:</span>
                      <StatValue>{model.avgMargin}</StatValue>
                    </StatItem>
                    <StatItem>
                      <span>Уровень:</span>
                      <StatValue>{model.tier}</StatValue>
                    </StatItem>
                  </StatsGrid>
                </ModelCard>
              );
            })}
          </div>

          <BottomActions>
            <Button
              variant="contained"
              color="secondary"
              fullWidth
              onClick={onClose}
              style={{ fontWeight: 600, padding: "10px 0" }}
            >
              Закрыть
            </Button>
            <Button
              variant="outlined"
              color="primary"
              fullWidth
              component={Link}
              to="/ai-lab"
              onClick={onClose}
              style={{ fontWeight: 600 }}
            >
              <IconScience style={{ marginRight: 8, fontSize: 18 }} />
              Лаборатория
            </Button>
          </BottomActions>
        </DrawerContent>
      </Drawer>
    );
  }
}
