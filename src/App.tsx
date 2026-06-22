import { HashRouter, Routes, Route, NavLink } from "react-router-dom";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";

const queryClient = new QueryClient();

function Today() {
  return <div><h2>Today</h2><p>Track your work for today.</p></div>;
}

function Reports() {
  return <div><h2>Reports</h2><p>View and generate reports.</p></div>;
}

function Settings() {
  return <div><h2>Settings</h2><p>Configure the application.</p></div>;
}

export default function App() {
  return (
    <QueryClientProvider client={queryClient}>
      <HashRouter>
        <nav>
          <NavLink to="/">Today</NavLink>{" | "}
          <NavLink to="/reports">Reports</NavLink>{" | "}
          <NavLink to="/settings">Settings</NavLink>
        </nav>
        <Routes>
          <Route path="/" element={<Today />} />
          <Route path="/reports" element={<Reports />} />
          <Route path="/settings" element={<Settings />} />
        </Routes>
      </HashRouter>
    </QueryClientProvider>
  );
}
