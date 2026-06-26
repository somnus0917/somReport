import React from "react";
import ReactDOM from "react-dom/client";
import { getCurrentWindow } from '@tauri-apps/api/window';
import App from "./App";
import FloatingWidget from "./components/FloatingWidget";
import "./styles.css";

function applyTheme(theme: 'light' | 'dark' | 'system') {
  let resolved: 'light' | 'dark' = 'dark';
  if (theme === 'system') {
    const isLight = window.matchMedia('(prefers-color-scheme: light)').matches;
    resolved = isLight ? 'light' : 'dark';
  } else {
    resolved = theme;
  }
  document.documentElement.setAttribute('data-theme', resolved);
}

const savedTheme = (localStorage.getItem('somreport-theme') as 'light' | 'dark' | 'system') || 'system';
applyTheme(savedTheme);

window.matchMedia('(prefers-color-scheme: light)').addEventListener('change', () => {
  const currentTheme = localStorage.getItem('somreport-theme') || 'system';
  if (currentTheme === 'system') {
    applyTheme('system');
  }
});

const isFloatingWindow = getCurrentWindow().label === 'float';

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    {isFloatingWindow ? <FloatingWidget /> : <App />}
  </React.StrictMode>,
);
