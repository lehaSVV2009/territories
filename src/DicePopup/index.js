import React from "react";
import Dice from "../libs/react-dice-3d";

const DicePopup = ({ dices }) => {
  return (
    <div>
      <Dice
        dices={[
          { backColor: "red", fontColor: "white" },
          { backColor: "blue", fontColor: "white" }
        ]}
        value={dices}
      />
    </div>
  );
};

export default DicePopup;
