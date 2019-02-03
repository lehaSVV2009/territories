import React, { Component } from "react";
import intl from "react-intl-universal";

import Button from "../libs/territories-ui/Button";
import Dialog from "../libs/territories-ui/Dialog";
import DialogActions from "../libs/territories-ui/DialogActions";
import DialogContent from "../libs/territories-ui/DialogContent";
import DialogTitle from "../libs/territories-ui/DialogTitle";
import Player from "../Player";
import congratulationsImage from "../images/congratulations.gif";
import { CenteredImage } from "./elements";

class Congratulations extends Component {
  state = {
    open: false
  };

  componentDidMount() {
    if (this.props.gameover) {
      this.handleOpen();
    }
  }

  componentDidUpdate(prevProps) {
    if (this.props.gameover && prevProps.gameover !== this.props.gameover) {
      this.handleOpen();
    }
  }

  handleOpen = () => {
    this.setState({ open: true });
  };

  handleClose = () => {
    this.setState({ open: false });
  };

  handleGoHome = () => {
    window.open(`${process.env.PUBLIC_URL}/`, "_self");
  };

  render() {
    const { open } = this.state;
    const { gameover } = this.props;
    return (
      <Dialog open={open} fullWidth onClose={this.handleClose}>
        {gameover && (
          <DialogTitle>
            {gameover.winner
              ? intl.get("congratulations.winner")
              : intl.get("congratulations.draw")}
          </DialogTitle>
        )}
        {gameover && (
          <DialogContent>
            {gameover.winner && <Player player={gameover.winner} />}
            <CenteredImage
              src={congratulationsImage}
              alt={intl.get("congratulations.image")}
            />
          </DialogContent>
        )}
        <DialogActions>
          <Button
            variant="contained"
            color="primary"
            onClick={this.handleClose}
          >
            {intl.get("congratulations.ok")}
          </Button>
        </DialogActions>
      </Dialog>
    );
  }
}

export default Congratulations;
