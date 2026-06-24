import { HashRouter, Routes, Route, NavLink } from "react-router-dom";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import Today from "./pages/Today";
import Reports from "./pages/Reports";
import Settings from "./pages/Settings";
import ModelConfig from "./pages/ModelConfig";

const queryClient = new QueryClient();

export default function App() {
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
