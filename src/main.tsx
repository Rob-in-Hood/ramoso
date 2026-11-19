import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import "./index.css";

// e2e builds replace Rust with mocks before anything calls it; other builds drop this branch.
if (import.meta.env.MODE === "e2e") {
  await import("../e2e/browser-mocks");
}

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
);
