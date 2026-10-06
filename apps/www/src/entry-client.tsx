import { StrictMode } from "react";
import { createRoot, hydrateRoot } from "react-dom/client";
import { BrowserRouter } from "react-router-dom";
import App from "./App";
import { loaderFor } from "./pages/loaders";
import "./styles/index.css";

const root = document.getElementById("root")!;
const tree = (
  <StrictMode>
    <BrowserRouter>
      <App />
    </BrowserRouter>
  </StrictMode>
);

// Load this page's chunk before hydrating, so the server HTML is adopted in one pass.
loaderFor(window.location.pathname)()
  .catch(() => undefined)
  .then(() => {
    if (root.firstElementChild) hydrateRoot(root, tree);
    else createRoot(root).render(tree);
  });
