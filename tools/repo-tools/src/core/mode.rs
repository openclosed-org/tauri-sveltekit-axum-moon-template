#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Mode {
    Warn,
    Strict,
}

impl Mode {
    pub(crate) fn is_strict(self) -> bool {
        matches!(self, Self::Strict)
    }

    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::Warn => "warn",
            Self::Strict => "strict",
        }
    }
}
