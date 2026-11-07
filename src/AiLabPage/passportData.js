import { BOT_TYPES } from "../AI";

export const PASSPORTS_DATA = {
  [BOT_TYPES.APPEX]: {
    id: BOT_TYPES.APPEX,
    name: "Appex",
    title: "Appex",
    subtitle: "Адаптивная модель с профилированием оппонента и интерполяцией весов.",
    tier: "S",
    color: "#9B59B6",
    statusText: "Готов",
    statusColor: "#9B59B6",
    badges: [
      { label: "Ранг: S", color: "#9B59B6", filled: true },
      { label: "Адаптивные веса", filled: false },
      { label: "Якорь G40 >= 35%", filled: false },
      { label: "Симметрия 180°", filled: false }
    ],
    metrics: [
      {
        title: "ВИНРЕЙТ VS ZOO",
        value: "92.5%",
        color: "#9B59B6",
        note: "185 побед из 200",
        subnote: ""
      },
      {
        title: "ВИНРЕЙТ VS G40",
        value: "50.0%",
        color: "#FB6542",
        note: "100 побед из 200",
        subnote: ""
      },
      {
        title: "ВИНРЕЙТ VS GREEDY & HARD",
        value: "95.9%",
        color: "#27AE60",
        note: "Партии контрольной серии",
        subnote: ""
      },
      {
        title: "СРЕДНЯЯ РАЗНИЦА",
        value: "+58.4",
        color: "#2C3E50",
        note: "Клеток в среднем",
        subnote: ""
      }
    ],
    architectureCards: [
      {
        title: "Профилирование оппонента",
        icon: "Timeline",
        color: "#9B59B6",
        desc: "Анализ метрик хода соперника: глубина продвижения, фланги, активность контуров и плотность фигур."
      },
      {
        title: "Классификатор стилей",
        icon: "Assessment",
        color: "#2980B9",
        desc: "Определение стиля соперника по 4 паттернам с экспоненциальным сглаживанием."
      },
      {
        title: "Интерполяция весов",
        icon: "Tune",
        color: "#FB6542",
        desc: "Плавное смешивание весов контр-векторов с гарантированным сохранением базовых весов G40."
      },
      {
        title: "Контр-векторы",
        icon: "Security",
        color: "#27AE60",
        desc: "Наборы эвристик под разные стили игры (раш, защита, фланги, прессинг)."
      }
    ],
    protocol: [
      {
        opponent: "Zoo & PhaseShift",
        wins: 185,
        losses: 15,
        winrate: "92.5%",
        margin: "+64.2",
        verdict: "Преимущество",
        color: "#27AE60"
      },
      {
        opponent: "G40",
        wins: 100,
        losses: 100,
        winrate: "50.0%",
        margin: "+0.2",
        verdict: "Паритет",
        color: "#375E97"
      },
      {
        opponent: "HardBot",
        wins: 46,
        losses: 4,
        winrate: "92.0%",
        margin: "+142.5",
        verdict: "Преимущество",
        color: "#27AE60"
      },
      {
        opponent: "SpatialSlayer V2",
        wins: 39,
        losses: 11,
        winrate: "78.0%",
        margin: "+62.8",
        verdict: "Преимущество",
        color: "#27AE60"
      },
      {
        opponent: "GreedyBot",
        wins: 50,
        losses: 0,
        winrate: "100.0%",
        margin: "+234.0",
        verdict: "Преимущество",
        color: "#27AE60"
      }
    ],
    dossier: {
      type: "Адаптивный алгоритм",
      generation: "v1.0",
      trainingMethod: "Оптимизация контр-векторов CMA-ES",
      invariance: "Симметрия 180°",
      anchorPreservation: ">= 35% весов G40",
      prng: "Mulberry32"
    }
  },

  [BOT_TYPES.G40]: {
    id: BOT_TYPES.G40,
    name: "G40",
    title: "G40",
    subtitle: "Оптимизированная эвристическая модель с защитой от замыкания контура.",
    tier: "S",
    color: "#FB6542",
    statusText: "Готов",
    statusColor: "#27AE60",
    badges: [
      { label: "Ранг: S", color: "#FB6542", filled: true },
      { label: "Оптимизация весов", filled: false },
      { label: "Симметрия 180°", filled: false }
    ],
    metrics: [
      {
        title: "ВИНРЕЙТ VS GREEDYBOT",
        value: "99.5%",
        color: "#FB6542",
        note: "50 побед из 50",
        subnote: ""
      },
      {
        title: "ВИНРЕЙТ VS HARDBOT",
        value: "86.0%",
        color: "#375E97",
        note: "43 победы из 50",
        subnote: ""
      },
      {
        title: "ВИНРЕЙТ VS BASELINE V2",
        value: "66.0%",
        color: "#27AE60",
        note: "33 победы из 50",
        subnote: ""
      },
      {
        title: "СРЕДНЯЯ РАЗНИЦА",
        value: "+42.7",
        color: "#2C3E50",
        note: "Клеток в турнире",
        subnote: ""
      }
    ],
    architectureCards: [
      {
        title: "Эвристический поиск",
        icon: "Timeline",
        color: "#FB6542",
        desc: "Оценка ходов с детекцией замкнутых контуров и зон Вороного."
      },
      {
        title: "Оптимизация CMA-ES",
        icon: "Assessment",
        color: "#27AE60",
        desc: "Подбор весов эвристик методом ковариационной матрицы."
      },
      {
        title: "Защита контура",
        icon: "Security",
        color: "#8E44AD",
        desc: "Высокий приоритет срыва замыкания фигур противником."
      },
      {
        title: "Контроль зон Вороного",
        icon: "Storage",
        color: "#375E97",
        desc: "Оценка влияния на свободные клетки поля."
      }
    ],
    protocol: [
      {
        opponent: "SpatialSlayer V2",
        wins: 33,
        losses: 17,
        winrate: "66.0%",
        margin: "+42.7",
        verdict: "Преимущество",
        color: "#27AE60"
      },
      {
        opponent: "HardBot",
        wins: 43,
        losses: 7,
        winrate: "86.0%",
        margin: "+121.0",
        verdict: "Преимущество",
        color: "#27AE60"
      },
      {
        opponent: "GreedyBot",
        wins: 50,
        losses: 0,
        winrate: "100.0%",
        margin: "+218.5",
        verdict: "Преимущество",
        color: "#27AE60"
      },
      {
        opponent: "EasyBot",
        wins: 50,
        losses: 0,
        winrate: "100.0%",
        margin: "+340.0",
        verdict: "Преимущество",
        color: "#27AE60"
      }
    ],
    dossier: {
      type: "Статическая эвристика",
      generation: "Gen 40",
      trainingMethod: "Эволюционная селекция CMA-ES",
      invariance: "Симметрия 180°",
      anchorPreservation: "100% веса",
      prng: "Mulberry32"
    }
  },

  [BOT_TYPES.SPATIAL_SLAYER]: {
    id: BOT_TYPES.SPATIAL_SLAYER,
    name: "SpatialSlayer V2",
    title: "SpatialSlayer V2",
    subtitle: "Базовая модель с расчетом зон Вороного и контролем центра поля.",
    tier: "A",
    color: "#375E97",
    statusText: "Готов",
    statusColor: "#375E97",
    badges: [
      { label: "Ранг: A", color: "#375E97", filled: true },
      { label: "Зоны Вороного", filled: false },
      { label: "Матрица поля", filled: false }
    ],
    metrics: [
      {
        title: "ВИНРЕЙТ VS GREEDYBOT",
        value: "98.0%",
        color: "#27AE60",
        note: "49 побед из 50",
        subnote: ""
      },
      {
        title: "ВИНРЕЙТ VS HARDBOT",
        value: "80.0%",
        color: "#375E97",
        note: "40 побед из 50",
        subnote: ""
      },
      {
        title: "ВИНРЕЙТ VS BASELINE",
        value: "50.0%",
        color: "#7F8C8D",
        note: "Паритет",
        subnote: ""
      },
      {
        title: "СРЕДНЯЯ РАЗНИЦА",
        value: "+0.0",
        color: "#2C3E50",
        note: "Базовый уровень",
        subnote: ""
      }
    ],
    architectureCards: [
      {
        title: "BFS Вороного",
        icon: "Timeline",
        color: "#375E97",
        desc: "Волновой расчет владения свободными клетками от выставленных блоков."
      },
      {
        title: "Матрица ценности",
        icon: "Storage",
        color: "#2980B9",
        desc: "Статическая карта приоритетов с акцентом на центральную зону."
      },
      {
        title: "Связность фронта",
        icon: "Security",
        color: "#27AE60",
        desc: "Контроль непрерывности выставляемых блоков."
      },
      {
        title: "Контроль краев",
        icon: "Assessment",
        color: "#8E44AD",
        desc: "Защита боковых направлений поля."
      }
    ],
    protocol: [
      {
        opponent: "HardBot",
        wins: 40,
        losses: 10,
        winrate: "80.0%",
        margin: "+78.5",
        verdict: "Преимущество",
        color: "#27AE60"
      },
      {
        opponent: "GreedyBot",
        wins: 49,
        losses: 1,
        winrate: "98.0%",
        margin: "+175.2",
        verdict: "Преимущество",
        color: "#27AE60"
      },
      {
        opponent: "G40",
        wins: 17,
        losses: 33,
        winrate: "34.0%",
        margin: "-42.7",
        verdict: "Поражение",
        color: "#E11D48"
      },
      {
        opponent: "Appex",
        wins: 11,
        losses: 39,
        winrate: "22.0%",
        margin: "-62.8",
        verdict: "Поражение",
        color: "#E11D48"
      }
    ],
    dossier: {
      type: "Базовая эвристика",
      generation: "V2",
      trainingMethod: "Калибровка весов Voronoi BFS",
      invariance: "Относительные координаты",
      anchorPreservation: "Базовые веса",
      prng: "Math.random"
    }
  },

  [BOT_TYPES.HARD]: {
    id: BOT_TYPES.HARD,
    name: "HardBot",
    title: "HardBot",
    subtitle: "Классический бот: захват центра и блокировка краев поля.",
    tier: "B",
    color: "#8E44AD",
    statusText: "Готов",
    statusColor: "#8E44AD",
    badges: [
      { label: "Ранг: B", color: "#8E44AD", filled: true },
      { label: "Центр поля", filled: false },
      { label: "Блокировка", filled: false }
    ],
    metrics: [
      {
        title: "ВИНРЕЙТ VS GREEDYBOT",
        value: "85.0%",
        color: "#8E44AD",
        note: "42 победы из 50",
        subnote: ""
      },
      {
        title: "ВИНРЕЙТ VS EASYBOT",
        value: "96.0%",
        color: "#27AE60",
        note: "48 побед из 50",
        subnote: ""
      },
      {
        title: "ВИНРЕЙТ VS BASELINE V2",
        value: "20.0%",
        color: "#E11D48",
        note: "10 побед из 50",
        subnote: ""
      },
      {
        title: "СРЕДНЯЯ РАЗНИЦА",
        value: "-121.0",
        color: "#E11D48",
        note: "В матчах vs G40",
        subnote: ""
      }
    ],
    architectureCards: [
      {
        title: "Захват центра",
        icon: "Timeline",
        color: "#8E44AD",
        desc: "Ранний выход в центральные колонки поля."
      },
      {
        title: "Разделительные линии",
        icon: "Security",
        color: "#2980B9",
        desc: "Построение вертикальных линий для ограничения ходов соперника."
      },
      {
        title: "Контроль периметра",
        icon: "Storage",
        color: "#27AE60",
        desc: "Блокировка верхнего и нижнего краев поля."
      },
      {
        title: "Детерминированный выбор",
        icon: "Assessment",
        color: "#FB6542",
        desc: "Фиксированные правила выбора позиции."
      }
    ],
    protocol: [
      {
        opponent: "GreedyBot",
        wins: 42,
        losses: 8,
        winrate: "84.0%",
        margin: "+95.0",
        verdict: "Преимущество",
        color: "#27AE60"
      },
      {
        opponent: "EasyBot",
        wins: 48,
        losses: 2,
        winrate: "96.0%",
        margin: "+210.0",
        verdict: "Преимущество",
        color: "#27AE60"
      },
      {
        opponent: "G40",
        wins: 7,
        losses: 43,
        winrate: "14.0%",
        margin: "-121.0",
        verdict: "Поражение",
        color: "#E11D48"
      },
      {
        opponent: "SpatialSlayer V2",
        wins: 10,
        losses: 40,
        winrate: "20.0%",
        margin: "-78.5",
        verdict: "Поражение",
        color: "#E11D48"
      }
    ],
    dossier: {
      type: "Правила (Rule-Based)",
      generation: "Классический",
      trainingMethod: "Эвристические правила",
      invariance: "Относительные координаты",
      anchorPreservation: "—",
      prng: "Детерминированный"
    }
  },

  [BOT_TYPES.GREEDY]: {
    id: BOT_TYPES.GREEDY,
    name: "GreedyBot",
    title: "GreedyBot",
    subtitle: "Жадный алгоритм, выбирающий фигуру с максимальной площадью.",
    tier: "C",
    color: "#27AE60",
    statusText: "Готов",
    statusColor: "#27AE60",
    badges: [
      { label: "Ранг: C", color: "#27AE60", filled: true },
      { label: "Максимальная площадь", filled: false }
    ],
    metrics: [
      {
        title: "ВИНРЕЙТ VS EASYBOT",
        value: "80.0%",
        color: "#27AE60",
        note: "40 побед из 50",
        subnote: ""
      },
      {
        title: "ВИНРЕЙТ VS HARDBOT",
        value: "16.0%",
        color: "#E11D48",
        note: "8 побед из 50",
        subnote: ""
      },
      {
        title: "ВИНРЕЙТ VS G40",
        value: "0.0%",
        color: "#E11D48",
        note: "0 побед из 50",
        subnote: ""
      },
      {
        title: "СРЕДНЯЯ РАЗНИЦА",
        value: "-218.5",
        color: "#E11D48",
        note: "В матчах vs G40",
        subnote: ""
      }
    ],
    architectureCards: [
      {
        title: "Максимизация площади",
        icon: "Timeline",
        color: "#27AE60",
        desc: "Выбор хода с наибольшим значением W * H на текущем шаге."
      },
      {
        title: "Без горизонта планирования",
        icon: "Assessment",
        color: "#E67E22",
        desc: "Оценка хода без учета последующих шагов."
      },
      {
        title: "Быстрый расчет",
        icon: "Storage",
        color: "#375E97",
        desc: "Минимальное время отклика за счет простоты критерия."
      },
      {
        title: "Отсутствие защиты",
        icon: "Security",
        color: "#E11D48",
        desc: "Не учитывает замыкание контуров противником."
      }
    ],
    protocol: [
      {
        opponent: "EasyBot",
        wins: 40,
        losses: 10,
        winrate: "80.0%",
        margin: "+85.0",
        verdict: "Преимущество",
        color: "#27AE60"
      },
      {
        opponent: "HardBot",
        wins: 8,
        losses: 42,
        winrate: "16.0%",
        margin: "-95.0",
        verdict: "Поражение",
        color: "#E11D48"
      },
      {
        opponent: "G40",
        wins: 0,
        losses: 50,
        winrate: "0.0%",
        margin: "-218.5",
        verdict: "Поражение",
        color: "#E11D48"
      },
      {
        opponent: "Appex",
        wins: 0,
        losses: 50,
        winrate: "0.0%",
        margin: "-234.0",
        verdict: "Поражение",
        color: "#E11D48"
      }
    ],
    dossier: {
      type: "Жадный алгоритм",
      generation: "Базовый",
      trainingMethod: "argmax(W * H)",
      invariance: "Инвариантен к ориентации",
      anchorPreservation: "—",
      prng: "Детерминированный"
    }
  },

  [BOT_TYPES.FAST]: {
    id: BOT_TYPES.FAST,
    name: "FastBot",
    title: "FastBot",
    subtitle: "Выбор первого найденного допустимого хода без оценки.",
    tier: "C",
    color: "#E67E22",
    statusText: "Готов",
    statusColor: "#E67E22",
    badges: [
      { label: "Ранг: C", color: "#E67E22", filled: true },
      { label: "Минимальная задержка", filled: false }
    ],
    metrics: [
      {
        title: "ВИНРЕЙТ VS EASYBOT",
        value: "76.0%",
        color: "#27AE60",
        note: "38 побед из 50",
        subnote: ""
      },
      {
        title: "ВИНРЕЙТ VS GREEDYBOT",
        value: "36.0%",
        color: "#E11D48",
        note: "18 побед из 50",
        subnote: ""
      },
      {
        title: "ВИНРЕЙТ VS HARDBOT",
        value: "10.0%",
        color: "#E11D48",
        note: "5 побед из 50",
        subnote: ""
      },
      {
        title: "СРЕДНЯЯ РАЗНИЦА",
        value: "-240.0",
        color: "#E11D48",
        note: "В матчах vs G40",
        subnote: ""
      }
    ],
    architectureCards: [
      {
        title: "Первый допустимый ход",
        icon: "Timeline",
        color: "#E67E22",
        desc: "Возврат первой валидной позиции при сканировании поля."
      },
      {
        title: "Минимальный оверхед",
        icon: "Storage",
        color: "#375E97",
        desc: "Не выделяет вспомогательную память и не строит деревья."
      },
      {
        title: "Плотное заполнение",
        icon: "Security",
        color: "#27AE60",
        desc: "Лексикографический обход дает компактное заполнение стартовой зоны."
      },
      {
        title: "Тестирование UI",
        icon: "Assessment",
        color: "#8E44AD",
        desc: "Подходит для тестирования анимаций и интерфейса с максимальной скоростью."
      }
    ],
    protocol: [
      {
        opponent: "EasyBot",
        wins: 38,
        losses: 12,
        winrate: "76.0%",
        margin: "+65.0",
        verdict: "Преимущество",
        color: "#27AE60"
      },
      {
        opponent: "GreedyBot",
        wins: 18,
        losses: 32,
        winrate: "36.0%",
        margin: "-85.0",
        verdict: "Поражение",
        color: "#E11D48"
      },
      {
        opponent: "HardBot",
        wins: 5,
        losses: 45,
        winrate: "10.0%",
        margin: "-180.0",
        verdict: "Поражение",
        color: "#E11D48"
      },
      {
        opponent: "G40",
        wins: 0,
        losses: 50,
        winrate: "0.0%",
        margin: "-260.0",
        verdict: "Поражение",
        color: "#E11D48"
      }
    ],
    dossier: {
      type: "Линейный скан",
      generation: "Базовый",
      trainingMethod: "Первый валидный ход",
      invariance: "От стартового угла",
      anchorPreservation: "—",
      prng: "Детерминированный"
    }
  },

  [BOT_TYPES.MEDIUM]: {
    id: BOT_TYPES.MEDIUM,
    name: "MediumBot",
    title: "MediumBot",
    subtitle: "Нейросеть прямого распространения на библиотеке Brain.js.",
    tier: "C",
    color: "#2980B9",
    statusText: "Готов",
    statusColor: "#2980B9",
    badges: [
      { label: "Ранг: C", color: "#2980B9", filled: true },
      { label: "Brain.js", filled: false }
    ],
    metrics: [
      {
        title: "ВИНРЕЙТ VS GREEDYBOT",
        value: "60.0%",
        color: "#2980B9",
        note: "30 побед из 50",
        subnote: ""
      },
      {
        title: "ВИНРЕЙТ VS EASYBOT",
        value: "90.0%",
        color: "#27AE60",
        note: "45 побед из 50",
        subnote: ""
      },
      {
        title: "ВИНРЕЙТ VS HARDBOT",
        value: "24.0%",
        color: "#E11D48",
        note: "12 побед из 50",
        subnote: ""
      },
      {
        title: "СРЕДНЯЯ РАЗНИЦА",
        value: "-180.0",
        color: "#E11D48",
        note: "В матчах vs G40",
        subnote: ""
      }
    ],
    architectureCards: [
      {
        title: "Полносвязная сеть",
        icon: "Timeline",
        color: "#2980B9",
        desc: "Оценка ходов с помощью перцептрона Brain.js."
      },
      {
        title: "Паттерны центра",
        icon: "Assessment",
        color: "#8E44AD",
        desc: "Обучение на эвристических ходах для продвижения к центру поля."
      },
      {
        title: "Ранжирование",
        icon: "Security",
        color: "#27AE60",
        desc: "Выбор допустимого хода с максимальным откликом сети."
      },
      {
        title: "Ограничения",
        icon: "Storage",
        color: "#FB6542",
        desc: "Не выполняет глубокий поиск замкнутых областей."
      }
    ],
    protocol: [
      {
        opponent: "GreedyBot",
        wins: 30,
        losses: 20,
        winrate: "60.0%",
        margin: "+15.0",
        verdict: "Преимущество",
        color: "#27AE60"
      },
      {
        opponent: "EasyBot",
        wins: 45,
        losses: 5,
        winrate: "90.0%",
        margin: "+140.0",
        verdict: "Преимущество",
        color: "#27AE60"
      },
      {
        opponent: "HardBot",
        wins: 12,
        losses: 38,
        winrate: "24.0%",
        margin: "-120.0",
        verdict: "Поражение",
        color: "#E11D48"
      },
      {
        opponent: "G40",
        wins: 1,
        losses: 49,
        winrate: "2.0%",
        margin: "-220.0",
        verdict: "Поражение",
        color: "#E11D48"
      }
    ],
    dossier: {
      type: "Нейросеть (ANN)",
      generation: "Экспериментальный",
      trainingMethod: "Supervised Learning",
      invariance: "Вектор признаков",
      anchorPreservation: "—",
      prng: "Brain.js"
    }
  },

  [BOT_TYPES.EASY]: {
    id: BOT_TYPES.EASY,
    name: "EasyBot",
    title: "EasyBot",
    subtitle: "Случайный равновероятный выбор хода из доступных.",
    tier: "D",
    color: "#7F8C8D",
    statusText: "Готов",
    statusColor: "#7F8C8D",
    badges: [
      { label: "Ранг: D", color: "#7F8C8D", filled: true },
      { label: "Случайный выбор", filled: false }
    ],
    metrics: [
      {
        title: "ВИНРЕЙТ VS EASYBOT",
        value: "50.0%",
        color: "#7F8C8D",
        note: "25 побед из 50",
        subnote: ""
      },
      {
        title: "ВИНРЕЙТ VS GREEDYBOT",
        value: "20.0%",
        color: "#E11D48",
        note: "10 побед из 50",
        subnote: ""
      },
      {
        title: "ВИНРЕЙТ VS HARDBOT",
        value: "4.0%",
        color: "#E11D48",
        note: "2 победы из 50",
        subnote: ""
      },
      {
        title: "СРЕДНЯЯ РАЗНИЦА",
        value: "-290.0",
        color: "#E11D48",
        note: "В матчах vs G40",
        subnote: ""
      }
    ],
    architectureCards: [
      {
        title: "Равномерное распределение",
        icon: "Timeline",
        color: "#7F8C8D",
        desc: "Случайный выбор из списка всех допустимых ходов."
      },
      {
        title: "Без критерия",
        icon: "Assessment",
        color: "#E67E22",
        desc: "Ходы совершаются без оценочной функции."
      },
      {
        title: "Несвязные блоки",
        icon: "Security",
        color: "#E11D48",
        desc: "Хаотичное размещение без сохранения фронта."
      },
      {
        title: "Базовая точка отсчета",
        icon: "Storage",
        color: "#2C3E50",
        desc: "Служит минимальным порогом проверки эффективности алгоритмов."
      }
    ],
    protocol: [
      {
        opponent: "FastBot",
        wins: 12,
        losses: 38,
        winrate: "24.0%",
        margin: "-65.0",
        verdict: "Поражение",
        color: "#E11D48"
      },
      {
        opponent: "GreedyBot",
        wins: 10,
        losses: 40,
        winrate: "20.0%",
        margin: "-85.0",
        verdict: "Поражение",
        color: "#E11D48"
      },
      {
        opponent: "HardBot",
        wins: 2,
        losses: 48,
        winrate: "4.0%",
        margin: "-210.0",
        verdict: "Поражение",
        color: "#E11D48"
      },
      {
        opponent: "G40",
        wins: 0,
        losses: 50,
        winrate: "0.0%",
        margin: "-340.0",
        verdict: "Поражение",
        color: "#E11D48"
      }
    ],
    dossier: {
      type: "Случайный выбор",
      generation: "Базовый",
      trainingMethod: "Равномерный генератор",
      invariance: "Стохастический",
      anchorPreservation: "—",
      prng: "Math.random"
    }
  }
};

// Backward-compatibility alias
if (BOT_TYPES.ARGON_4 && !PASSPORTS_DATA[BOT_TYPES.ARGON_4]) {
  PASSPORTS_DATA[BOT_TYPES.ARGON_4] = PASSPORTS_DATA[BOT_TYPES.G40];
}
