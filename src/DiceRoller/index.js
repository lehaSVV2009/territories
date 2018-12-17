import React, { Component } from "react";
import random from "lodash.random";
import Button from "../libs/territories-ui/Button";
import Dialog from "../libs/territories-ui/Dialog";
import Dice from "../libs/react-dice-3d";

const generateRandomDices = () => {
  return [random(1, 6), random(1, 6)];
};

class DiceRoller extends Component {
  state = {
    open: false,
    dices: null
  };

  handleRollDices = () => {
    const dices = generateRandomDices();
    this.setState({ open: true, dices });
    setTimeout(() => {
      this.setState({ open: false, dices: null });
      this.props.onRoll(dices);
    }, 3000);
  };

  render() {
    const { open, dices } = this.state;

    return (
      <div>
        <Button onClick={this.handleRollDices}>Roll Dices</Button>
        <Dialog open={open}>
          {/* Dice component is mounted again when dialog is open 
          cause of required dice init logic in componentDidMount */}
          {open && (
            <Dice
              options={[
                { backColor: "red", fontColor: "white" },
                { backColor: "blue", fontColor: "white" }
              ]}
              value={dices}
            />
          )}
        </Dialog>
      </div>
    );
  }
}
export default DiceRoller;
