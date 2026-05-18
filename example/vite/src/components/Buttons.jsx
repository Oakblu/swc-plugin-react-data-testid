import React from "react";

export const Button = ({ children, onClick, disabled = false }) => {
  return (
    <button onClick={onClick} disabled={disabled}>
      {children}
    </button>
  );
};

export const IconButton = ({ icon, children, onClick }) => {
  return (
    <button onClick={onClick}>
      <span>{icon}</span>
      <span>{children}</span>
    </button>
  );
};

export const LinkButton = ({ href, children }) => (
  <a href={href}>
    <button>{children}</button>
  </a>
);
