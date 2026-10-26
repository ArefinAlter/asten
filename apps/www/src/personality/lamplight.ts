import { useEffect } from "react";
import { fbm1, reducedMotion } from "./random";

/**
 * The night theme is lamplight. The lamp's pool of warm light on the page flickers a little, as flame does:
 * not on a timer but on layered noise, about twelve times a second, and only while it is night and the tab is
 * showing. The value lands in --lamp (0.9 to 1) and the stylesheet decides what to do with it. Reduced motion
 * holds it still.
 */
export function useLamplight() {
  useEffect(() => {
    const root = document.documentElement;
    root.style.setProperty("--lamp", "1");
    if (reducedMotion()) return;
    const dark = window.matchMedia("(prefers-color-scheme: dark)");
    let raf = 0;
    let last = 0;
    const night = () => {
      const set = root.getAttribute("data-theme");
      return set === "night" || (set !== "day" && dark.matches);
    };
    const frame = (now: number) => {
      raf = 0;
      if (document.hidden || !night()) {
        root.style.setProperty("--lamp", "1");
        return;
      }
      if (now - last > 80) {
        last = now;
        const t = now / 1000;
        const flick = 0.5 + 0.5 * fbm1(t * 1.7, 4, 4);
        const sway = 0.5 + 0.5 * fbm1(t * 0.35 + 9, 6, 2);
        root.style.setProperty("--lamp", (0.9 + 0.08 * flick + 0.02 * sway).toFixed(3));
      }
      raf = requestAnimationFrame(frame);
    };
    const start = () => {
      if (!raf && !document.hidden && night()) raf = requestAnimationFrame(frame);
    };
    const obs = new MutationObserver(start);
    obs.observe(root, { attributes: true, attributeFilter: ["data-theme"] });
    dark.addEventListener("change", start);
    document.addEventListener("visibilitychange", start);
    start();
    return () => {
      cancelAnimationFrame(raf);
      obs.disconnect();
      dark.removeEventListener("change", start);
      document.removeEventListener("visibilitychange", start);
    };
  }, []);
}
