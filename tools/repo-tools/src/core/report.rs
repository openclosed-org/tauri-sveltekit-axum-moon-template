use crate::core::mode::Mode;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Severity {
    Info,
    Warn,
    Error,
}

#[derive(Debug, Clone)]
pub(crate) struct Issue {
    pub(crate) severity: Severity,
    pub(crate) scope: String,
    pub(crate) message: String,
    pub(crate) evidence: Option<String>,
    pub(crate) remediation: Option<String>,
}

impl Issue {
    pub(crate) fn new(
        severity: Severity,
        scope: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            severity,
            scope: scope.into(),
            message: message.into(),
            evidence: None,
            remediation: None,
        }
    }

    pub(crate) fn error(scope: impl Into<String>, message: impl Into<String>) -> Self {
        Self::new(Severity::Error, scope, message)
    }

    pub(crate) fn warn(scope: impl Into<String>, message: impl Into<String>) -> Self {
        Self::new(Severity::Warn, scope, message)
    }

    #[allow(dead_code)]
    pub(crate) fn info(scope: impl Into<String>, message: impl Into<String>) -> Self {
        Self::new(Severity::Info, scope, message)
    }
}

#[derive(Debug, Clone)]
pub(crate) struct Report {
    title: String,
    mode: Mode,
    issues: Vec<Issue>,
}

impl Report {
    pub(crate) fn new(title: impl Into<String>, mode: Mode) -> Self {
        Self {
            title: title.into(),
            mode,
            issues: Vec::new(),
        }
    }

    #[allow(dead_code)]
    pub(crate) fn push(&mut self, issue: Issue) {
        self.issues.push(issue);
    }

    pub(crate) fn extend(&mut self, issues: impl IntoIterator<Item = Issue>) {
        self.issues.extend(issues);
    }

    #[allow(dead_code)]
    pub(crate) fn is_empty(&self) -> bool {
        self.issues.is_empty()
    }

    pub(crate) fn error_count(&self) -> usize {
        self.issues
            .iter()
            .filter(|issue| issue.severity == Severity::Error)
            .count()
    }

    pub(crate) fn warning_count(&self) -> usize {
        self.issues
            .iter()
            .filter(|issue| issue.severity == Severity::Warn)
            .count()
    }

    pub(crate) fn print(&self) {
        print_mode_header(&self.title, self.mode);
        for issue in &self.issues {
            println!(
                "[{}] {}: {}",
                severity_label(issue.severity),
                issue.scope,
                issue.message
            );
            if let Some(evidence) = &issue.evidence {
                println!("  evidence: {evidence}");
            }
            if let Some(remediation) = &issue.remediation {
                println!("  remediation: {remediation}");
            }
        }
        if !self.issues.is_empty() {
            println!(
                "Summary: {} error(s), {} warning(s)",
                self.error_count(),
                self.warning_count()
            );
        }
    }

    pub(crate) fn exit_if_needed(&self) {
        let code = self.status_code();
        if code != 0 {
            std::process::exit(code);
        }
    }

    pub(crate) fn status_code(&self) -> i32 {
        status_code(self.mode, self.error_count(), self.warning_count())
    }
}

fn severity_label(severity: Severity) -> &'static str {
    match severity {
        Severity::Info => "INFO",
        Severity::Warn => "WARN",
        Severity::Error => "ERROR",
    }
}

pub(crate) fn print_mode_header(name: &str, mode: Mode) {
    println!("=== {name} ({}) ===", mode.label());
}

pub(crate) fn status_code(mode: Mode, error_count: usize, warning_count: usize) -> i32 {
    if error_count > 0 || (mode.is_strict() && warning_count > 0) {
        1
    } else {
        0
    }
}
