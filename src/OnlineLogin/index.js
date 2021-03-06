import React, { Component } from "react";
import intl from "react-intl-universal";

import Button from "../libs/territories-ui/Button";
import { Container, Item } from "../libs/territories-ui/Grid";
import { StyledTextField } from "./elements";

class OnlineLogin extends Component {
  constructor(props) {
    super(props);
    this.state = {
      changingPlayerName: props.playerName
    };
  }

  handlePlayerNameChange = ({ target: { value } }) => {
    this.setState({ changingPlayerName: value });
  };

  handleLogin = () => {
    // TODO add user to AUTH server
    this.props.onLogin(this.state.changingPlayerName);
  };

  getErrorMessage = (changingPlayerName, playersNames) => {
    if (!changingPlayerName) {
      return intl.get("online.name_not_empty");
    }
    if (!/^\w+$/.test(changingPlayerName)) {
      return intl.get("online.name_invalid_characters");
    }
    if (changingPlayerName.length > 15) {
      return intl.get("online.name_too_long");
    }
    // TODO load players names from AUTH server
    if (playersNames.includes(changingPlayerName)) {
      return intl.get("online.name_taken");
    }
    return null;
  };

  render() {
    const { changingPlayerName } = this.state;
    const { playersNames } = this.props;

    const errorMessage = this.getErrorMessage(changingPlayerName, playersNames);
    const hasError = !!errorMessage;

    return (
      <Container column alignItems="center">
        <Item>
          <StyledTextField
            label={intl.get("online.enter_name")}
            error={hasError}
            helperText={errorMessage}
            margin="normal"
            value={changingPlayerName}
            variant="outlined"
            onKeyPress={({ key }) =>
              !hasError && key === "Enter" && this.handleLogin()
            }
            onChange={this.handlePlayerNameChange}
          />
        </Item>
        <Item>
          <Button
            disabled={hasError}
            color="primary"
            size="large"
            variant="extendedFab"
            onClick={this.handleLogin}
          >
            {intl.get("online.login")}
          </Button>
        </Item>
      </Container>
    );
  }
}

export default OnlineLogin;
