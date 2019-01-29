import intl from "react-intl-universal";

const SUPPORTED_LOCALES = { en: "en", ru: "ru" };

const locales = {
  en: require("./locales/en.json"),
  ru: require("./locales/ru.json")
};

export default () => {
  let currentLocale = intl.determineLocale({
    cookieLocaleKey: "lang"
  });
  if (!SUPPORTED_LOCALES[currentLocale]) {
    currentLocale = "en";
  }

  return intl.init({ currentLocale, locales });
};
