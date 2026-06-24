import React from "react";
import ReactDOM from "react-dom/client";
import { getCurrentWindow } from '@tauri-apps/api/window';
import App from "./App";
import FloatingWidget from "./components/FloatingWidget";
import "./styles.css";

const isFloatingWindow = getCurrentWindow().label === 'float';

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    {isFloatingWindow ? <FloatingWidget /> : <App />}
  </React.StrictMode>,
);
