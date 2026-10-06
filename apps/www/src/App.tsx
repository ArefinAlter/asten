import { lazy } from "react";
import { Route, Routes } from "react-router-dom";
import { Layout } from "./components/Layout";
import { audiences } from "./content/audiences";
import { features } from "./content/features";
import { articleList } from "./content/journal/meta";
import { legalList } from "./content/legal-index";
import { loaders } from "./pages/loaders";

const Home = lazy(() => loaders.home().then((m) => ({ default: m.Home })));
const HowItWorks = lazy(() => loaders.how().then((m) => ({ default: m.HowItWorks })));
const Features = lazy(() => loaders.features().then((m) => ({ default: m.Features })));
const FeaturePage = lazy(() => loaders.feature().then((m) => ({ default: m.FeaturePage })));
const AiStance = lazy(() => loaders.ai().then((m) => ({ default: m.AiStance })));
const AudiencePage = lazy(() => loaders.audience().then((m) => ({ default: m.AudiencePage })));
const Compare = lazy(() => loaders.compare().then((m) => ({ default: m.Compare })));
const Download = lazy(() => loaders.download().then((m) => ({ default: m.Download })));
const Roadmap = lazy(() => loaders.roadmap().then((m) => ({ default: m.Roadmap })));
const JournalIndex = lazy(() => loaders.journalIndex().then((m) => ({ default: m.JournalIndex })));
const ArticlePage = lazy(() => loaders.article().then((m) => ({ default: m.ArticlePage })));
const Glossary = lazy(() => loaders.reference().then((m) => ({ default: m.Glossary })));
const Faq = lazy(() => loaders.reference().then((m) => ({ default: m.Faq })));
const Guide = lazy(() => loaders.reference().then((m) => ({ default: m.Guide })));
const Name = lazy(() => loaders.name().then((m) => ({ default: m.Name })));
const About = lazy(() => loaders.company().then((m) => ({ default: m.About })));
const Contact = lazy(() => loaders.company().then((m) => ({ default: m.Contact })));
const Brand = lazy(() => loaders.brand().then((m) => ({ default: m.Brand })));
const LegalIndex = lazy(() => loaders.legal().then((m) => ({ default: m.LegalIndex })));
const LegalPage = lazy(() => loaders.legal().then((m) => ({ default: m.LegalPage })));
const NotFound = lazy(() => loaders.notFound().then((m) => ({ default: m.NotFound })));

export default function App() {
  return (
    <Routes>
      <Route element={<Layout />}>
        <Route index element={<Home />} />
        <Route path="how-it-works" element={<HowItWorks />} />
        <Route path="features" element={<Features />} />
        {features.map((feature) => (
          <Route key={feature.slug} path={feature.path.slice(1)} element={<FeaturePage feature={feature} />} />
        ))}
        <Route path="ai" element={<AiStance />} />
        {audiences.map((audience) => (
          <Route key={audience.slug} path={audience.path.slice(1)} element={<AudiencePage audience={audience} />} />
        ))}
        <Route path="compare" element={<Compare />} />
        <Route path="download" element={<Download />} />
        <Route path="roadmap" element={<Roadmap />} />
        <Route path="journal" element={<JournalIndex />} />
        {articleList.map((article) => (
          <Route key={article.slug} path={`journal/${article.slug}`} element={<ArticlePage slug={article.slug} />} />
        ))}
        <Route path="glossary" element={<Glossary />} />
        <Route path="faq" element={<Faq />} />
        <Route path="guide" element={<Guide />} />
        <Route path="name" element={<Name />} />
        <Route path="about" element={<About />} />
        <Route path="brand" element={<Brand />} />
        <Route path="contact" element={<Contact />} />
        <Route path="legal" element={<LegalIndex />} />
        {legalList.map((doc) => (
          <Route key={doc.path} path={doc.path.slice(1)} element={<LegalPage path={doc.path} />} />
        ))}
        <Route path="*" element={<NotFound />} />
      </Route>
    </Routes>
  );
}
