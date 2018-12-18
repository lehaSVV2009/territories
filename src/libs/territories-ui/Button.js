import React from "react";

// TODO improve button styling
const Button = ({ children, ...props }) => (
  <button {...props}>{children}</button>
);

export default Button;
