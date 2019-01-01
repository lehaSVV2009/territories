import React, { Component } from "react";
import AppBar from "../libs/territories-ui/AppBar";
import Button from "../libs/territories-ui/Button";
import MenuItem from "../libs/territories-ui/MenuItem";
import Select from "../libs/territories-ui/Select";
import Toolbar from "../libs/territories-ui/Toolbar";

export default class Header extends Component {
  state = {
    lang: "en"
  };

  handleGithubClick = () => {
    window.open("https://github.com/lehaSVV2009/territories", "_blank");
  };

  handleRulesClick = () => {
    window.open(
      "https://pikabu.ru/story/prikolnaya_miniigra_stoit_poprobovat_6269129",
      "_blank"
    );
  };

  handleLanguageChange = ({ target: { value } }) => {
    this.setState({ lang: value });
  };

  render() {
    const { lang } = this.state;

    return (
      <AppBar position="fixed">
        <Toolbar>
          {/* TODO replace style with classname */}
          <span style={{ flexGrow: 1, textTransform: "uppercase" }}>
            Territories
          </span>
          <Select
            value={lang}
            onChange={this.handleLanguageChange}
            style={{ color: "white" }}
          >
            <MenuItem value="en">en</MenuItem>
            {/* TODO implement i18n with russian */}
            {/* <MenuItem value="ru">ru</MenuItem> */}
          </Select>
          <Button color="inherit" onClick={this.handleRulesClick}>
            Rules
          </Button>
          <Button color="inherit" onClick={this.handleGithubClick}>
            Github
          </Button>
        </Toolbar>
      </AppBar>
    );
  }
}
