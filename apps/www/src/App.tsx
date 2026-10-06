import { Navigate, Route, Routes } from "react-router-dom";
import { Layout } from "./components/Layout";
import { ScrollToTop } from "./components/ScrollToTop";
import { Cast } from "./pages/Cast";
import { Continuity } from "./pages/Continuity";
import { Download } from "./pages/Download";
import { Engine } from "./pages/Engine";
import { Environment } from "./pages/Environment";
import { Home } from "./pages/Home";
import { Interop } from "./pages/Interop";
import { LocalPage } from "./pages/Local";
import { Name } from "./pages/Name";
import { Packs } from "./pages/Packs";
import { Time } from "./pages/Time";
import { World } from "./pages/World";

export default function App() {
  return (
    <>
      <ScrollToTop />
      <Routes>
        <Route element={<Layout />}>
          <Route index element={<Home />} />
          <Route path="environment" element={<Environment />} />
          <Route path="continuity" element={<Continuity />} />
          <Route path="cast" element={<Cast />} />
          <Route path="time" element={<Time />} />
          <Route path="world" element={<World />} />
          <Route path="engine" element={<Engine />} />
          <Route path="packs" element={<Packs />} />
          <Route path="interop" element={<Interop />} />
          <Route path="local" element={<LocalPage />} />
          <Route path="name" element={<Name />} />
          <Route path="download" element={<Download />} />
          <Route path="*" element={<Navigate to="/" replace />} />
        </Route>
      </Routes>
    </>
  );
}
