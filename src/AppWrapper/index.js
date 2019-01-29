import React, { Component } from "react";
import Cookies from "js-cookie";
import intl from "react-intl-universal";

import AppBar from "../libs/territories-ui/AppBar";
import GithubIcon from "../libs/territories-icons/Github";
import IconButton from "../libs/territories-ui/IconButton";
import MenuItem from "../libs/territories-ui/MenuItem";
import Toolbar from "../libs/territories-ui/Toolbar";
import Tooltip from "../libs/territories-ui/Tooltip";
import {
  DetachedButton,
  FixedAppBarMargin,
  Logo,
  WhiteSelect
} from "./elements";

export default class AppWrapper extends Component {
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
    Cookies.set("lang", value);
    window.location.reload();
  };

  render() {
    const { currentLocale, locales } = intl.getInitOptions();

    return (
      <div>
        <AppBar color="secondary" position="fixed">
          <Toolbar>
            <Logo onClick={this.handleLogoClick}>
              {intl.get("app_bar.title")}
            </Logo>
            <WhiteSelect
              value={currentLocale}
              onChange={this.handleLanguageChange}
            >
              {Object.keys(locales).map(lang => (
                <MenuItem key={lang} value={lang}>
                  {lang}
                </MenuItem>
              ))}
            </WhiteSelect>
            <DetachedButton color="inherit" onClick={this.handleRulesClick}>
              {intl.get("app_bar.rules")}
            </DetachedButton>
            <Tooltip title="Github">
              <IconButton onClick={this.handleGithubClick}>
                <GithubIcon />
              </IconButton>
            </Tooltip>
          </Toolbar>
        </AppBar>
        <FixedAppBarMargin>{this.props.children}</FixedAppBarMargin>
      </div>
    );
  }
}
