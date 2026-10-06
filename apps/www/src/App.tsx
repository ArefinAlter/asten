import { Navigate, Route, Routes } from "react-router-dom";
import { Layout } from "./components/Layout";
import { ScrollToTop } from "./components/ScrollToTop";
import { Continuity } from "./pages/Continuity";
import { Download } from "./pages/Download";
import { Environment } from "./pages/Environment";
import { Home } from "./pages/Home";
import { Interop } from "./pages/Interop";
import { LocalPage } from "./pages/Local";

export default function App() {
  return (
    <>
      <ScrollToTop />
      <Routes>
      <Route element={<Layout />}>
        <Route index element={<Home />} />
        <Route path="environment" element={<Environment />} />
        <Route path="continuity" element={<Continuity />} />
        <Route path="interop" element={<Interop />} />
        <Route path="local" element={<LocalPage />} />
        <Route path="download" element={<Download />} />
        <Route path="*" element={<Navigate to="/" replace />} />
      </Route>
    </Routes>
    </>
  );
}
