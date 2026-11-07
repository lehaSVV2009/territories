import SpatialSlayerBot from "./SpatialSlayerBot";
import weightsData from "./weights.json";

export default class G40Bot extends SpatialSlayerBot {
  constructor({ playerID, name } = {}) {
    const weights = weightsData.params || weightsData.weights;
    const matrix = weightsData.matrix;
    super({
      playerID,
      name: name || "G40 (Чемпион PyRust Gen 40)",
      weights,
      matrix
    });
  }
}
