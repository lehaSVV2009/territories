import EasyBot from "./EasyBot";
import MediumBot from "./MediumBot";
import HardBot from "./HardBot";
import GreedyBot from "./GreedyBot";
import FastBot from "./FastBot";
import SpatialSlayerBot from "./SpatialSlayerBot";
import G40Bot from "./G40Bot";
import AppexBot from "./AppexBot";

export const BOT_TYPES = {
  APPEX: "APPEX",
  G40: "G40",
  ARGON_4: "G40",
  SPATIAL_SLAYER: "SPATIAL_SLAYER",
  HARD: "HARD",
  MEDIUM: "MEDIUM",
  GREEDY: "GREEDY",
  FAST: "FAST",
  EASY: "EASY"
};

export const MODELS_CATALOG = [
  {
    id: BOT_TYPES.APPEX,
    name: "Appex",
    title: "Appex",
    badge: "Адаптивный",
    tier: "S",
    color: "#9B59B6",
    description:
      "Адаптирует веса в реальном времени под стиль соперника.",
    winrateVsGreedy: "99.8%",
    winrateVsHard: "92.0%",
    winrateVsBaseline: "78.0%",
    avgMargin: "+58.4",
    tactics: "Адаптация весов"
  },
  {
    id: BOT_TYPES.G40,
    name: "G40",
    title: "G40",
    badge: "Оптимизированный",
    tier: "S",
    color: "#FB6542",
    description:
      "Оптимизированные эвристические веса с защитой от замыкания.",
    winrateVsGreedy: "99.5%",
    winrateVsHard: "85.0%",
    winrateVsBaseline: "65.0%",
    avgMargin: "+42.7",
    tactics: "Защита контура, зоны Вороного"
  },
  {
    id: BOT_TYPES.SPATIAL_SLAYER,
    name: "SpatialSlayer V2",
    title: "SpatialSlayer V2",
    badge: "Базовый",
    tier: "A",
    color: "#375E97",
    description:
      "Базовая модель с контролем центра и расчетом зон Вороного.",
    winrateVsGreedy: "98.0%",
    winrateVsHard: "80.0%",
    winrateVsBaseline: "50.0%",
    avgMargin: "+0.0",
    tactics: "Контроль центра"
  },
  {
    id: BOT_TYPES.HARD,
    name: "HardBot",
    title: "HardBot",
    badge: "Сложный",
    tier: "B",
    color: "#8E44AD",
    description:
      "Эвристический бот с захватом центра и блокировкой линий.",
    winrateVsGreedy: "85.0%",
    winrateVsHard: "50.0%",
    winrateVsBaseline: "20.0%",
    avgMargin: "-121.0",
    tactics: "Центр и края"
  },
  {
    id: BOT_TYPES.GREEDY,
    name: "GreedyBot",
    title: "GreedyBot",
    badge: "Жадный",
    tier: "C",
    color: "#27AE60",
    description:
      "Выбирает ход с максимальной площадью фигуры.",
    winrateVsGreedy: "50.0%",
    winrateVsHard: "15.0%",
    winrateVsBaseline: "0.5%",
    avgMargin: "-218.5",
    tactics: "Максимизация площади"
  },
  {
    id: BOT_TYPES.FAST,
    name: "FastBot",
    title: "FastBot",
    badge: "Быстрый",
    tier: "C",
    color: "#E67E22",
    description:
      "Выбирает первый допустимый ход.",
    winrateVsGreedy: "35.0%",
    winrateVsHard: "10.0%",
    winrateVsBaseline: "1.0%",
    avgMargin: "-240.0",
    tactics: "Первый доступный ход"
  },
  {
    id: BOT_TYPES.MEDIUM,
    name: "MediumBot",
    title: "MediumBot",
    badge: "Средний",
    tier: "C",
    color: "#2980B9",
    description:
      "Нейросеть на базе Brain.js.",
    winrateVsGreedy: "60.0%",
    winrateVsHard: "25.0%",
    winrateVsBaseline: "5.0%",
    avgMargin: "-180.0",
    tactics: "Паттерны ходов"
  },
  {
    id: BOT_TYPES.EASY,
    name: "EasyBot",
    title: "EasyBot",
    badge: "Простой",
    tier: "D",
    color: "#7F8C8D",
    description:
      "Случайный выбор хода.",
    winrateVsGreedy: "20.0%",
    winrateVsHard: "5.0%",
    winrateVsBaseline: "0.1%",
    avgMargin: "-290.0",
    tactics: "Случайный ход"
  }
];

export default options => {
  if (!options || options.playerID === undefined) {
    throw new Error("playerID parameter is missing");
  }

  const { playerID, name, type } = options;

  switch (type) {
    case BOT_TYPES.APPEX:
      return new AppexBot({ playerID, name });
    case BOT_TYPES.G40:
    case "ARGON_4":
      return new G40Bot({ playerID, name });
    case BOT_TYPES.SPATIAL_SLAYER:
      return new SpatialSlayerBot({ playerID, name });
    case BOT_TYPES.GREEDY:
      return new GreedyBot({ playerID, name });
    case BOT_TYPES.FAST:
      return new FastBot({ playerID, name });
    case BOT_TYPES.HARD:
      return new HardBot({ playerID, name });
    case BOT_TYPES.MEDIUM:
      return new MediumBot({ playerID, name });
    case BOT_TYPES.EASY:
    default:
      return new EasyBot({ playerID, name });
  }
};
