import React from "react";
import { Client } from "boardgame.io/react";

import AI, { BOT_TYPES } from "../AI";
import Game from "../Game";
import { PLAYER_2 } from "../gameUtils";
import UI from "../UI";

const bot = AI({ playerID: PLAYER_2, type: BOT_TYPES.MEDIUM });

// TODO add redux enhancer with redux-persist, so game can be resumed
const AiPage = Client({
  game: Game,
  board: ({ G, ctx, moves, events, isActive }) => (
    <UI
      ai={bot}
      G={G}
      ctx={ctx}
      moves={moves}
      events={events}
      isActive={isActive}
    />
  ),
  debug: false
});

export default AiPage;
