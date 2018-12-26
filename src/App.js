import { Client } from "boardgame.io/react";

import AI from "./AI";
import UI from "./UI";
import Game from "./Game";

const Territories = Client({
  game: Game({}),
  board: UI,
  ai: AI,
  debug: false
});

export default Territories;
