use margin_domain::{Diagnostic, Document, Project};

pub struct AnalysisContext<'a> {
    pub project: &'a Project,
    pub documents: &'a [Document],
}

pub trait Rule: Send + Sync {
    fn code(&self) -> &'static str;
    fn evaluate(&self, context: &AnalysisContext<'_>, output: &mut Vec<Diagnostic>);
}

#[derive(Default)]
pub struct DiagnosticEngine {
    rules: Vec<Box<dyn Rule>>,
}

impl DiagnosticEngine {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&mut self, rule: impl Rule + 'static) {
        self.rules.push(Box::new(rule));
    }

    #[must_use]
    pub fn analyze(&self, context: &AnalysisContext<'_>) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        for rule in &self.rules {
            rule.evaluate(context, &mut diagnostics);
        }
        diagnostics
    }

    #[must_use]
    pub fn rule_codes(&self) -> Vec<&'static str> {
        self.rules.iter().map(|rule| rule.code()).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::DiagnosticEngine;

    #[test]
    fn empty_engine_has_no_rules() {
        assert!(DiagnosticEngine::new().rule_codes().is_empty());
    }
}
