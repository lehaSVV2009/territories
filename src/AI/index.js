import EasyBot from "./EasyBot";
import MediumBot from "./MediumBot";
import HardBot from "./HardBot";

export const BOT_TYPES = {
  EASY: "EASY",
  MEDIUM: "MEDIUM",
  HARD: "HARD"
};

export default options => {
  if (!options || !options.playerID) {
    throw new Error("playerID parameter is missing");
  }

  const { playerID, name, type } = options;
  if (type === BOT_TYPES.HARD) {
    return new HardBot({ playerID, name });
  }

  if (type === BOT_TYPES.MEDIUM) {
    return new MediumBot({ playerID, name });
  }

  return new EasyBot({ playerID, name });
};
