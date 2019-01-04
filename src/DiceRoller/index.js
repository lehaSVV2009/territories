import React, { Component, Fragment } from "react";
import intl from "react-intl-universal";
import random from "lodash.random";

import Button from "../libs/territories-ui/Button";
import Dialog from "../libs/territories-ui/Dialog";
import Dices3d, { DICE_TYPES } from "../libs/react-dice-3d";

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
    const { buttonProps } = this.props;
    const { open, dices } = this.state;

    return (
      <Fragment>
        <Button
          size="large"
          variant="contained"
          onClick={this.handleRollDices}
          {...buttonProps}
        >
          {intl.get("player_controls.roll_dices")}
        </Button>
        <Dialog open={open} fullWidth>
          {/* Dice component is mounted again when dialog is open 
          cause of required dice init logic in componentDidMount */}
          {open && (
            <Dices3d
              dices={[
                {
                  type: DICE_TYPES.D6,
                  backColor: "red",
                  fontColor: "white",
                  value: dices[0]
                },
                {
                  type: DICE_TYPES.D6,
                  backColor: "blue",
                  fontColor: "white",
                  value: dices[1]
                }
              ]}
            />
          )}
        </Dialog>
      </Fragment>
    );
  }
}
export default DiceRoller;
