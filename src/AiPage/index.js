import React from "react";
import { Client } from "boardgame.io/react";
import { Game, PLAYER_2 } from "territories-core";

import AI, { BOT_TYPES } from "../AI";
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
