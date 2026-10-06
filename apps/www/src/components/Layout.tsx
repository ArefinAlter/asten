import { NavLink, Outlet } from "react-router-dom";
import { Wordmark } from "../art/Ink";

const LINKS = [
  ["/", "Notice"],
  ["/environment", "Desk"],
  ["/continuity", "Continuity"],
  ["/interop", "Guests"],
  ["/local", "Local"],
] as const;

export function Layout() {
  return (
    <div className="site">
      <header className="top">
        <NavLink to="/" className="brand" aria-label="Margin home">
          <Wordmark />
        </NavLink>
        <nav>
          {LINKS.map(([to, label]) => (
            <NavLink
              key={to}
              to={to}
              end={to === "/"}
              className={({ isActive }) => (isActive ? "on" : "")}
            >
              {label}
            </NavLink>
          ))}
        </nav>
        <NavLink to="/download" className="cta">
          Open the library
        </NavLink>
      </header>
      <Outlet />
      <footer className="bound">
        <Wordmark />
        <p>Let AI be the editor, not the writer.</p>
        <p>Local first. The files stay on this machine.</p>
      </footer>
    </div>
  );
}
