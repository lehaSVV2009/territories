import Bot from "./Bot";

export default class HardBot extends Bot {
  constructor({ playerID, name }) {
    super({ playerID, name: name || "Hard Bot" });
  }

  guessCellToDropRectangle = () => {
    throw new Error("Not implemented yet");
  };
}
