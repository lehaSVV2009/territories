import React, { Component } from "react";
import AppBar from "../libs/territories-ui/AppBar";
import MenuItem from "../libs/territories-ui/MenuItem";
import Toolbar from "../libs/territories-ui/Toolbar";
import { DetachedButton, Logo, WhiteSelect } from "./elements";

export default class Header extends Component {
  state = {
    lang: "en"
  };

  handleLogoClick = () => {
    window.open("/", "_self");
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
      <AppBar position="fixed">
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
          <DetachedButton color="inherit" onClick={this.handleGithubClick}>
            Github
          </DetachedButton>
        </Toolbar>
      </AppBar>
    );
  }
}
