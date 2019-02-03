import { Client } from "boardgame.io/react";

import Game from "../Game";
import UI from "../UI";

// TODO add redux enhancer with redux-persist, so game can be resumed
const TwoPlayersPage = Client({
  game: Game,
  board: UI,
  debug: false
});

export default TwoPlayersPage;
