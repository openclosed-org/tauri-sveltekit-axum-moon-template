#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum OperationPhase {
    Plan,
    Preflight,
    Execute,
    Verify,
}

impl OperationPhase {
    fn label(self) -> &'static str {
        match self {
            Self::Plan => "plan",
            Self::Preflight => "preflight",
            Self::Execute => "execute",
            Self::Verify => "verify",
        }
    }
}

pub(crate) struct Operation {
    name: &'static str,
    apply: bool,
}

impl Operation {
    pub(crate) fn new(name: &'static str, apply: bool) -> Self {
        Self { name, apply }
    }

    pub(crate) fn phase(&self, phase: OperationPhase, message: impl AsRef<str>) {
        println!("[{}:{}] {}", self.name, phase.label(), message.as_ref());
    }

    pub(crate) fn is_apply(&self) -> bool {
        self.apply
    }
}
