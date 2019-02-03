import React, { Component, Fragment } from "react";

import Dialog from "../libs/territories-ui/Dialog";
import Dices3d, { DICE_TYPES } from "../libs/react-dice-3d";

class DicesWindow extends Component {
  state = {
    open: false
  };

  componentDidUpdate(prevProps) {
    const { rollingDices } = this.props;
    if (rollingDices && rollingDices !== prevProps.rollingDices) {
      this.setState({ open: true });
      setTimeout(() => {
        this.setState({ open: false });
        this.props.onFinishRoll();
      }, 3000);
    }
  }

  render() {
    const { rollingDices } = this.props;
    const { open } = this.state;

    return (
      <Fragment>
        <Dialog open={open} fullWidth>
          {/* Dice component is mounted again when dialog is open 
          cause of required dice init logic in componentDidMount */}
          {open && rollingDices && (
            <Dices3d
              dices={[
                {
                  type: DICE_TYPES.D6,
                  backColor: "red",
                  fontColor: "white",
                  value: rollingDices[0]
                },
                {
                  type: DICE_TYPES.D6,
                  backColor: "blue",
                  fontColor: "white",
                  value: rollingDices[1]
                }
              ]}
            />
          )}
        </Dialog>
      </Fragment>
    );
  }
}
export default DicesWindow;
