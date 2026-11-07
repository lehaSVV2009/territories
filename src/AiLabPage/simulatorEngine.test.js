import { simulateSingleMatch, runTournamentBatch } from "./simulatorEngine";
import { BOT_TYPES } from "../AI";

describe("simulatorEngine", () => {
  it("simulates a single match between G40 and GreedyBot", () => {
    const result = simulateSingleMatch({
      p1Type: BOT_TYPES.G40,
      p2Type: BOT_TYPES.GREEDY,
      seed: 42,
      maxTurns: 30,
      recordHistory: true
    });

    expect(result).toBeDefined();
    expect(result.p1Score).toBeGreaterThan(0);
    expect(result.totalTurns).toBeGreaterThan(1);
    expect(result.history.length).toBeGreaterThan(1);
    expect(result.history[0].turn).toBe(0);
  });
});
