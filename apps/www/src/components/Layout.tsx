import { NavLink, Outlet, useLocation } from "react-router-dom";
import { Wordmark } from "../art/Ink";
import { pages } from "../seo/catalog";
import { Seo } from "../seo/Seo";

const story = pages.filter((page) => page.kind === "story" && page.nav);
const record = pages.filter((page) => page.kind !== "story" && page.nav);

export function Layout() {
  const { pathname } = useLocation();
  return (
    <div className="site">
      <Seo pathname={pathname} />
      <header className="top">
        <NavLink to="/" className="brand" aria-label="Asten home">
          <Wordmark />
        </NavLink>
        <nav aria-label="Product">
          {story.map((page) => (
            <NavLink
              key={page.path}
              to={page.path}
              end={page.path === "/"}
              className={({ isActive }) => (isActive ? "on" : "")}
            >
              {page.nav}
            </NavLink>
          ))}
          {record
            .filter((page) => page.kind === "record")
            .map((page) => (
              <NavLink
                key={page.path}
                to={page.path}
                className={({ isActive }) => (isActive ? "on" : "")}
              >
                {page.nav}
              </NavLink>
            ))}
        </nav>
        <NavLink to="/download" className="cta">
          Open the library
        </NavLink>
      </header>
      <Outlet />
      <footer className="bound">
        <div className="footer-brand">
          <Wordmark />
          <p>Asten is a Verne product.</p>
          <p>Let AI be the editor, not the writer. The files stay on this machine.</p>
        </div>
        <p className="verne-credit">
          <span className="built">Designed and built by</span>
          <img src="/verne-mark.png" alt="" />
          <span className="ltd">Verne Ltd.</span>
        </p>
        <nav className="footer-links" aria-label="Legal">
          {record
            .filter((page) => page.kind === "legal")
            .map((page) => (
              <NavLink key={page.path} to={page.path}>
                {page.nav}
              </NavLink>
            ))}
        </nav>
      </footer>
    </div>
  );
}
