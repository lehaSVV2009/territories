import React, { Component } from "react";
import Dialog from "../libs/territories-ui/Dialog";
import Button from "../libs/territories-ui/Button";
import Player from "../Player";

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

  render() {
    const { open } = this.state;
    const { gameover } = this.props;
    return (
      <Dialog open={open}>
        <h1>Congratulations</h1>
        {gameover &&
          (gameover.winner ? (
            <div>
              Winner: <Player player={gameover.winner} />
            </div>
          ) : (
            "Draw!"
          ))}
        <Button onClick={this.handleClose}>X</Button>
      </Dialog>
    );
  }
}

export default Congratulations;
