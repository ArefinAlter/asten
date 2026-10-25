import { useEffect, useRef, useState } from "react";
import { Link, NavLink, useLocation } from "react-router-dom";
import { Glyph } from "../art/Glyphs";
import { audiences } from "../content/audiences";
import { features } from "../content/features";
import { BrandLink } from "./Brand";
import { Arrow, Chevron } from "./Icons";
import { ThemeToggle } from "./ThemeToggle";

type MenuId = "features" | "writers" | null;

export function Masthead() {
  const { pathname } = useLocation();
  const [menu, setMenu] = useState<MenuId>(null);
  const [mobile, setMobile] = useState(false);
  const header = useRef<HTMLElement>(null);
  const progress = useRef<HTMLDivElement>(null);
  const closeTimer = useRef<number | undefined>(undefined);

  // Close everything on navigation.
  useEffect(() => {
    setMenu(null);
    setMobile(false);
  }, [pathname]);

  // Escape closes; clicking outside closes.
  useEffect(() => {
    function onKey(event: KeyboardEvent) {
      if (event.key === "Escape") {
        setMenu(null);
        setMobile(false);
      }
    }
    function onPointer(event: PointerEvent) {
      if (header.current && !header.current.contains(event.target as Node)) setMenu(null);
    }
    document.addEventListener("keydown", onKey);
    document.addEventListener("pointerdown", onPointer);
    return () => {
      document.removeEventListener("keydown", onKey);
      document.removeEventListener("pointerdown", onPointer);
    };
  }, []);

  // Lock page scroll under the mobile menu.
  useEffect(() => {
    document.body.style.overflow = mobile ? "hidden" : "";
    return () => {
      document.body.style.overflow = "";
    };
  }, [mobile]);

  // Reading progress (a red rubric line) and hide-on-scroll-down.
  useEffect(() => {
    let lastY = window.scrollY;
    let frame = 0;
    function update() {
      frame = 0;
      const y = window.scrollY;
      const max = document.documentElement.scrollHeight - window.innerHeight;
      progress.current?.style.setProperty("--progress", String(max > 0 ? Math.min(1, y / max) : 0));
      const node = header.current;
      if (node) {
        node.classList.toggle("is-scrolled", y > 8);
        const goingDown = y > lastY + 4;
        const goingUp = y < lastY - 4;
        if (goingDown && y > 280 && !node.matches(":focus-within")) node.classList.add("is-hidden");
        if (goingUp || y < 120) node.classList.remove("is-hidden");
      }
      lastY = y;
    }
    function onScroll() {
      if (!frame) frame = window.requestAnimationFrame(update);
    }
    update();
    window.addEventListener("scroll", onScroll, { passive: true });
    window.addEventListener("resize", onScroll);
    return () => {
      window.removeEventListener("scroll", onScroll);
      window.removeEventListener("resize", onScroll);
      if (frame) window.cancelAnimationFrame(frame);
    };
  }, []);

  function openSoon(id: MenuId) {
    window.clearTimeout(closeTimer.current);
    setMenu(id);
  }

  function closeSoon() {
    window.clearTimeout(closeTimer.current);
    closeTimer.current = window.setTimeout(() => setMenu(null), 160);
  }

  const hoverProps = (id: Exclude<MenuId, null>) => ({
    onPointerEnter: (event: React.PointerEvent) => {
      if (event.pointerType === "mouse") openSoon(id);
    },
    onPointerLeave: (event: React.PointerEvent) => {
      if (event.pointerType === "mouse") closeSoon();
    },
  });

  return (
    <>
      <header className="masthead" ref={header}>
        <div className="wrap masthead-inner">
          <BrandLink />
          <nav className="nav" aria-label="Primary">
            <NavLink to="/how-it-works" className={({ isActive }) => `nav-link${isActive ? " is-active" : ""}`}>
              How it works
            </NavLink>

            <div className="nav-item" {...hoverProps("features")}>
              <button
                type="button"
                className="nav-trigger"
                aria-expanded={menu === "features"}
                aria-controls="menu-features"
                onClick={() => setMenu(menu === "features" ? null : "features")}
              >
                Features <Chevron />
              </button>
              <div id="menu-features" className={`nav-menu${menu === "features" ? " is-open" : ""}`}>
                <div className="menu-grid">
                  {features.map((feature) => (
                    <Link key={feature.slug} to={feature.path} className="menu-link">
                      <Glyph name={feature.glyph} />
                      <strong>{feature.name}</strong>
                      <span>{feature.short}</span>
                    </Link>
                  ))}
                </div>
                <div className="menu-foot">
                  <span>Every feature runs on your own computer.</span>
                  <Link to="/features" className="link-arrow">
                    All features <Arrow />
                  </Link>
                </div>
              </div>
            </div>

            <div className="nav-item" {...hoverProps("writers")}>
              <button
                type="button"
                className="nav-trigger"
                aria-expanded={menu === "writers"}
                aria-controls="menu-writers"
                onClick={() => setMenu(menu === "writers" ? null : "writers")}
              >
                For writers <Chevron />
              </button>
              <div id="menu-writers" className={`nav-menu nav-menu--narrow${menu === "writers" ? " is-open" : ""}`}>
                <div className="menu-grid">
                  {audiences.map((audience) => (
                    <Link key={audience.slug} to={audience.path} className="menu-link">
                      <Glyph name={audience.glyph} />
                      <strong>{audience.name}</strong>
                      <span>{audience.short}</span>
                    </Link>
                  ))}
                </div>
                <div className="menu-foot">
                  <span>Comparisons with other kinds of tools</span>
                  <Link to="/compare" className="link-arrow">
                    Compare <Arrow />
                  </Link>
                </div>
              </div>
            </div>

            <NavLink to="/ai" className={({ isActive }) => `nav-link${isActive ? " is-active" : ""}`}>
              Our AI stance
            </NavLink>
            <NavLink to="/journal" className={({ isActive }) => `nav-link${isActive ? " is-active" : ""}`}>
              Journal
            </NavLink>
          </nav>

          <div className="masthead-actions">
            <ThemeToggle />
            <Link to="/download" className="btn btn--primary btn--small" data-magnetic>
              Try the pre-alpha
            </Link>
            <button
              type="button"
              className="menu-button"
              aria-expanded={mobile}
              aria-controls="mobile-nav"
              aria-label={mobile ? "Close menu" : "Open menu"}
              onClick={() => setMobile(!mobile)}
            >
              <span />
            </button>
          </div>
        </div>
        <div className="progress" ref={progress} aria-hidden="true" />
      </header>

      <nav id="mobile-nav" className={`mobile-nav${mobile ? " is-open" : ""}`} aria-label="Mobile" aria-hidden={!mobile}>
        <ul>
          <li><Link to="/how-it-works" tabIndex={mobile ? 0 : -1}>How it works</Link></li>
          <li><Link to="/ai" tabIndex={mobile ? 0 : -1}>Our AI stance</Link></li>
          <li><Link to="/journal" tabIndex={mobile ? 0 : -1}>Journal</Link></li>
          <li><Link to="/roadmap" tabIndex={mobile ? 0 : -1}>Roadmap</Link></li>
        </ul>
        <h2>Features</h2>
        <ul>
          {features.map((feature) => (
            <li key={feature.slug}>
              <Link to={feature.path} tabIndex={mobile ? 0 : -1}>
                <Glyph name={feature.glyph} /> {feature.name}
              </Link>
            </li>
          ))}
        </ul>
        <h2>For writers</h2>
        <ul>
          {audiences.map((audience) => (
            <li key={audience.slug}>
              <Link to={audience.path} tabIndex={mobile ? 0 : -1}>
                <Glyph name={audience.glyph} /> {audience.name}
              </Link>
            </li>
          ))}
        </ul>
        <Link to="/download" className="btn btn--primary" tabIndex={mobile ? 0 : -1}>
          Try the pre-alpha
        </Link>
      </nav>
    </>
  );
}
