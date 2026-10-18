import React from "react";
import ReactDOM from "react-dom/client";

import App from "./App";
import { initTheme } from "./stores/theme";
import "./styles/globals.css";

initTheme();

const rootElement = document.getElementById("root");
if (!rootElement) {
  throw new Error("index.html 缺少 #root 挂载点");
}

ReactDOM.createRoot(rootElement).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
);
