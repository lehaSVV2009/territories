import React, { Component } from "react";
import AppBar from "../libs/territories-ui/AppBar";
import GithubIcon from "../libs/territories-icons/Github";
import IconButton from "../libs/territories-ui/IconButton";
import MenuItem from "../libs/territories-ui/MenuItem";
import Toolbar from "../libs/territories-ui/Toolbar";
import Tooltip from "../libs/territories-ui/Tooltip";
import { DetachedButton, Logo, WhiteSelect } from "./elements";

export default class Header extends Component {
  state = {
    lang: "en"
  };

  handleLogoClick = () => {
    window.open(`${process.env.PUBLIC_URL}/`, "_self");
  };

  handleGithubClick = () => {
    window.open("https://github.com/lehaSVV2009/territories");
  };

  handleRulesClick = () => {
    window.open(
      "https://pikabu.ru/story/prikolnaya_miniigra_stoit_poprobovat_6269129"
    );
  };

  handleLanguageChange = ({ target: { value } }) => {
    this.setState({ lang: value });
  };

  render() {
    const { lang } = this.state;

    return (
      <AppBar color="secondary" position="fixed">
        <Toolbar>
          <Logo onClick={this.handleLogoClick}>Territories</Logo>
          <WhiteSelect value={lang} onChange={this.handleLanguageChange}>
            <MenuItem value="en">en</MenuItem>
            {/* TODO implement i18n with russian */}
            {/* <MenuItem value="ru">ru</MenuItem> */}
          </WhiteSelect>
          <DetachedButton color="inherit" onClick={this.handleRulesClick}>
            Rules
          </DetachedButton>
          <Tooltip title="Github">
            <IconButton onClick={this.handleGithubClick}>
              <GithubIcon />
            </IconButton>
          </Tooltip>
        </Toolbar>
      </AppBar>
    );
  }
}
