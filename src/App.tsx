import { useEffect } from "react";
import { HashRouter, Routes, Route, NavLink } from "react-router-dom";
import { QueryClientProvider } from "@tanstack/react-query";
import { queryClient } from "./lib/queryClient";
import { sendNotification } from "./api/tauri";
import Today from "./pages/Today";
import Reports from "./pages/Reports";
import Settings from "./pages/Settings";
import ModelConfig from "./pages/ModelConfig";

export default function App() {
  useEffect(() => {
    // Every minute, check if we need to show the work-end reminder
    const interval = setInterval(() => {
      const now = new Date();
      const currentHour = now.getHours();
      const currentMinute = now.getMinutes();

      // Remind the user between 18:00 and 18:30
      if (currentHour === 18 && currentMinute >= 0 && currentMinute <= 30) {
        const todayStr = now.toDateString();
        const lastRemindedDate = localStorage.getItem("last-reminded-date");

        if (lastRemindedDate !== todayStr) {
          localStorage.setItem("last-reminded-date", todayStr);
          sendNotification(
            "日报助手下班提醒",
            "下班时间快到了，别忘了点击“报告”页面生成今天的日报哦！"
          ).catch((e) => console.error("Failed to send reminder notification:", e));
        }
      }
    }, 60000);

    return () => clearInterval(interval);
  }, []);

  return (
    <QueryClientProvider client={queryClient}>
      <HashRouter>
        <div className="app-layout">
          <nav className="sidebar">
            <h1 className="sidebar-title">日报助手</h1>
            <NavLink to="/" end>今日</NavLink>
            <NavLink to="/reports">报告</NavLink>
            <NavLink to="/settings">设置</NavLink>
          </nav>
          <main className="main-content">
            <Routes>
              <Route path="/" element={<Today />} />
              <Route path="/reports" element={<Reports />} />
              <Route path="/settings" element={<Settings />} />
              <Route path="/settings/model/:role" element={<ModelConfig />} />
            </Routes>
          </main>
        </div>
      </HashRouter>
    </QueryClientProvider>
  );
}
