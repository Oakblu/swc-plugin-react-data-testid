import React from "react";

// Simple button component
export const Button = ({ children, onClick, disabled = false }) => {
  return (
    <button onClick={onClick} disabled={disabled}>
      {children}
    </button>
  );
};

// Button with icon
export const IconButton = ({ icon, children, onClick }) => {
  return (
    <button onClick={onClick}>
      <span>{icon}</span>
      <span>{children}</span>
    </button>
  );
};

// Arrow function component
export const LinkButton = ({ href, children }) => (
  <a href={href}>
    <button>{children}</button>
  </a>
);
