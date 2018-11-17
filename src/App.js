import { Client } from "boardgame.io/react";

import AI from "./AI";
import Board from "./Board";
import Game from "./Game";

const Territories = Client({
  game: Game,
  board: Board,
  ai: AI
});

export default Territories;
