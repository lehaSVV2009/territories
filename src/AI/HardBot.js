import Bot from "./Bot";

export default class HardBot extends Bot {
  constructor({ playerID, name }) {
    super({ playerID, name: name || "Hard Bot" });
  }

  findBestPlaceForRectangle = () => {
    throw new Error("Not implemented yet");
  };
}
