import { AI } from "boardgame.io/ai";

const TerritoriesAI = AI({
  enumerate: (G, ctx) => {
    const actions = [];
    actions.push({ move: "changeDices", args: [[1, 4]] });
    return actions;
  }
});

export default TerritoriesAI;
