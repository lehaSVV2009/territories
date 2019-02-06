import Bot from "./Bot";

export default class MediumBot extends Bot {
  constructor({ playerID, name }) {
    super({ playerID, name: name || "Medium Bot" });
  }

  findBestPlaceForRectangle = () => {
    throw new Error("Not implemented yet");
  };
}
