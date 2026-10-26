import { useEffect, useRef } from "react";
import { Manuscript } from "./Manuscript";
import { clamp, fbm1, reducedMotion } from "./random";

/**
 * The hero's picture: a painted window onto dusk at Deir el-Bahri, Thoth standing in it with his palette and
 * reed, and, laid across its corner, a page of the sample novel with the red pencil at work on it. The window
 * answers the pointer and the scroll with a smoothed drift (layers move by different amounts), plus a slow,
 * noise-driven breath in the sky. It stops when it is off screen or the tab is hidden; with reduced motion it
 * simply stands still.
 */
export function HeroDesk() {
  const root = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const el = root.current;
    if (!el || reducedMotion()) return;
    let tx = 0;
    let ty = 0;
    let cx = 0;
    let cy = 0;
    let ts = 0;
    let cs = 0;
    let raf = 0;
    let visible = true;
    let lastPaint = 0;

    const frame = (now: number) => {
      raf = 0;
      if (!visible || document.hidden) return;
      cx += (tx - cx) * 0.075;
      cy += (ty - cy) * 0.075;
      cs += (ts - cs) * 0.12;
      if (now - lastPaint >= 32) {
        // about 30 frames a second is plenty for a slow drift
        lastPaint = now;
        const t = now / 1000;
        el.style.setProperty("--px", cx.toFixed(4));
        el.style.setProperty("--py", cy.toFixed(4));
        el.style.setProperty("--sy", cs.toFixed(4));
        el.style.setProperty("--bx", (fbm1(t * 0.11, 3) * 5).toFixed(2));
        el.style.setProperty("--by", (fbm1(t * 0.09 + 20, 8) * 2.5).toFixed(2));
      }
      raf = requestAnimationFrame(frame);
    };
    const kick = () => {
      if (!raf && visible && !document.hidden) raf = requestAnimationFrame(frame);
    };
    const onMove = (e: PointerEvent) => {
      const r = el.getBoundingClientRect();
      tx = clamp((e.clientX - (r.left + r.width / 2)) / (window.innerWidth / 2), -1, 1);
      ty = clamp((e.clientY - (r.top + r.height / 2)) / (window.innerHeight / 2), -1, 1);
    };
    const onScroll = () => {
      ts = clamp(window.scrollY / Math.max(1, window.innerHeight));
    };
    const io = new IntersectionObserver(([entry]) => {
      visible = entry.isIntersecting;
      if (visible) kick();
    });
    io.observe(el);
    window.addEventListener("pointermove", onMove, { passive: true });
    window.addEventListener("scroll", onScroll, { passive: true });
    document.addEventListener("visibilitychange", kick);
    kick();
    return () => {
      cancelAnimationFrame(raf);
      io.disconnect();
      window.removeEventListener("pointermove", onMove);
      window.removeEventListener("scroll", onScroll);
      document.removeEventListener("visibilitychange", kick);
    };
  }, []);

  return (
    <div className="hero-desk" ref={root}>
      <figure className="window">
        <div className="window-frame">
          <img
            className="window-sky"
            src="/art/hero-window.webp"
            width={1040}
            height={1210}
            alt="Dusk over the golden cliffs at Deir el-Bahri, painted in oil, with ibis-headed Thoth standing in the foreground holding a scribe’s palette and a reed pen."
            fetchPriority="high"
            decoding="async"
          />
          <img className="window-thoth" src="/art/thoth.webp" width={514} height={1075} alt="" decoding="async" />
          <span className="window-glow" aria-hidden="true" />
        </div>
      </figure>
      <Manuscript />
    </div>
  );
}
