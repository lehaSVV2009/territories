import React, { Component } from "react";
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

  handleNewGame = () => {
    window.open("/", "_self");
  };

  render() {
    const { open } = this.state;
    const { gameover } = this.props;
    return (
      <Dialog open={open} fullWidth onClose={this.handleClose}>
        {gameover && (
          <DialogTitle>{gameover.winner ? "Winner" : "Draw"}</DialogTitle>
        )}
        {gameover && (
          <DialogContent>
            {gameover.winner && <Player player={gameover.winner} />}
            <CenteredImage src={congratulationsImage} alt="Congratulations" />
          </DialogContent>
        )}
        <DialogActions>
          <Button
            variant="contained"
            color="primary"
            onClick={this.handleNewGame}
          >
            New Game
          </Button>
          <Button variant="contained" onClick={this.handleClose}>
            Cancel
          </Button>
        </DialogActions>
      </Dialog>
    );
  }
}

export default Congratulations;
