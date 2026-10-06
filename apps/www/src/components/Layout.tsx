import { Suspense, useEffect, useRef } from "react";
import { Outlet, useLocation } from "react-router-dom";
import { loaderFor } from "../pages/loaders";
import { Seo } from "../seo/Seo";
import { Footer } from "./Footer";
import { Masthead } from "./Masthead";

export function Layout() {
  const { pathname, hash } = useLocation();
  const main = useRef<HTMLElement>(null);
  const first = useRef(true);
  // The arrival animation plays only after a client-side navigation, never on first load.
  const initialPath = useRef(pathname);
  const moved = useRef(false);
  if (pathname !== initialPath.current) moved.current = true;

  // On navigation: scroll to the top (or the anchor) and move focus into the page.
  useEffect(() => {
    if (first.current) {
      first.current = false;
      return;
    }
    main.current?.focus({ preventScroll: true });
    if (!hash) {
      window.scrollTo(0, 0);
      return;
    }
    // The page may still be loading its chunk, so wait for the anchor to exist.
    const id = decodeURIComponent(hash.slice(1));
    let frames = 0;
    let frame = 0;
    const seek = () => {
      const target = document.getElementById(id);
      if (target) target.scrollIntoView();
      else if (frames++ < 120) frame = window.requestAnimationFrame(seek);
    };
    seek();
    return () => window.cancelAnimationFrame(frame);
  }, [pathname, hash]);

  useReveal();
  usePrefetch();
  useMagnetic();

  return (
    <>
      <a className="skip-link" href="#main">
        Skip to content
      </a>
      <Seo pathname={pathname} />
      <Masthead />
      <main id="main" ref={main} tabIndex={-1} key={pathname} className={moved.current ? "page-enter" : undefined}>
        <Suspense fallback={<div className="page-loading" aria-busy="true" />}>
          <Outlet />
        </Suspense>
      </main>
      <Footer />
    </>
  );
}

const REVEAL = "[data-reveal]";

/**
 * Registers rise into place as they enter the viewport. Anything already on
 * screen when the page hydrates is shown at once, so nothing flickers. Pages
 * arrive lazily, so new sections are picked up whenever they enter the DOM,
 * not only when the address changes.
 */
function useReveal() {
  useEffect(() => {
    const reduced = window.matchMedia("(prefers-reduced-motion: reduce)").matches;
    const supported = "IntersectionObserver" in window && "MutationObserver" in window;
    if (reduced || !supported) {
      const showAll = () => document.querySelectorAll(`${REVEAL}:not(.is-in)`).forEach((node) => node.classList.add("is-in"));
      showAll();
      if (!("MutationObserver" in window)) return;
      const mutations = new MutationObserver(showAll);
      mutations.observe(document.body, { childList: true, subtree: true });
      return () => mutations.disconnect();
    }

    const intersections = new IntersectionObserver(
      (entries) => {
        for (const entry of entries) {
          if (entry.isIntersecting) {
            entry.target.classList.add("is-in");
            intersections.unobserve(entry.target);
          }
        }
      },
      { rootMargin: "0px 0px -8% 0px", threshold: 0.08 },
    );
    const watch = (scope: Element | Document) => {
      if (scope instanceof Element && scope.matches(`${REVEAL}:not(.is-in)`)) intersections.observe(scope);
      scope.querySelectorAll(`${REVEAL}:not(.is-in)`).forEach((node) => intersections.observe(node));
    };

    // First load: show what is already on screen, then hide the rest until reached.
    const height = window.innerHeight;
    document.querySelectorAll(REVEAL).forEach((node) => {
      const box = node.getBoundingClientRect();
      if (box.top < height * 0.92 && box.bottom > 0) node.classList.add("is-in");
    });
    document.documentElement.classList.add("reveal-ready");
    watch(document);

    // Later pages: watch sections as they are added.
    const mutations = new MutationObserver((records) => {
      for (const record of records) {
        record.addedNodes.forEach((node) => {
          if (node instanceof Element) watch(node);
        });
      }
    });
    mutations.observe(document.body, { childList: true, subtree: true });

    return () => {
      mutations.disconnect();
      intersections.disconnect();
    };
  }, []);
}

/** Fetch a page's code as soon as a link to it is hovered or focused. */
function usePrefetch() {
  useEffect(() => {
    const done = new Set<string>();
    function onIntent(event: Event) {
      const link = (event.target as Element | null)?.closest?.("a[href^='/']");
      const href = link?.getAttribute("href");
      if (!href || href.startsWith("//")) return;
      const path = href.split(/[?#]/)[0] ?? "/";
      if (done.has(path) || /\.[a-z0-9]+$/i.test(path)) return;
      done.add(path);
      loaderFor(path)().catch(() => done.delete(path));
    }
    document.addEventListener("pointerover", onIntent, { passive: true });
    document.addEventListener("focusin", onIntent);
    document.addEventListener("touchstart", onIntent, { passive: true });
    return () => {
      document.removeEventListener("pointerover", onIntent);
      document.removeEventListener("focusin", onIntent);
      document.removeEventListener("touchstart", onIntent);
    };
  }, []);
}

/** Primary buttons lean a few pixels toward the pointer. */
function useMagnetic() {
  useEffect(() => {
    const fine = window.matchMedia("(pointer: fine)").matches;
    const reduced = window.matchMedia("(prefers-reduced-motion: reduce)").matches;
    if (!fine || reduced) return;
    let active: HTMLElement | null = null;
    function onMove(event: PointerEvent) {
      const target = (event.target as Element | null)?.closest<HTMLElement>("[data-magnetic]") ?? null;
      if (active && active !== target) {
        active.style.removeProperty("--mx");
        active.style.removeProperty("--my");
      }
      active = target;
      if (!target) return;
      const box = target.getBoundingClientRect();
      const dx = (event.clientX - (box.left + box.width / 2)) / (box.width / 2);
      const dy = (event.clientY - (box.top + box.height / 2)) / (box.height / 2);
      target.style.setProperty("--mx", `${(dx * 4).toFixed(2)}px`);
      target.style.setProperty("--my", `${(dy * 3).toFixed(2)}px`);
    }
    document.addEventListener("pointermove", onMove, { passive: true });
    return () => document.removeEventListener("pointermove", onMove);
  }, []);
}
